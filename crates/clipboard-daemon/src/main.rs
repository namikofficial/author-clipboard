//! author-clipboard-daemon: Background clipboard monitoring daemon
//!
//! Watches for clipboard changes via the Wayland wlr-data-control protocol
//! and stores them in a local `SQLite` database.

use std::io::Write;
use std::os::fd::AsFd;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use author_clipboard_shared::clipboard;
use author_clipboard_shared::config::Config;
use author_clipboard_shared::encryption::EncryptionManager;
use author_clipboard_shared::image_store;
use author_clipboard_shared::ipc::{
    remove_ipc_socket, CopyMode, IpcCommand, IpcMessage, IpcRequest, IpcResponse, IpcServer,
    IPC_VERSION,
};
use author_clipboard_shared::types::{AuditEventKind, ClipboardItem};
use author_clipboard_shared::Database;
use tracing::{debug, error, info, warn};
use wayland_client::protocol::wl_registry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{delegate_noop, Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
};

fn truncate_preview(content: &str, max_chars: usize) -> String {
    if content.chars().count() > max_chars {
        format!("{}...", content.chars().take(max_chars).collect::<String>())
    } else {
        content.to_string()
    }
}

/// Tracks MIME types offered by a clipboard data offer.
#[derive(Debug, Default)]
struct OfferMimeTypes {
    types: Vec<String>,
}

/// Application state for the Wayland event loop.
struct AppState {
    /// Bound wlr-data-control manager (clipboard protocol).
    manager: Option<ZwlrDataControlManagerV1>,
    /// Bound seat for clipboard device creation.
    seat: Option<WlSeat>,
    /// Active data control device.
    device: Option<ZwlrDataControlDeviceV1>,
    /// Currently pending clipboard offer with its advertised MIME types.
    pending_offer: Option<(ZwlrDataControlOfferV1, OfferMimeTypes)>,
    /// The most recently received clipboard text (for deduplication).
    last_content: Option<String>,
    /// Database for clipboard history persistence.
    db: Database,
    /// Application configuration.
    config: Config,
    /// Encryption manager for sensitive content at rest (shared via Arc).
    encryption_manager: Arc<Option<EncryptionManager>>,
    /// Monotonic snapshot revision shared with the IPC query boundary.
    revision: Arc<std::sync::atomic::AtomicU64>,
}

impl AppState {
    fn new(
        db: Database,
        config: Config,
        encryption_manager: Arc<Option<EncryptionManager>>,
        revision: Arc<std::sync::atomic::AtomicU64>,
    ) -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            pending_offer: None,
            last_content: None,
            db,
            config,
            encryption_manager,
            revision,
        }
    }

    /// Called when we have both manager and seat — creates the data device.
    fn try_create_device(&mut self, qh: &QueueHandle<Self>) {
        if let (Some(manager), Some(seat)) = (&self.manager, &self.seat) {
            if self.device.is_none() {
                let device = manager.get_data_device(seat, qh, ());
                info!("Created data control device");
                self.device = Some(device);
            }
        }
    }

    /// Insert a clipboard item, encrypting it at rest if sensitive and encryption is enabled.
    fn insert_item(&self, item: &ClipboardItem) -> anyhow::Result<i64> {
        let ignore_path = self.config.data_dir.join(".ignore-next-copy");
        if ignore_path.exists() && std::fs::remove_file(&ignore_path).is_ok() {
            info!("Consumed ignore-next-copy request");
            return Ok(0);
        }
        let mut ruled_item = item.clone();
        match author_clipboard_shared::rules::evaluate(
            &self.config.capture_rules,
            &item.content,
            &item.mime_type,
            item.source_app.as_deref(),
        ) {
            Some(author_clipboard_shared::rules::CaptureRuleAction::Ignore) => {
                info!("Clipboard capture ignored by configured rule");
                return Ok(0);
            }
            Some(author_clipboard_shared::rules::CaptureRuleAction::ForceSensitive) => {
                ruled_item.sensitive = true;
            }
            Some(author_clipboard_shared::rules::CaptureRuleAction::Tag { tag }) => {
                warn!(tag, "Capture-rule tags are not persisted by this schema");
            }
            None => {}
        }
        let item = &ruled_item;
        let result = if self.config.encrypt_sensitive && item.sensitive {
            if let Some(ref manager) = *self.encryption_manager {
                self.db
                    .insert_with_encryption(item, manager, true)
                    .map_err(|e| anyhow::anyhow!("Encryption insert failed: {e}"))
            } else {
                self.db
                    .insert_or_bump(item, self.config.dedup_window_seconds)
                    .map_err(|e| anyhow::anyhow!("DB insert failed: {e}"))
            }
        } else {
            self.db
                .insert_or_bump(item, self.config.dedup_window_seconds)
                .map_err(|e| anyhow::anyhow!("DB insert failed: {e}"))
        };
        if result.is_ok() {
            let revision = self
                .revision
                .fetch_add(1, std::sync::atomic::Ordering::Release);
            let _ = std::fs::write(
                self.config.data_dir.join(".history_revision"),
                revision.saturating_add(1).to_string(),
            );
        }
        result
    }

    /// Read raw bytes from a clipboard offer via a pipe.
    fn read_offer_bytes(
        offer: &ZwlrDataControlOfferV1,
        mime_type: &str,
        conn: &Connection,
    ) -> Result<Vec<u8>> {
        let (read_fd, write_fd) = rustix::pipe::pipe().context("Failed to create pipe")?;

        offer.receive(mime_type.to_string(), write_fd.as_fd());

        // Flush the Wayland connection so the compositor receives the
        // receive request before we try to read from the pipe.
        conn.flush().context("Failed to flush Wayland connection")?;

        // Close the write end so we get EOF after the compositor writes.
        drop(write_fd);

        let mut data = Vec::new();
        let mut file = std::fs::File::from(read_fd);
        std::io::Read::read_to_end(&mut file, &mut data)
            .context("Failed to read clipboard content from pipe")?;

        Ok(data)
    }

    /// Read text content from a clipboard offer via a pipe.
    fn read_offer_content(offer: &ZwlrDataControlOfferV1, conn: &Connection) -> Result<String> {
        let data = Self::read_offer_bytes(offer, "text/plain;charset=utf-8", conn)?;
        String::from_utf8(data).context("Clipboard content is not valid UTF-8")
    }
}

// ── Wayland dispatch implementations ────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "zwlr_data_control_manager_v1" => {
                    let manager = registry.bind::<ZwlrDataControlManagerV1, _, _>(
                        name,
                        version.min(2),
                        qh,
                        (),
                    );
                    info!("Bound wlr-data-control-manager v{version}");
                    state.manager = Some(manager);
                    state.try_create_device(qh);
                }
                "wl_seat" => {
                    let seat = registry.bind::<WlSeat, _, _>(name, version.min(7), qh, ());
                    info!("Bound wl_seat v{version}");
                    state.seat = Some(seat);
                    state.try_create_device(qh);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrDataControlManagerV1,
        _event: <ZwlrDataControlManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrDataControlDeviceV1, ()> for AppState {
    #[allow(clippy::too_many_lines, unused_variables)]
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlDeviceV1,
        event: zwlr_data_control_device_v1::Event,
        _data: &(),
        conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_device_v1::Event::DataOffer { id } => {
                debug!("New data offer received");
                state.pending_offer = Some((id, OfferMimeTypes::default()));
            }
            zwlr_data_control_device_v1::Event::Selection { id } => {
                // Check incognito mode - skip storing if active
                if state.config.is_incognito() {
                    debug!("🕶️  Incognito mode active, skipping clipboard storage");
                    state.pending_offer = None;
                    return;
                }

                if let Some(offer) = id {
                    let mime_types = state.pending_offer.as_ref().map(|(_, mimes)| &mimes.types);

                    // Check for image MIME types first (prefer image over text)
                    let image_mime = mime_types.and_then(|types| {
                        types
                            .iter()
                            .find(|t| image_store::is_image_mime(t))
                            .cloned()
                    });

                    let has_text = mime_types
                        .is_some_and(|types| types.iter().any(|t| t.starts_with("text/plain")));

                    let has_html =
                        mime_types.is_some_and(|types| types.iter().any(|t| t == "text/html"));

                    let has_uri_list =
                        mime_types.is_some_and(|types| types.iter().any(|t| t == "text/uri-list"));

                    if let Some(mime) = image_mime {
                        // Handle image clipboard
                        match Self::read_offer_bytes(&offer, &mime, conn) {
                            Ok(data) if data.is_empty() => {
                                debug!("Ignoring empty image clipboard");
                            }
                            Ok(data) if data.len() > state.config.max_item_size => {
                                debug!(
                                    "Ignoring oversized image ({} bytes, max {})",
                                    data.len(),
                                    state.config.max_item_size
                                );
                            }
                            Ok(data) => {
                                let hash = ClipboardItem::hash_bytes(&data);

                                match image_store::save_image(
                                    &state.config.data_dir,
                                    &data,
                                    &mime,
                                    hash,
                                ) {
                                    Ok(filename) => {
                                        let item = ClipboardItem::new_image(
                                            filename.clone(),
                                            mime.clone(),
                                            hash,
                                        );

                                        match state.insert_item(&item) {
                                            Ok(_) => info!(
                                                "🖼️  Stored image: {filename} ({} bytes, {mime})",
                                                data.len()
                                            ),
                                            Err(e) => warn!("DB insert failed for image: {e}"),
                                        }
                                    }
                                    Err(e) => warn!("Failed to save image: {e}"),
                                }
                            }
                            Err(e) => warn!("Failed to read image clipboard: {e}"),
                        }
                    } else if has_html {
                        // Handle HTML clipboard content
                        match Self::read_offer_bytes(&offer, "text/html", conn) {
                            Ok(html_data) if html_data.is_empty() => {
                                debug!("Ignoring empty HTML clipboard");
                            }
                            Ok(html_data) if html_data.len() > state.config.max_item_size => {
                                debug!(
                                    "Ignoring oversized HTML clipboard ({} bytes)",
                                    html_data.len()
                                );
                            }
                            Ok(html_data) => {
                                let html_content = String::from_utf8_lossy(&html_data).to_string();
                                // Also read plain text version for search indexing
                                let plain_text = if has_text {
                                    Self::read_offer_content(&offer, conn).unwrap_or_default()
                                } else {
                                    String::new()
                                };
                                let plain_text = plain_text.trim().to_string();

                                if state.last_content.as_deref() == Some(&html_content) {
                                    debug!("Ignoring duplicate HTML clipboard content");
                                } else if state.config.is_mime_denied("text/html")
                                    || state.config.is_content_denied(&html_content)
                                {
                                    debug!("Content blocked by denylist rules, skipping");
                                } else if state.config.is_app_denied(None) {
                                    // wlr-data-control does not currently expose
                                    // source-app info; this branch is
                                    // forward-compatible — see
                                    // specs/features/025-phase15-denylist/09-decisions.md
                                    debug!("Content blocked by app denylist, skipping");
                                } else {
                                    let preview = if plain_text.is_empty() {
                                        "HTML content".to_string()
                                    } else {
                                        truncate_preview(&plain_text, 80)
                                    };

                                    let item =
                                        ClipboardItem::new_html(html_content.clone(), plain_text);
                                    match state.insert_item(&item) {
                                        Ok(_) => info!("📄 Stored HTML: {preview}"),
                                        Err(e) => warn!("DB insert failed for HTML: {e}"),
                                    }
                                    if let Err(e) =
                                        state.db.enforce_max_items(state.config.max_items)
                                    {
                                        warn!("Cleanup failed: {e}");
                                    }
                                    state.last_content = Some(html_content);
                                }
                            }
                            Err(e) => warn!("Failed to read HTML clipboard: {e}"),
                        }
                    } else if has_uri_list {
                        // Handle file list clipboard content
                        match Self::read_offer_bytes(&offer, "text/uri-list", conn) {
                            Ok(data) if data.is_empty() => {
                                debug!("Ignoring empty file list clipboard");
                            }
                            Ok(data) => {
                                let file_list = String::from_utf8_lossy(&data).trim().to_string();
                                if file_list.is_empty() {
                                    debug!("Ignoring empty file list");
                                } else if state.last_content.as_deref() == Some(&file_list) {
                                    debug!("Ignoring duplicate file list clipboard");
                                } else if state.config.is_mime_denied("text/uri-list")
                                    || state.config.is_content_denied(&file_list)
                                {
                                    debug!("Content blocked by denylist rules, skipping");
                                } else if state.config.is_app_denied(None) {
                                    // See deviation note above; currently a no-op.
                                    debug!("Content blocked by app denylist, skipping");
                                } else {
                                    let file_count = file_list
                                        .lines()
                                        .filter(|l| !l.starts_with('#') && !l.is_empty())
                                        .count();

                                    let item = ClipboardItem::new_files(file_list.clone());
                                    match state.insert_item(&item) {
                                        Ok(_) => {
                                            info!("📁 Stored file list ({file_count} files)");
                                        }
                                        Err(e) => warn!("DB insert failed for file list: {e}"),
                                    }
                                    if let Err(e) =
                                        state.db.enforce_max_items(state.config.max_items)
                                    {
                                        warn!("Cleanup failed: {e}");
                                    }
                                    state.last_content = Some(file_list);
                                }
                            }
                            Err(e) => warn!("Failed to read file list clipboard: {e}"),
                        }
                    } else if has_text {
                        match Self::read_offer_content(&offer, conn) {
                            Ok(content) => {
                                let content = content.trim().to_string();
                                if content.is_empty() {
                                    debug!("Ignoring empty clipboard content");
                                } else if content.len() > state.config.max_item_size {
                                    debug!(
                                        "Ignoring oversized clipboard content ({} bytes)",
                                        content.len()
                                    );
                                } else if state.last_content.as_deref() == Some(&content) {
                                    debug!("Ignoring duplicate clipboard content");
                                } else if state.config.is_mime_denied("text/plain")
                                    || state.config.is_content_denied(&content)
                                {
                                    debug!("Content blocked by denylist rules, skipping");
                                } else if state.config.is_app_denied(None) {
                                    // See deviation note above; currently a no-op.
                                    debug!("Content blocked by app denylist, skipping");
                                } else {
                                    let preview = truncate_preview(&content, 80);

                                    let item = ClipboardItem::new_text(content.clone());

                                    match state.insert_item(&item) {
                                        Ok(_) => {
                                            if item.sensitive {
                                                info!(
                                                    "📋 Stored sensitive text item ({} bytes)",
                                                    content.len()
                                                );
                                                let _ = state.db.log_audit_event(
                                                    &AuditEventKind::SensitiveItemDetected,
                                                    Some(&format!(
                                                        "content_type=text; length={}; timestamp={}",
                                                        content.len(),
                                                        item.timestamp.to_rfc3339()
                                                    )),
                                                );
                                            } else {
                                                info!("📋 Stored: {preview}");
                                            }
                                        }
                                        Err(e) => warn!("DB insert failed: {e}"),
                                    }

                                    if let Err(e) =
                                        state.db.enforce_max_items(state.config.max_items)
                                    {
                                        warn!("Cleanup failed: {e}");
                                    }

                                    state.last_content = Some(content);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read clipboard: {e}");
                            }
                        }
                    } else {
                        debug!("Selection has no supported MIME type, skipping");
                    }

                    offer.destroy();
                    state.pending_offer = None;
                } else {
                    debug!("Clipboard cleared (no selection)");
                    state.pending_offer = None;
                }
            }
            zwlr_data_control_device_v1::Event::Finished => {
                warn!("Data control device finished — compositor may have restarted");
                state.device = None;
            }
            zwlr_data_control_device_v1::Event::PrimarySelection { .. } | _ => {}
        }
    }

    fn event_created_child(
        opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> Arc<dyn wayland_client::backend::ObjectData> {
        // Opcode 0 = data_offer event, which creates a ZwlrDataControlOfferV1
        if opcode == 0 {
            qhandle.make_data::<ZwlrDataControlOfferV1, _>(())
        } else {
            panic!("unknown opcode for event_created_child: {opcode}");
        }
    }
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlOfferV1,
        event: zwlr_data_control_offer_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event {
            debug!("Offer MIME type: {mime_type}");
            if let Some((_, ref mut mimes)) = state.pending_offer {
                mimes.types.push(mime_type);
            }
        }
    }
}

// WlSeat events not needed — just need the object for get_data_device.
delegate_noop!(AppState: ignore WlSeat);

/// Check if the screen is currently locked via loginctl.
fn is_screen_locked() -> bool {
    let output = std::process::Command::new("loginctl")
        .args(["show-session", "auto", "--property=LockedHint", "--value"])
        .output();

    match output {
        Ok(out) => {
            let value = String::from_utf8_lossy(&out.stdout);
            value.trim() == "yes"
        }
        Err(_) => false,
    }
}

/// A subscription for live update notifications.
#[derive(Debug)]
struct Subscription {
    /// Unique subscription ID.
    id: u64,
    /// Events to subscribe to.
    events: Vec<String>,
    /// Channel to send events to.
    sender: std::sync::mpsc::Sender<IpcResponse>,
}

/// Shared state for IPC command handling.
#[derive(Clone)]
#[allow(dead_code)]
struct IpcHandlerState {
    db: Arc<Mutex<Database>>,
    config: Config,
    data_dir: std::path::PathBuf,
    visibility_path: std::path::PathBuf,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,
    next_sub_id: Arc<Mutex<u64>>,
    encryption_manager: Arc<Option<EncryptionManager>>,
    revision: Arc<std::sync::atomic::AtomicU64>,
}

impl IpcHandlerState {
    fn new(
        db: Database,
        config: Config,
        data_dir: std::path::PathBuf,
        encryption_manager: Arc<Option<EncryptionManager>>,
        revision: Arc<std::sync::atomic::AtomicU64>,
    ) -> Self {
        let visibility_path = data_dir.join(".visibility_toggle");
        Self {
            db: Arc::new(Mutex::new(db)),
            config,
            data_dir,
            visibility_path,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            next_sub_id: Arc::new(Mutex::new(1)),
            encryption_manager,
            revision,
        }
    }

    /// Broadcast an event to all subscribers.
    fn broadcast(&self, event: &str, data: &serde_json::Value) {
        let response = IpcResponse::ok(serde_json::json!({
            "type": "event",
            "event": event,
            "data": data,
        }));

        let mut subs = self.subscriptions.lock().unwrap();
        subs.retain(|sub| {
            if sub.events.iter().any(|e| e == event || e == "*") {
                if let Err(e) = sub.sender.send(response.clone()) {
                    debug!("Failed to send event to subscription {}: {}", sub.id, e);
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });
    }

    /// Handle an IPC request and return the response.
    fn handle_request(&self, request: &IpcRequest) -> IpcResponse {
        // Check version compatibility
        if request.version != IPC_VERSION {
            return IpcResponse::err_with_min_version(
                "UNSUPPORTED_VERSION",
                format!("Protocol version {} is not supported", request.version),
                IPC_VERSION,
            );
        }

        // Parse and handle the command
        let cmd_result = serde_json::from_value::<IpcCommand>(request.args.clone());
        let cmd = match cmd_result {
            Ok(cmd) => cmd,
            Err(e) => {
                return IpcResponse::err(
                    "INVALID_COMMAND",
                    format!("Failed to parse command: {e}"),
                );
            }
        };

        self.handle_command(cmd)
    }

    #[allow(clippy::too_many_lines, unused_variables)]
    fn handle_command(&self, cmd: IpcCommand) -> IpcResponse {
        match cmd {
            // ── Visibility ────────────────────────────────────────────────
            IpcCommand::Toggle => {
                if let Err(e) = std::fs::write(&self.visibility_path, "toggle") {
                    return IpcResponse::err("IO_ERROR", format!("Failed to write signal: {e}"));
                }
                IpcResponse::ok(serde_json::json!({"visible": true}))
            }
            IpcCommand::Show => {
                if let Err(e) = std::fs::write(&self.visibility_path, "show") {
                    return IpcResponse::err("IO_ERROR", format!("Failed to write signal: {e}"));
                }
                IpcResponse::ok(serde_json::json!({"visible": true}))
            }
            IpcCommand::Hide => {
                if let Err(e) = std::fs::write(&self.visibility_path, "hide") {
                    return IpcResponse::err("IO_ERROR", format!("Failed to write signal: {e}"));
                }
                IpcResponse::ok(serde_json::json!({"visible": false}))
            }
            IpcCommand::ShowAt { x, y } => {
                if let Err(e) = std::fs::write(&self.visibility_path, format!("show_at:{x}:{y}")) {
                    return IpcResponse::err("IO_ERROR", format!("Failed to write signal: {e}"));
                }
                IpcResponse::ok(serde_json::json!({"visible": true, "x": x, "y": y}))
            }

            // ── Health ────────────────────────────────────────────────────
            IpcCommand::Ping => IpcResponse::ok(serde_json::json!({
                "status": "ok",
                "daemon_pid": std::process::id(),
            })),
            IpcCommand::Status => {
                let db = self.db.lock().unwrap();
                let stats = match db.get_stats() {
                    Ok(s) => s,
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to get stats: {e}"))
                    }
                };
                let incognito = self.config.is_incognito();
                IpcResponse::ok(serde_json::json!({
                    "daemon_version": env!("CARGO_PKG_VERSION"),
                    "daemon_pid": std::process::id(),
                    "visible": false,
                    "item_count": stats.total_items,
                    "pinned_count": stats.pinned_items,
                    "incognito": incognito,
                    "database_size_bytes": stats.total_size_bytes,
                    "capture_active": true,
                }))
            }

            // ── Query ────────────────────────────────────────────────────
            IpcCommand::History {
                limit,
                offset,
                filters,
            } => {
                let db = self.db.lock().unwrap();
                let items = match db.get_recent(limit) {
                    Ok(items) => items,
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to get history: {e}"))
                    }
                };

                // Apply filters
                let filtered: Vec<_> = items
                    .into_iter()
                    .filter(|item| {
                        if let Some(ref filters) = filters {
                            if let Some(ref content_types) = filters.content_type {
                                if !content_types
                                    .iter()
                                    .any(|ct| ct == item.content_type.as_str())
                                {
                                    return false;
                                }
                            }
                            if let Some(pinned) = filters.pinned {
                                if item.pinned != pinned {
                                    return false;
                                }
                            }
                            if let Some(sensitive) = filters.sensitive {
                                if item.sensitive != sensitive {
                                    return false;
                                }
                            }
                            if let Some(ref source_app) = filters.source_app {
                                if item.source_app.as_ref() != Some(source_app) {
                                    return false;
                                }
                            }
                        }
                        true
                    })
                    .collect();

                let total = filtered.len();
                let offset = offset.unwrap_or(0);
                let items: Vec<_> = filtered
                    .into_iter()
                    .skip(offset)
                    .take(limit)
                    .map(|item| self.item_to_json(&item))
                    .collect();

                IpcResponse::ok(serde_json::json!({
                    "items": items,
                    "total": total,
                    "offset": offset,
                    "limit": limit,
                    "has_more": offset + items.len() < total,
                    "revision": self.revision.load(std::sync::atomic::Ordering::Acquire),
                }))
            }
            IpcCommand::GetItem { id } => {
                let db = self.db.lock().unwrap();
                let item = match db.get_by_id(id) {
                    Ok(Some(item)) => item,
                    Ok(None) => {
                        return IpcResponse::err("NOT_FOUND", format!("Item {id} not found"))
                    }
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to get item: {e}"))
                    }
                };
                IpcResponse::ok(self.item_to_json(&item))
            }
            IpcCommand::Search {
                query,
                limit,
                filters,
            } => {
                let db = self.db.lock().unwrap();
                let items = match db.search(&query, limit.unwrap_or(50)) {
                    Ok(items) => items,
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to search: {e}"))
                    }
                };

                let total = items.len();
                let items: Vec<_> = items
                    .into_iter()
                    .map(|item| self.item_to_json(&item))
                    .collect();

                IpcResponse::ok(serde_json::json!({
                    "items": items,
                    "total": total,
                    "offset": 0,
                    "limit": limit.unwrap_or(50),
                    "has_more": false,
                    "revision": self.revision.load(std::sync::atomic::Ordering::Acquire),
                }))
            }
            IpcCommand::GetStats => {
                let db = self.db.lock().unwrap();
                let stats = match db.get_stats() {
                    Ok(s) => s,
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to get stats: {e}"))
                    }
                };
                IpcResponse::ok(serde_json::json!({
                    "total_items": stats.total_items,
                    "pinned_items": stats.pinned_items,
                    "total_size_bytes": stats.total_size_bytes,
                    "oldest_item": null,
                    "newest_item": null,
                    "capture_rate_per_hour": null,
                }))
            }
            IpcCommand::GetAuditLog { limit } => {
                let db = self.db.lock().unwrap();
                let events = match db.get_audit_log(limit.unwrap_or(100)) {
                    Ok(e) => e,
                    Err(e) => {
                        return IpcResponse::err(
                            "DB_ERROR",
                            format!("Failed to get audit log: {e}"),
                        )
                    }
                };
                let events_json: Vec<_> = events
                    .into_iter()
                    .map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "event_kind": e.event_kind,
                            "details": e.details,
                            "timestamp": e.timestamp.to_rfc3339(),
                        })
                    })
                    .collect();
                IpcResponse::ok(serde_json::json!({ "events": events_json }))
            }

            // ── Mutations ────────────────────────────────────────────────
            IpcCommand::Copy { id, mode, mime } => {
                let db = self.db.lock().unwrap();
                let item = match db.get_by_id(id) {
                    Ok(Some(item)) => item,
                    Ok(None) => {
                        return IpcResponse::err("NOT_FOUND", format!("Item {id} not found"))
                    }
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to get item: {e}"))
                    }
                };

                // Decrypt item if encrypted (for sensitive items stored with encryption)
                let item = if item.encrypted {
                    if let Some(ref manager) = *self.encryption_manager {
                        match db.decrypt_item(&item, manager) {
                            Ok(decrypted) => decrypted,
                            Err(e) => {
                                return IpcResponse::err(
                                    "DECRYPT_ERROR",
                                    format!("Failed to decrypt item: {e}"),
                                )
                            }
                        }
                    } else {
                        // Encryption manager not available but item is encrypted - this shouldn't happen
                        return IpcResponse::err(
                            "DECRYPT_ERROR",
                            "Item is encrypted but encryption manager is not available".to_string(),
                        );
                    }
                } else {
                    item
                };
                drop(db);

                let result = match mode {
                    CopyMode::Copy | CopyMode::QuickPaste => {
                        clipboard::set_clipboard_item(&item, &self.data_dir)
                    }
                    CopyMode::CopyPlainText => {
                        let mut plain = item.clone();
                        plain.content = item.plain_text.clone().unwrap_or_default();
                        clipboard::set_clipboard_item(&plain, &self.data_dir)
                    }
                    CopyMode::CopyRedacted => {
                        let mut redacted = item.clone();
                        if item.sensitive {
                            redacted.content = "••••••••".to_string();
                        }
                        clipboard::set_clipboard_item(&redacted, &self.data_dir)
                    }
                };

                match result {
                    Ok(result) => {
                        // Log audit event
                        if let Ok(db) = self.db.lock() {
                            let _ = db.log_audit_event(
                                &AuditEventKind::ItemDeleted,
                                Some(&format!("id={}; sensitive={}", id, item.sensitive)),
                            );
                        }
                        IpcResponse::ok(serde_json::json!({
                            "id": id,
                            "mime_type": mime.unwrap_or(result.mime_type),
                            "behavior": result.behavior,
                            "sensitive_confirmed": false,
                        }))
                    }
                    Err(e) => IpcResponse::err("COPY_ERROR", format!("Failed to copy: {e}")),
                }
            }
            IpcCommand::Transform {
                content,
                transform,
                sensitive,
                confirm_sensitive,
            } => {
                match author_clipboard_shared::transform::apply(
                    &content,
                    &transform,
                    sensitive,
                    confirm_sensitive,
                ) {
                    Ok(output) => IpcResponse::ok(serde_json::json!({ "output": output })),
                    Err(error) => IpcResponse::err("transform_failed", error.to_string()),
                }
            }
            IpcCommand::Pin { id } => {
                let db = self.db.lock().unwrap();
                if let Err(e) = db.set_pinned(id, true) {
                    return IpcResponse::err("DB_ERROR", format!("Failed to pin item: {e}"));
                }
                self.broadcast("PinToggled", &serde_json::json!({"id": id, "pinned": true}));
                IpcResponse::ok(serde_json::json!({"id": id, "pinned": true}))
            }
            IpcCommand::Unpin { id } => {
                let db = self.db.lock().unwrap();
                if let Err(e) = db.set_pinned(id, false) {
                    return IpcResponse::err("DB_ERROR", format!("Failed to unpin item: {e}"));
                }
                self.broadcast(
                    "PinToggled",
                    &serde_json::json!({"id": id, "pinned": false}),
                );
                IpcResponse::ok(serde_json::json!({"id": id, "pinned": false}))
            }
            IpcCommand::Delete { id } => {
                let db = self.db.lock().unwrap();
                if let Err(e) = db.delete_item(id) {
                    return IpcResponse::err("DB_ERROR", format!("Failed to delete item: {e}"));
                }
                self.broadcast("ItemDeleted", &serde_json::json!({"id": id}));
                IpcResponse::ok(serde_json::json!({"deleted_id": id}))
            }
            IpcCommand::ClearUnpinned => {
                let db = self.db.lock().unwrap();
                let count = match db.clear_unpinned() {
                    Ok(c) => c,
                    Err(e) => return IpcResponse::err("DB_ERROR", format!("Failed to clear: {e}")),
                };
                let _ = db.log_audit_event(
                    &AuditEventKind::HistoryCleared,
                    Some(&format!("deleted_count={count}")),
                );
                drop(db);
                self.broadcast(
                    "HistoryCleared",
                    &serde_json::json!({"deleted_count": count}),
                );
                IpcResponse::ok(serde_json::json!({"deleted_count": count}))
            }
            IpcCommand::ClearAll => {
                let db = self.db.lock().unwrap();
                let count = match db.clear_all() {
                    Ok(c) => c,
                    Err(e) => return IpcResponse::err("DB_ERROR", format!("Failed to clear: {e}")),
                };
                let _ = db.log_audit_event(
                    &AuditEventKind::HistoryCleared,
                    Some(&format!("deleted_count={count}; include_pinned=true")),
                );
                drop(db);
                self.broadcast(
                    "HistoryCleared",
                    &serde_json::json!({"deleted_count": count, "include_pinned": true}),
                );
                IpcResponse::ok(serde_json::json!({"deleted_count": count}))
            }
            IpcCommand::IgnoreNextCopy => {
                let path = self.config.data_dir.join(".ignore-next-copy");
                match std::fs::write(path, "armed") {
                    Ok(()) => IpcResponse::ok(serde_json::json!({ "armed": true })),
                    Err(error) => IpcResponse::err(
                        "IO_ERROR",
                        format!("Failed to arm ignore-next-copy: {error}"),
                    ),
                }
            }

            // ── Snippets ──────────────────────────────────────────────────
            IpcCommand::ListSnippets => {
                let db = self.db.lock().unwrap();
                let snippets = match db.list_snippets() {
                    Ok(s) => s,
                    Err(e) => {
                        return IpcResponse::err(
                            "DB_ERROR",
                            format!("Failed to list snippets: {e}"),
                        )
                    }
                };
                let snippets_json: Vec<_> = snippets
                    .into_iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "name": s.name,
                            "content": s.content,
                            "updated_at": s.updated_at.to_rfc3339(),
                        })
                    })
                    .collect();
                IpcResponse::ok(serde_json::json!({ "snippets": snippets_json }))
            }
            IpcCommand::UpsertSnippet { name, content } => {
                let db = self.db.lock().unwrap();
                if let Err(e) = db.upsert_snippet(&name, &content) {
                    return IpcResponse::err("DB_ERROR", format!("Failed to upsert snippet: {e}"));
                }
                let snippets = db.list_snippets().ok();
                drop(db);
                if let Some(snippets) = snippets {
                    if let Some(snippet) = snippets.iter().find(|s| s.name == name) {
                        self.broadcast(
                            "SnippetUpdated",
                            &serde_json::json!({
                                "id": snippet.id,
                                "name": snippet.name,
                            }),
                        );
                        return IpcResponse::ok(serde_json::json!({
                            "id": snippet.id,
                            "name": snippet.name,
                            "content": snippet.content,
                            "updated_at": snippet.updated_at.to_rfc3339(),
                        }));
                    }
                }
                IpcResponse::ok(serde_json::json!({ "name": name }))
            }
            IpcCommand::DeleteSnippet { id } => {
                let db = self.db.lock().unwrap();
                if let Err(e) = db.delete_snippet(id) {
                    return IpcResponse::err("DB_ERROR", format!("Failed to delete snippet: {e}"));
                }
                self.broadcast("SnippetDeleted", &serde_json::json!({"id": id}));
                IpcResponse::ok(serde_json::json!({"deleted_id": id}))
            }
            IpcCommand::RenderSnippet { id } => {
                let snippet = {
                    let db = self.db.lock().unwrap();
                    match db.get_snippet(id) {
                        Ok(Some(s)) => s,
                        Ok(None) => {
                            return IpcResponse::err(
                                "SNIPPET_NOT_FOUND",
                                format!("No snippet with id={id}"),
                            );
                        }
                        Err(e) => {
                            return IpcResponse::err(
                                "DB_ERROR",
                                format!("Failed to load snippet {id}: {e}"),
                            );
                        }
                    }
                };
                // `${clipboard}` is not populated here because the IPC
                // handler doesn't share state with the Wayland capture
                // loop. Picker preview uses `render_now` (no clipboard)
                // anyway. A follow-up can plumb `last_content` through
                // a shared Arc when needed — see
                // specs/features/026-snippet-templates/09-decisions.md.
                let ctx = author_clipboard_shared::template::RenderContext {
                    now: None, // render() falls back to Utc::now()
                    clipboard: None,
                    user: std::env::var("USER")
                        .ok()
                        .or_else(|| std::env::var("LOGNAME").ok()),
                    hostname: std::env::var("HOSTNAME").ok(),
                };
                let (rendered, cursor_offset) =
                    match author_clipboard_shared::snippet_template::expand(
                        &snippet.content,
                        &ctx,
                        None,
                        false,
                        false,
                    ) {
                        Ok(result) => result,
                        Err(error) => {
                            return IpcResponse::err("SNIPPET_TEMPLATE_INVALID", error.to_string())
                        }
                    };
                IpcResponse::ok(serde_json::json!({
                    "content": rendered,
                    "cursor_offset": cursor_offset,
                }))
            }

            // ── Collections ──────────────────────────────────────────────
            IpcCommand::ListCollections => {
                let db = self.db.lock().unwrap();
                let collections = match db.list_collections() {
                    Ok(c) => c,
                    Err(e) => {
                        return IpcResponse::err(
                            "DB_ERROR",
                            format!("Failed to list collections: {e}"),
                        );
                    }
                };
                let collections_json: Vec<_> = collections
                    .into_iter()
                    .map(|c| {
                        serde_json::json!({
                            "id": c.id,
                            "name": c.name,
                            "created_at": c.created_at.to_rfc3339(),
                            "updated_at": c.updated_at.to_rfc3339(),
                        })
                    })
                    .collect();
                IpcResponse::ok(serde_json::json!({ "collections": collections_json }))
            }
            IpcCommand::CreateCollection { name } => {
                let db = self.db.lock().unwrap();
                match db.create_collection(&name) {
                    Ok(id) => {
                        self.broadcast(
                            "CollectionCreated",
                            &serde_json::json!({"id": id, "name": name}),
                        );
                        IpcResponse::ok(serde_json::json!({"id": id, "name": name}))
                    }
                    Err(e) => {
                        IpcResponse::err("DB_ERROR", format!("Failed to create collection: {e}"))
                    }
                }
            }
            IpcCommand::DeleteCollection { id } => {
                let db = self.db.lock().unwrap();
                match db.delete_collection(&id) {
                    Ok(()) => {
                        self.broadcast("CollectionDeleted", &serde_json::json!({"id": id}));
                        IpcResponse::ok(serde_json::json!({"deleted_id": id}))
                    }
                    Err(e) => {
                        IpcResponse::err("DB_ERROR", format!("Failed to delete collection: {e}"))
                    }
                }
            }
            IpcCommand::RenameCollection { id, new_name } => {
                let db = self.db.lock().unwrap();
                match db.rename_collection(&id, &new_name) {
                    Ok(()) => IpcResponse::ok(serde_json::json!({"id": id, "name": new_name})),
                    Err(e) => {
                        IpcResponse::err("DB_ERROR", format!("Failed to rename collection: {e}"))
                    }
                }
            }
            IpcCommand::GetCollectionItems { id } => {
                let db = self.db.lock().unwrap();
                match db.get_collection_items(&id) {
                    Ok(items) => {
                        let items_json: Vec<_> =
                            items.iter().map(|item| self.item_to_json(item)).collect();
                        IpcResponse::ok(serde_json::json!({ "items": items_json }))
                    }
                    Err(e) => {
                        IpcResponse::err("DB_ERROR", format!("Failed to get collection items: {e}"))
                    }
                }
            }
            IpcCommand::AddToCollection {
                collection_id,
                item_id,
            } => {
                let db = self.db.lock().unwrap();
                match db.add_to_collection(&collection_id, item_id) {
                    Ok(()) => IpcResponse::ok(
                        serde_json::json!({"collection_id": collection_id, "item_id": item_id}),
                    ),
                    Err(e) => {
                        IpcResponse::err("DB_ERROR", format!("Failed to add to collection: {e}"))
                    }
                }
            }
            IpcCommand::RemoveFromCollection {
                collection_id,
                item_id,
            } => {
                let db = self.db.lock().unwrap();
                match db.remove_from_collection(&collection_id, item_id) {
                    Ok(()) => IpcResponse::ok(
                        serde_json::json!({"collection_id": collection_id, "item_id": item_id}),
                    ),
                    Err(e) => IpcResponse::err(
                        "DB_ERROR",
                        format!("Failed to remove from collection: {e}"),
                    ),
                }
            }

            // ── Config ────────────────────────────────────────────────────
            IpcCommand::GetConfig => IpcResponse::ok(serde_json::json!({
                "max_items": self.config.max_items,
                "max_item_size": self.config.max_item_size,
                "ttl_seconds": self.config.ttl_seconds,
                "cleanup_interval_seconds": self.config.cleanup_interval_seconds,
                "keyboard_shortcut": self.config.keyboard_shortcut,
                "encrypt_sensitive": self.config.encrypt_sensitive,
                "clear_on_lock": self.config.clear_on_lock,
                "dedup_window_seconds": self.config.dedup_window_seconds,
                "mime_denylist": self.config.mime_denylist,
                "content_denylist": self.config.content_denylist,
                "content_pattern_mode": self.config.content_pattern_mode,
                "app_denylist": self.config.app_denylist,
                "picker": {
                    "default_mode": self.config.picker.default_mode,
                    "default_source": self.config.picker.default_source,
                    "max_results": self.config.picker.max_results,
                    "show_sensitive_previews": self.config.picker.show_sensitive_previews,
                    "confirm_sensitive_copy": self.config.picker.confirm_sensitive_copy,
                    "close_after_copy": self.config.picker.close_after_copy,
                    "prefer_quick_paste": self.config.picker.prefer_quick_paste,
                    "width": self.config.picker.width,
                    "height": self.config.picker.height,
                },
            })),
            IpcCommand::UpdateConfig { config } => {
                // For now, just acknowledge - full config update would require persisting
                let updated_keys =
                    serde_json::from_value::<Vec<String>>(config.clone()).unwrap_or_default();
                IpcResponse::ok(serde_json::json!({ "updated_keys": updated_keys }))
            }
            IpcCommand::ToggleStar { id } => {
                let db = self.db.lock().unwrap();
                let starred = match db.toggle_star(id) {
                    Ok(s) => s,
                    Err(e) => {
                        return IpcResponse::err("DB_ERROR", format!("Failed to toggle star: {e}"))
                    }
                };
                self.broadcast(
                    "StarToggled",
                    &serde_json::json!({"id": id, "starred": starred}),
                );
                IpcResponse::ok(serde_json::json!({"id": id, "starred": starred}))
            }
        }
    }

    /// Convert a clipboard item to JSON, applying sensitivity masking.
    /// For encrypted items, uses the pre-computed `redacted_preview` to avoid
    /// leaking ciphertext in the UI.
    fn item_to_json(&self, item: &ClipboardItem) -> serde_json::Value {
        let show_sensitive = self.config.picker.show_sensitive_previews;

        // For encrypted items, use the pre-computed redacted_preview.
        // This avoids ever passing ciphertext to the UI.
        let (content, plain_text, preview, encrypted) =
            if item.encrypted || (item.sensitive && !show_sensitive) {
                // Use redacted preview for encrypted or sensitive items
                let redacted = if item.encrypted {
                    item.redacted_preview
                        .clone()
                        .unwrap_or_else(|| "••••••••".to_string())
                } else {
                    "••••••••".to_string()
                };
                (redacted.clone(), redacted.clone(), redacted, item.encrypted)
            } else {
                (
                    item.content.clone(),
                    item.plain_text.clone().unwrap_or_default(),
                    truncate_preview(&item.content, 80),
                    item.encrypted,
                )
            };

        serde_json::json!({
            "id": item.id,
            "content_hash": format!("{:016x}", item.content_hash),
            "content": content,
            "mime_type": item.mime_type,
            "content_type": item.content_type.as_str(),
            "timestamp": item.timestamp.to_rfc3339(),
            "pinned": item.pinned,
            "starred": item.starred,
            "source_app": item.source_app,
            "sensitive": item.sensitive,
            "encrypted": encrypted,
            "plain_text": plain_text,
            "preview": preview,
        })
    }
}

/// Spawn a background thread running the IPC server that listens for
/// toggle/show/hide commands and writes a visibility signal file for the applet.
fn spawn_ipc_server(state: IpcHandlerState) {
    std::thread::spawn(move || {
        let server = match IpcServer::bind() {
            Ok(s) => {
                info!(
                    "🔌 IPC server listening at {}",
                    author_clipboard_shared::ipc::socket_path().display()
                );
                s
            }
            Err(e) => {
                warn!("Failed to start IPC server: {e}");
                return;
            }
        };

        loop {
            match server.accept_stream() {
                Ok((mut stream, msg)) => {
                    let request_id = match &msg {
                        IpcMessage::Request(request) => request.request_id,
                        _ => None,
                    };
                    let response = match &msg {
                        // Handle legacy messages
                        IpcMessage::Toggle | IpcMessage::Show | IpcMessage::Hide => {
                            info!("🎯 IPC received: {msg:?}");
                            let signal = match msg {
                                IpcMessage::Toggle => "toggle",
                                IpcMessage::Show => "show",
                                IpcMessage::Hide => "hide",
                                _ => continue,
                            };
                            if let Err(e) = std::fs::write(&state.visibility_path, signal) {
                                warn!("Failed to write visibility signal: {e}");
                            }
                            Some(IpcMessage::Status {
                                visible: true,
                                item_count: 0,
                            })
                        }
                        IpcMessage::ShowAt { x, y } => {
                            info!("🎯 IPC ShowAt: x={x}, y={y}");
                            if let Err(e) =
                                std::fs::write(&state.visibility_path, format!("show_at:{x}:{y}"))
                            {
                                warn!("Failed to write visibility signal: {e}");
                            }
                            Some(IpcMessage::Status {
                                visible: true,
                                item_count: 0,
                            })
                        }
                        IpcMessage::Ping => {
                            debug!("IPC ping received");
                            Some(IpcMessage::Pong)
                        }
                        // Handle versioned requests
                        IpcMessage::Request(request) => {
                            debug!("IPC request: cmd={}", request.cmd);
                            let response = state.handle_request(request);
                            let mut response = response;
                            response.request_id = request_id;
                            Some(IpcMessage::Response(response))
                        }
                        _ => {
                            debug!("IPC message: {msg:?}");
                            None
                        }
                    };

                    if let Some(resp) = response {
                        match serde_json::to_string(&resp) {
                            Ok(json) => {
                                if let Err(error) =
                                    writeln!(stream, "{json}").and_then(|()| stream.flush())
                                {
                                    debug!(?error, "failed to write IPC response");
                                }
                            }
                            Err(error) => debug!(?error, "failed to serialize IPC response"),
                        }
                    }
                }
                Err(e) => {
                    debug!("IPC accept error (may be transient): {e}");
                }
            }
        }
    });
}

/// Removes the daemon PID file when dropped, ensuring cleanup on exit or panic.
struct PidFileGuard(std::path::PathBuf);
impl Drop for PidFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[allow(clippy::too_many_lines)]
fn run() -> Result<()> {
    Config::save_default_if_missing().context("Failed to initialize config file")?;
    let config = Config::load();

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)
        .with_context(|| format!("Failed to create data dir: {}", config.data_dir.display()))?;

    // Single-instance lock: prevent two daemon processes writing to the same DB.
    let pid_file_path = config.data_dir.join("daemon.pid");
    if let Ok(existing_pid) = std::fs::read_to_string(&pid_file_path) {
        let existing_pid = existing_pid.trim().to_string();
        // Check if that process is actually running
        let proc_path = format!("/proc/{existing_pid}");
        if std::path::Path::new(&proc_path).exists() {
            anyhow::bail!(
                "Another daemon instance is already running (PID {existing_pid}). \
                 Stop it first with: kill {existing_pid}"
            );
        }
        // Stale PID file — previous daemon crashed; remove it and continue.
        let _ = std::fs::remove_file(&pid_file_path);
    }
    let current_pid = std::process::id();
    std::fs::write(&pid_file_path, current_pid.to_string()).context("Failed to write PID file")?;
    let _pid_guard = PidFileGuard(pid_file_path);

    let db = Database::open(&config.db_path()).context("Failed to open clipboard database")?;
    info!("Database opened at {}", config.db_path().display());

    // Ensure image storage directories exist
    image_store::ensure_dirs(&config.data_dir)
        .context("Failed to create image storage directories")?;

    // Spawn screen lock monitor thread
    let lock_db_path = config.db_path();
    let clear_on_lock = config.clear_on_lock;
    std::thread::spawn(move || {
        let mut was_locked = false;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));

            if !clear_on_lock {
                continue;
            }

            let locked = is_screen_locked();
            if locked && !was_locked {
                info!("🔒 Screen locked — clearing sensitive items");
                if let Ok(lock_db) = Database::open(&lock_db_path) {
                    match lock_db.clear_sensitive() {
                        Ok(count) if count > 0 => {
                            info!("Cleared {count} sensitive items on lock");
                        }
                        Ok(_) => debug!("No sensitive items to clear"),
                        Err(e) => warn!("Failed to clear sensitive items: {e}"),
                    }
                }
            }
            was_locked = locked;
        }
    });

    // Spawn IPC server thread for shortcut activation (with separate DB connection)
    let ipc_db = Database::open(&config.db_path()).context("Failed to open IPC database")?;

    // Initialize encryption manager if encrypt_sensitive is enabled.
    // If initialization fails, we continue without encryption (items stored as plaintext).
    let encryption_manager: Arc<Option<EncryptionManager>> = if config.encrypt_sensitive {
        match EncryptionManager::new(&config.data_dir) {
            Ok(mgr) => {
                info!("Encryption manager initialized (sensitive items will be encrypted at rest)");
                Arc::new(Some(mgr))
            }
            Err(e) => {
                warn!("Failed to initialize encryption manager: {}. Sensitive items will be stored as plaintext.", e);
                Arc::new(None)
            }
        }
    } else {
        Arc::new(None)
    };

    let revision = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let ipc_state = IpcHandlerState::new(
        ipc_db,
        config.clone(),
        config.data_dir.clone(),
        Arc::clone(&encryption_manager),
        Arc::clone(&revision),
    );
    spawn_ipc_server(ipc_state);

    let conn = Connection::connect_to_env().context(
        "Failed to connect to Wayland display. \
         Ensure you are running on a Wayland compositor (e.g. COSMIC).",
    )?;

    let display = conn.display();

    let mut event_queue: EventQueue<AppState> = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut state = AppState::new(db, config, Arc::clone(&encryption_manager), revision);

    // Trigger global advertisement
    display.get_registry(&qh, ());

    // Initial roundtrip to receive globals
    event_queue
        .roundtrip(&mut state)
        .context("Initial Wayland roundtrip failed")?;

    if state.manager.is_none() {
        anyhow::bail!(
            "Compositor does not support wlr-data-control-unstable-v1. \
             On COSMIC, set COSMIC_DATA_CONTROL_ENABLED=1. \
             On Hyprland/Sway, check compositor support and daemon logs."
        );
    }

    if state.device.is_none() {
        anyhow::bail!("No seat found — cannot create data control device.");
    }

    info!("Clipboard monitoring active. Copy text anywhere to see it here.");

    // Main event loop
    loop {
        if let Err(e) = event_queue.blocking_dispatch(&mut state) {
            info!("Daemon shutting down...");
            remove_ipc_socket();
            return Err(e).context("Wayland event dispatch failed");
        }
    }
}

fn main() {
    // Handle --help and --version before tracing init
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("author-clipboard-daemon - Clipboard monitoring daemon");
                println!();
                println!("USAGE: author-clipboard-daemon [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  -h, --help       Print this help message");
                println!("  -V, --version    Print version information");
                println!();
                println!("ENVIRONMENT:");
                println!("  RUST_LOG    Set log level (default: info)");
                return;
            }
            "--version" | "-V" => {
                println!("author-clipboard-daemon {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Run with --help for usage information.");
                std::process::exit(1);
            }
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Check display server compatibility before doing anything else.
    {
        use author_clipboard_shared::compositor::{
            detect_display_server, get_compositor_help, DisplayServer,
        };
        let server = detect_display_server();
        if let Some(help) = get_compositor_help(&server) {
            if matches!(server, DisplayServer::X11 | DisplayServer::Unknown) {
                eprintln!("Error: Unsupported display server configuration\n\n{help}");
                std::process::exit(1);
            }
            // For COSMIC env warnings, try anyway; registry binding is the real protocol check.
            eprintln!("Warning: {help}");
        }
    }

    info!("author-clipboard-daemon starting...");

    match run() {
        Ok(()) => {
            info!("Daemon stopped cleanly.");
        }
        Err(e) => {
            error!("Fatal error: {e:#}");
            remove_ipc_socket();
            std::process::exit(1);
        }
    }
}
