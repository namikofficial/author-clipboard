//! author-clipboard-daemon: Background clipboard monitoring daemon
//!
//! Watches for clipboard changes via the Wayland wlr-data-control protocol
//! and stores them in a local `SQLite` database.

use std::os::fd::AsFd;
use std::sync::Arc;

use anyhow::{Context, Result};
use author_clipboard_shared::config::Config;
use author_clipboard_shared::image_store;
use author_clipboard_shared::ipc::{IpcMessage, IpcServer};
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
}

impl AppState {
    fn new(db: Database, config: Config) -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            pending_offer: None,
            last_content: None,
            db,
            config,
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
    #[allow(clippy::too_many_lines)]
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

                                        match state.db.insert_or_bump(&item) {
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
                                } else {
                                    let preview = if plain_text.len() > 80 {
                                        format!("{}...", &plain_text[..80])
                                    } else if plain_text.is_empty() {
                                        "HTML content".to_string()
                                    } else {
                                        plain_text.clone()
                                    };

                                    let item =
                                        ClipboardItem::new_html(html_content.clone(), plain_text);
                                    match state.db.insert_or_bump(&item) {
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
                                } else {
                                    let file_count = file_list
                                        .lines()
                                        .filter(|l| !l.starts_with('#') && !l.is_empty())
                                        .count();

                                    let item = ClipboardItem::new_files(file_list.clone());
                                    match state.db.insert_or_bump(&item) {
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
                                } else {
                                    let preview = if content.len() > 80 {
                                        format!("{}...", &content[..80])
                                    } else {
                                        content.clone()
                                    };

                                    let item = ClipboardItem::new_text(content.clone());

                                    match state.db.insert_or_bump(&item) {
                                        Ok(_) => {
                                            info!("📋 Stored: {preview}");
                                            if item.sensitive {
                                                let _ = state.db.log_audit_event(
                                                    &AuditEventKind::SensitiveItemDetected,
                                                    Some(&format!(
                                                        "Sensitive text item stored ({preview})"
                                                    )),
                                                );
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

/// Spawn a background thread running the IPC server that listens for
/// toggle/show/hide commands and writes a visibility signal file for the applet.
fn spawn_ipc_server(data_dir: std::path::PathBuf) {
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

        let visibility_path = data_dir.join(".visibility_toggle");

        loop {
            match server.accept() {
                Ok(msg) => match msg {
                    IpcMessage::Toggle | IpcMessage::Show | IpcMessage::Hide => {
                        info!("🎯 IPC received: {msg:?}");
                        let signal = match msg {
                            IpcMessage::Toggle => "toggle",
                            IpcMessage::Show => "show",
                            IpcMessage::Hide => "hide",
                            _ => continue,
                        };
                        if let Err(e) = std::fs::write(&visibility_path, signal) {
                            warn!("Failed to write visibility signal: {e}");
                        }
                    }
                    IpcMessage::Ping => {
                        debug!("IPC ping received");
                    }
                    _ => {
                        debug!("IPC message: {msg:?}");
                    }
                },
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

fn run() -> Result<()> {
    let config = Config::default();

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

    // Spawn IPC server thread for shortcut activation
    spawn_ipc_server(config.data_dir.clone());

    let conn = Connection::connect_to_env().context(
        "Failed to connect to Wayland display. \
         Ensure you are running on a Wayland compositor (e.g. COSMIC).",
    )?;

    let display = conn.display();

    let mut event_queue: EventQueue<AppState> = conn.new_event_queue();
    let qh = event_queue.handle();

    let mut state = AppState::new(db, config);

    // Trigger global advertisement
    display.get_registry(&qh, ());

    // Initial roundtrip to receive globals
    event_queue
        .roundtrip(&mut state)
        .context("Initial Wayland roundtrip failed")?;

    if state.manager.is_none() {
        anyhow::bail!(
            "Compositor does not support wlr-data-control-unstable-v1. \
             On COSMIC, set COSMIC_DATA_CONTROL_ENABLED=1."
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
                println!("author-clipboard-daemon 0.1.0");
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

/// Remove the IPC socket file, ignoring errors if it doesn't exist.
fn remove_ipc_socket() {
    let path = std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| std::path::PathBuf::from("/tmp/author-clipboard.sock"),
        |dir| std::path::PathBuf::from(dir).join("author-clipboard.sock"),
    );
    let _unused = std::fs::remove_file(path);
}
