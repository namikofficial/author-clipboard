//! author-clipboard-daemon: Background clipboard monitoring daemon
//!
//! Watches for clipboard changes via the Wayland wlr-data-control protocol
//! and stores them in a local `SQLite` database.

use std::os::fd::AsFd;

use anyhow::{Context, Result};
use author_clipboard_shared::config::Config;
use author_clipboard_shared::image_store;
use author_clipboard_shared::types::ClipboardItem;
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
    fn read_offer_bytes(offer: &ZwlrDataControlOfferV1, mime_type: &str) -> Result<Vec<u8>> {
        let (read_fd, write_fd) = rustix::pipe::pipe().context("Failed to create pipe")?;

        offer.receive(mime_type.to_string(), write_fd.as_fd());

        // Close the write end so we get EOF after the compositor writes.
        drop(write_fd);

        let mut data = Vec::new();
        let mut file = std::fs::File::from(read_fd);
        std::io::Read::read_to_end(&mut file, &mut data)
            .context("Failed to read clipboard content from pipe")?;

        Ok(data)
    }

    /// Read text content from a clipboard offer via a pipe.
    fn read_offer_content(offer: &ZwlrDataControlOfferV1) -> Result<String> {
        let data = Self::read_offer_bytes(offer, "text/plain;charset=utf-8")?;
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
        _conn: &Connection,
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

                    if let Some(mime) = image_mime {
                        // Handle image clipboard
                        match Self::read_offer_bytes(&offer, &mime) {
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
                    } else if has_text {
                        match Self::read_offer_content(&offer) {
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
                                        Ok(_) => info!("📋 Stored: {preview}"),
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
                        debug!("Selection has no text/plain MIME type, skipping");
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

fn run() -> Result<()> {
    let config = Config::default();

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)
        .with_context(|| format!("Failed to create data dir: {}", config.data_dir.display()))?;

    let db = Database::open(&config.db_path()).context("Failed to open clipboard database")?;
    info!("Database opened at {}", config.db_path().display());

    // Ensure image storage directories exist
    image_store::ensure_dirs(&config.data_dir)
        .context("Failed to create image storage directories")?;

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
        event_queue
            .blocking_dispatch(&mut state)
            .context("Wayland event dispatch failed")?;
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("author-clipboard-daemon starting...");

    if let Err(e) = run() {
        error!("Fatal error: {e:#}");
        std::process::exit(1);
    }
}
