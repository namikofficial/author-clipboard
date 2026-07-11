use anyhow::{Context, Result};
use author_clipboard_shared::clipboard;
use author_clipboard_shared::compositor::{detect_display_server, probe_wayland_protocols};
use author_clipboard_shared::config::Config;
use author_clipboard_shared::db::Database;
use author_clipboard_shared::import_export::{self, ExportMode};
use author_clipboard_shared::ipc::{CopyMode, IpcClient, IpcCommand, IpcMessage};
use author_clipboard_shared::picker::{self, PickerAction, PickerOptions, PickerSource};
use author_clipboard_shared::transform::TransformKind;
use clap::{Parser, Subcommand, ValueEnum};
use std::io::Read as _;
use std::process::{Command as ProcessCommand, Stdio};

/// CLI control tool for author-clipboard
#[derive(Parser)]
#[command(
    name = "author-clipboard-ctl",
    version,
    about = "Control the author-clipboard daemon"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Toggle clipboard picker visibility
    Toggle,
    /// Show the clipboard picker
    Show,
    /// Hide the clipboard picker
    Hide,
    /// Show at specific screen coordinates
    ShowAt {
        #[arg(short, long)]
        x: i32,
        #[arg(short, long)]
        y: i32,
    },
    /// Check if daemon is running
    Ping,
    /// Get daemon status
    Status {
        /// Output as JSON (for status bars / scripts)
        #[arg(long, default_value = "false")]
        json: bool,
        /// Pretty-print JSON output
        #[arg(long, default_value = "false")]
        pretty: bool,
    },
    /// List recent clipboard items
    History {
        /// Number of items to show (default: 10)
        #[arg(short, long, default_value = "10")]
        count: usize,
        /// Output as JSON
        #[arg(long, default_value = "false")]
        json: bool,
        /// Pretty-print JSON output
        #[arg(long, default_value = "false")]
        pretty: bool,
    },
    /// Clear all unpinned clipboard items
    Clear,
    /// Skip the next eligible clipboard capture exactly once.
    IgnoreNextCopy,
    /// Export clipboard history to JSON
    Export {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Export privacy/scope mode.
        #[arg(long, value_enum, default_value_t = ExportModeArg::Redacted)]
        mode: ExportModeArg,
        /// Explicitly acknowledge that full export can contain secrets.
        #[arg(long)]
        confirm_sensitive: bool,
    },
    /// Preview or import a versioned Author Clipboard export.
    Import {
        /// Input JSON file.
        input: String,
        /// Validate and report counts without writing.
        #[arg(long)]
        dry_run: bool,
        /// Confirm writing imported items to history.
        #[arg(long)]
        confirm: bool,
    },
    /// Apply a pure text transformation.
    Transform {
        #[arg(value_enum)]
        kind: TransformArg,
        /// Input text (reads stdin when omitted).
        input: Option<String>,
        /// Language hint for fenced-code output.
        #[arg(long)]
        language: Option<String>,
        /// Treat input as sensitive.
        #[arg(long)]
        sensitive: bool,
        /// Confirm transformation of sensitive input.
        #[arg(long)]
        confirm_sensitive: bool,
    },
    /// Show current configuration
    Config,
    /// Probe compositor and clipboard protocol support
    Doctor {
        /// Emit machine-readable diagnostics.
        #[arg(long)]
        json: bool,
        /// Apply only safe, application-owned directory fixes.
        #[arg(long)]
        fix: bool,
    },
    /// Copy a history item by id
    Copy {
        /// Clipboard item id
        id: i64,
    },
    /// Open an external menu picker via wofi, rofi, or fuzzel
    Picker {
        /// Menu backend to use (`auto`, `wofi`, `fuzzel`, or `rofi`).
        #[arg(short, long, value_enum, default_value_t = MenuBackend::Auto)]
        menu: MenuBackend,
        /// Data source to pick from
        #[arg(short, long, default_value = "history")]
        source: SourceArg,
        /// Number of items to show
        #[arg(short, long, default_value = "50")]
        count: usize,
        /// Prompt shown by the menu backend
        #[arg(short, long, default_value = "Clipboard")]
        prompt: String,
        /// Include sensitive items (masked by default)
        #[arg(long)]
        include_sensitive: bool,
        /// Action on selection: copy or quick-paste
        #[arg(short, long, default_value = "copy")]
        action: ActionArg,
        /// Filter chip: all / text / images / files / pinned / starred / sensitive
        #[arg(long, default_value = "all")]
        filter: String,
    },
    /// Print recommended Hyprland config for keybinds and window rules
    HyprlandConfig {
        /// Idempotently update the Author Clipboard managed block in this file.
        #[arg(long, value_name = "PATH")]
        write: Option<std::path::PathBuf>,
    },
    /// Manage collections (list, create, delete, rename, add/remove items)
    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },
    /// Manage saved filters (list, create, delete)
    Filter {
        #[command(subcommand)]
        action: FilterAction,
    },
    /// Render a snippet template and write the result to the clipboard.
    ///
    /// Accepts the snippet name or numeric id. By default, the rendered
    /// text is copied to the clipboard AND printed to stdout. Use
    /// `--stdout` to skip the clipboard write, or `--cursor-offset` to
    /// also print the byte offset of the `${cursor}` marker.
    ///
    /// See `specs/features/026-snippet-templates/`.
    ExpandSnippet {
        /// Snippet name, or numeric id (e.g. `42`).
        name_or_id: String,
        /// Print to stdout only; do not touch the clipboard.
        #[arg(long)]
        stdout: bool,
        /// Also print `text<TAB>offset` after the rendered text.
        #[arg(long)]
        cursor_offset: bool,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum MenuBackend {
    Auto,
    Wofi,
    Rofi,
    Fuzzel,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum SourceArg {
    History,
    Snippets,
    Emoji,
    Symbols,
    Kaomoji,
    All,
}

impl From<SourceArg> for PickerSource {
    fn from(s: SourceArg) -> Self {
        match s {
            SourceArg::History => PickerSource::History,
            SourceArg::Snippets => PickerSource::Snippets,
            SourceArg::Emoji => PickerSource::Emoji,
            SourceArg::Symbols => PickerSource::Symbols,
            SourceArg::Kaomoji => PickerSource::Kaomoji,
            SourceArg::All => PickerSource::All,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum ActionArg {
    Copy,
    #[value(alias = "quick-paste")]
    QuickPaste,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum ExportModeArg {
    Redacted,
    Full,
    Snippets,
    Settings,
}
impl From<ExportModeArg> for ExportMode {
    fn from(value: ExportModeArg) -> Self {
        match value {
            ExportModeArg::Redacted => Self::Redacted,
            ExportModeArg::Full => Self::FullWithConfirmation,
            ExportModeArg::Snippets => Self::SnippetsOnly,
            ExportModeArg::Settings => Self::SettingsOnly,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum TransformArg {
    PlainText,
    MarkdownLink,
    FencedCode,
    Quote,
    JsonPretty,
    JsonMinified,
    Redacted,
}

impl From<ActionArg> for PickerAction {
    fn from(a: ActionArg) -> Self {
        match a {
            ActionArg::Copy => PickerAction::Copy,
            ActionArg::QuickPaste => PickerAction::QuickPaste,
        }
    }
}

/// Collection management subcommands
#[derive(Subcommand)]
enum CollectionAction {
    /// List all collections
    List,
    /// Create a new collection
    Create {
        /// Name of the new collection
        name: String,
    },
    /// Delete a collection by ID
    Delete {
        /// Collection ID
        id: String,
    },
    /// Rename a collection
    Rename {
        /// Collection ID
        id: String,
        /// New name for the collection
        new_name: String,
    },
    /// List items in a collection
    Items {
        /// Collection ID
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a clipboard item to a collection
    Add {
        /// Collection ID
        collection_id: String,
        /// Clipboard item ID
        item_id: i64,
    },
    /// Remove a clipboard item from a collection
    Remove {
        /// Collection ID
        collection_id: String,
        /// Clipboard item ID
        item_id: i64,
    },
}

/// Saved filter management subcommands
#[derive(Subcommand)]
enum FilterAction {
    /// List all saved filters
    List,
    /// Create or update a saved filter
    Save {
        /// Name of the filter
        name: String,
        /// Query string (e.g., "type:text pinned:true")
        query: String,
    },
    /// Delete a saved filter by name
    Delete {
        /// Name of the filter to delete
        name: String,
    },
}

#[allow(
    clippy::too_many_lines,
    clippy::single_match_else,
    clippy::redundant_closure_for_method_calls,
    clippy::uninlined_format_args
)]
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Toggle => toggle_applet()?,
        Command::Show => launch_applet()?,
        Command::Hide => kill_applet()?,
        Command::ShowAt { x, y } => send_ipc(&IpcMessage::ShowAt { x, y })?,
        Command::Ping => {
            let client = IpcClient::new();
            match client.send(&IpcMessage::Ping) {
                Ok(_) => println!("Daemon is running"),
                Err(e) => {
                    eprintln!("Daemon is not running: {e}");
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Command::Status { json, pretty } => {
            if json || pretty {
                let payload = build_status_json_payload()?;
                let rendered = if pretty {
                    serde_json::to_string_pretty(&payload).context("Failed to format JSON")?
                } else {
                    serde_json::to_string(&payload).context("Failed to serialize JSON")?
                };
                println!("{rendered}");
            } else {
                let client = IpcClient::new();
                match client.send_command(&IpcCommand::Status) {
                    Ok(resp) => {
                        if let Some(data) = resp.data {
                            println!(
                                "Items: {}",
                                data.get("item_count").unwrap_or(&serde_json::Value::Null)
                            );
                            println!(
                                "Pinned: {}",
                                data.get("pinned_count").unwrap_or(&serde_json::Value::Null)
                            );
                            if let Some(size) = data
                                .get("database_size_bytes")
                                .and_then(serde_json::Value::as_u64)
                            {
                                #[allow(clippy::cast_precision_loss)]
                                let size_kb = size as f64 / 1024.0;
                                println!("Size: {size_kb:.1} KB");
                            }
                            println!("Daemon: running");
                        }
                    }
                    Err(_) => {
                        // Fallback to direct DB access if daemon is not running
                        println!("Daemon: not running (using direct DB access)");
                        let config = Config::load();
                        if let Ok(db) = Database::open(&config.db_path()) {
                            if let Ok(stats) = db.get_stats() {
                                println!("Items: {}", stats.total_items);
                                println!("Pinned: {}", stats.pinned_items);
                                #[allow(clippy::cast_precision_loss)]
                                let size_kb = stats.total_size_bytes as f64 / 1024.0;
                                println!("Size: {size_kb:.1} KB");
                            }
                        }
                    }
                }
            }
        }
        Command::History {
            count,
            json,
            pretty,
        } => {
            let client = IpcClient::new();
            match client.send_command(&IpcCommand::History {
                limit: count,
                offset: None,
                filters: None,
            }) {
                Ok(resp) => {
                    if json || pretty {
                        print_json(&resp, pretty)?;
                    } else if let Some(data) = resp.data {
                        let items = data
                            .get("items")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        if items.is_empty() {
                            println!("No clipboard items.");
                        } else {
                            for item in items {
                                let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                                let preview =
                                    item.get("preview").and_then(|v| v.as_str()).unwrap_or("");
                                let pinned = item
                                    .get("pinned")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                let content_type = item
                                    .get("content_type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("text");
                                let pinned_str = if pinned { " [pinned]" } else { "" };
                                println!("[{}] {}{} ({})", id, preview, pinned_str, content_type);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access (warning: bypasses daemon policy)");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    let items = db.get_recent(count).context("Failed to get items")?;
                    if items.is_empty() {
                        println!("No clipboard items.");
                    } else {
                        for item in &items {
                            let preview = if item.content.chars().count() > 80 {
                                format!("{}...", item.content.chars().take(80).collect::<String>())
                            } else {
                                item.content.clone()
                            };
                            let preview = preview.replace('\n', " ");
                            let pinned = if item.pinned { " [pinned]" } else { "" };
                            println!(
                                "[{}] {}{} ({})",
                                item.id,
                                preview,
                                pinned,
                                item.content_type.as_str()
                            );
                        }
                    }
                }
            }
        }
        Command::Clear => {
            let client = IpcClient::new();
            match client.send_command(&IpcCommand::ClearUnpinned) {
                Ok(resp) => {
                    if let Some(data) = resp.data {
                        let count = data
                            .get("deleted_count")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0);
                        println!("Cleared {count} unpinned items.");
                    }
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access (warning: bypasses daemon policy)");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    let count = db.clear_unpinned().context("Failed to clear items")?;
                    println!("Cleared {count} unpinned items.");
                }
            }
        }
        Command::IgnoreNextCopy => {
            let response = IpcClient::new()
                .send_command(&IpcCommand::IgnoreNextCopy)
                .context("Daemon unavailable; ignore-next-copy was not armed")?;
            anyhow::ensure!(response.ok, "Daemon rejected ignore-next-copy request");
            println!("The next eligible clipboard capture will be ignored.");
        }
        Command::Export {
            output,
            mode,
            confirm_sensitive,
        } => {
            let config = Config::load();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let items = db
                .get_recent(i32::MAX as usize)
                .context("Failed to load history")?;
            let json = import_export::export_history(&items, mode.into(), confirm_sensitive)
                .context("Failed to export")?;
            if let Some(path) = output {
                std::fs::write(&path, &json)
                    .with_context(|| format!("Failed to write to {path}"))?;
                println!("Exported to {path}");
            } else {
                println!("{json}");
            }
        }
        Command::Import {
            input,
            dry_run,
            confirm,
        } => {
            let json = std::fs::read_to_string(&input)
                .with_context(|| format!("Failed to read {input}"))?;
            let preview =
                import_export::preview_import(&json).context("Import validation failed")?;
            println!(
                "History: {}, sensitive: {}",
                preview.history_count, preview.sensitive_count
            );
            for warning in &preview.warnings {
                eprintln!("Warning: {warning}");
            }
            if !dry_run {
                anyhow::ensure!(
                    confirm,
                    "Import writes require --confirm (use --dry-run to preview)"
                );
                let items =
                    import_export::validated_history(&json).context("Import validation failed")?;
                let config = Config::load();
                let db = Database::open(&config.db_path()).context("Failed to open database")?;
                let legacy_json =
                    serde_json::to_string(&items).context("Failed to prepare import")?;
                let count = db.import_items(&legacy_json).map_err(anyhow::Error::msg)?;
                println!("Imported {count} items.");
            }
        }
        Command::Transform {
            kind,
            input,
            language,
            sensitive,
            confirm_sensitive,
        } => {
            let input = match input {
                Some(value) => value,
                None => {
                    let mut value = String::new();
                    std::io::stdin()
                        .read_to_string(&mut value)
                        .context("Failed to read stdin")?;
                    value
                }
            };
            let kind = match kind {
                TransformArg::PlainText => TransformKind::PlainText,
                TransformArg::MarkdownLink => TransformKind::MarkdownLink,
                TransformArg::FencedCode => TransformKind::FencedCode {
                    language_hint: language,
                },
                TransformArg::Quote => TransformKind::Quote,
                TransformArg::JsonPretty => TransformKind::JsonPretty,
                TransformArg::JsonMinified => TransformKind::JsonMinified,
                TransformArg::Redacted => TransformKind::Redacted,
            };
            let output = author_clipboard_shared::transform::apply(
                &input,
                &kind,
                sensitive,
                confirm_sensitive,
            )
            .context("Transform failed")?;
            println!("{output}");
        }
        Command::Config => {
            let config = Config::load();
            println!("max_items: {}", config.max_items);
            println!("max_item_size: {}", config.max_item_size);
            println!("ttl_seconds: {}", config.ttl_seconds);
            println!("cleanup_interval: {}s", config.cleanup_interval_seconds);
            println!("keyboard_shortcut: {}", config.keyboard_shortcut);
            println!("encrypt_sensitive: {}", config.encrypt_sensitive);
            println!("clear_on_lock: {}", config.clear_on_lock);
            println!("dedup_window_seconds: {}", config.dedup_window_seconds);
            println!("mime_denylist: {:?}", config.mime_denylist);
            println!("content_denylist: {:?}", config.content_denylist);
            println!("content_pattern_mode: {:?}", config.content_pattern_mode);
            println!("picker.default_source: {}", config.picker.default_source);
            println!("picker.default_mode: {}", config.picker.default_mode);
            println!("picker.max_results: {}", config.picker.max_results);
            println!(
                "picker.show_sensitive_previews: {}",
                config.picker.show_sensitive_previews
            );
            println!(
                "picker.confirm_sensitive_copy: {}",
                config.picker.confirm_sensitive_copy
            );
            println!(
                "picker.close_after_copy: {}",
                config.picker.close_after_copy
            );
            println!("picker.width: {}", config.picker.width);
            println!("picker.height: {}", config.picker.height);
            println!("data_dir: {}", config.data_dir.display());
            println!("db_path: {}", config.db_path().display());
        }
        Command::Doctor { json, fix } => run_doctor(json, fix)?,
        Command::Copy { id } => copy_item_by_id(id)?,
        Command::ExpandSnippet {
            name_or_id,
            stdout,
            cursor_offset,
        } => run_expand_snippet(&name_or_id, stdout, cursor_offset)?,
        Command::Picker {
            menu,
            source,
            count,
            prompt,
            include_sensitive,
            action,
            filter,
        } => run_external_picker(
            menu,
            source,
            count,
            &prompt,
            include_sensitive,
            action,
            filter.as_str(),
        )?,
        Command::HyprlandConfig { write } => run_hyprland_config(write.as_deref())?,
        Command::Collection { action } => run_collection(action)?,
        Command::Filter { action } => run_filter(action)?,
    }
    Ok(())
}

impl Command {
    /// Returns `true` when a subcommand that supports `--json` was
    /// invoked with that flag. Kept for forward compatibility (and
    /// potential shared tests) — the actual dispatch is in the
    /// `match cli.command` arm.
    #[allow(dead_code)]
    fn is_json(&self) -> bool {
        match self {
            Command::History { json, .. } | Command::Status { json, .. } => *json,
            _ => false,
        }
    }
}

fn print_json(resp: &author_clipboard_shared::ipc::IpcResponse, pretty: bool) -> Result<()> {
    let json = if pretty {
        serde_json::to_string_pretty(&resp).context("Failed to format JSON")?
    } else {
        serde_json::to_string(&resp).context("Failed to serialize JSON")?
    };
    println!("{json}");
    Ok(())
}

/// Build the structured status payload for `--json` output.
///
/// Always reads from the local `SQLite` database so the payload is
/// available even when the daemon is down (graceful degradation for
/// the Waybar / Wayle module). The `running` and `daemon_pid` fields
/// reflect the live IPC ping.
fn build_status_json_payload() -> Result<serde_json::Value> {
    let client = IpcClient::new();
    let (running, daemon_pid) = match client.send_command(&IpcCommand::Ping) {
        Ok(resp) => {
            let pid = resp
                .data
                .as_ref()
                .and_then(|d| d.get("daemon_pid"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|p| u32::try_from(p).ok());
            (true, pid)
        }
        _ => (false, None),
    };

    let config = Config::load();
    let db = Database::open(&config.db_path()).context("Failed to open database")?;

    let stats = db.get_stats().context("Failed to read stats")?;
    let most_recent = db.get_most_recent().context("Failed to read most recent")?;

    let (last_type, last_preview, last_timestamp, sensitive_last) = match most_recent {
        Some(item) => {
            let preview = if item.sensitive {
                "Sensitive item".to_string()
            } else {
                preview_text(&item)
            };
            (
                item.content_type.as_str().to_string(),
                preview,
                Some(item.timestamp.timestamp()),
                item.sensitive,
            )
        }
        None => ("text".to_string(), String::new(), None, false),
    };

    Ok(serde_json::json!({
        "running": running,
        "daemon_pid": daemon_pid,
        "total": stats.total_items,
        "pinned": stats.pinned_items,
        "last_type": last_type,
        "last_preview": last_preview,
        "last_timestamp": last_timestamp,
        "sensitive_last": sensitive_last,
    }))
}

/// Truncate the most recent item to a single-line preview suitable for
/// the Waybar tooltip. Strips newlines and limits the length.
fn preview_text(item: &author_clipboard_shared::ClipboardItem) -> String {
    let raw = item.plain_text.as_deref().unwrap_or(&item.content);
    let single_line: String = raw.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    if single_line.chars().count() > 60 {
        let truncated: String = single_line.chars().take(57).collect();
        format!("{truncated}…")
    } else {
        single_line
    }
}

#[allow(clippy::too_many_lines)]
fn run_doctor(json: bool, fix: bool) -> Result<()> {
    let config = Config::load();
    let mut fixes = Vec::new();
    if fix && !config.data_dir.exists() {
        std::fs::create_dir_all(&config.data_dir).with_context(|| {
            format!(
                "failed to create application data directory {}",
                config.data_dir.display()
            )
        })?;
        fixes.push(format!("created {}", config.data_dir.display()));
    }
    let server = detect_display_server();
    let protocols = probe_wayland_protocols();
    let daemon = IpcClient::new().send_command(&IpcCommand::Ping).is_ok();
    let database = Database::open(&config.db_path()).is_ok();
    let checks = serde_json::json!({
        "daemon": daemon,
        "config_loaded": true,
        "data_dir": config.data_dir.is_dir(),
        "database": database,
        "wayland": protocols.wayland,
        "wlr_data_control": protocols.wlr_data_control,
        "seat": protocols.seat,
        "compositor": format!("{server:?}"),
        "wl_copy": command_exists("wl-copy"),
        "wtype": command_exists("wtype"),
        "ydotool": command_exists("ydotool"),
        "wofi": command_exists("wofi"),
        "fuzzel": command_exists("fuzzel"),
        "rofi": command_exists("rofi"),
    });
    let healthy = daemon && database && protocols.wayland && protocols.seat;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "healthy": healthy,
                "checks": checks,
                "fixes": fixes,
                "probe_error": protocols.error,
            }))?
        );
        return Ok(());
    }
    println!("Display: {server:?}");
    println!(
        "Daemon: {}",
        if daemon { "reachable" } else { "unavailable" }
    );
    println!(
        "Database: {}",
        if database { "readable" } else { "unavailable" }
    );
    println!(
        "Wayland: {}",
        if protocols.wayland {
            "connected"
        } else {
            "unavailable"
        }
    );
    println!(
        "wlr-data-control: {}",
        if protocols.wlr_data_control {
            "available"
        } else {
            "missing"
        }
    );
    println!(
        "wl_seat: {}",
        if protocols.seat {
            "available"
        } else {
            "missing"
        }
    );
    if let Some(error) = protocols.error {
        println!("Probe error: {error}");
    }
    println!(
        "Clipboard capture: {}",
        if protocols.wayland && protocols.wlr_data_control && protocols.seat {
            "supported"
        } else {
            "unsupported"
        }
    );
    for tool in ["wl-copy", "wtype", "ydotool", "wofi", "fuzzel", "rofi"] {
        println!(
            "{tool}: {}",
            if command_exists(tool) {
                "available"
            } else {
                "missing (optional)"
            }
        );
    }
    for applied in fixes {
        println!("Fixed: {applied}");
    }
    Ok(())
}

fn command_exists(name: &str) -> bool {
    ProcessCommand::new("sh")
        .args(["-c", "command -v -- \"$1\" >/dev/null 2>&1", "doctor", name])
        .status()
        .is_ok_and(|status| status.success())
}

#[allow(clippy::single_match_else)]
fn run_expand_snippet(name_or_id: &str, stdout_only: bool, show_cursor_offset: bool) -> Result<()> {
    // Resolve name → id if the argument isn't numeric.
    let id = if let Ok(n) = name_or_id.parse::<i64>() {
        n
    } else {
        // List snippets, match by exact name.
        let client = IpcClient::new();
        let resp = client
            .send_command(&IpcCommand::ListSnippets)
            .context("Failed to list snippets")?;
        let snippets = resp
            .data
            .as_ref()
            .and_then(|d| d.get("snippets"))
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();
        let mut found: Option<i64> = None;
        for s in &snippets {
            let sname = s.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let sid = s.get("id").and_then(serde_json::Value::as_i64).unwrap_or(0);
            if sname == name_or_id {
                found = Some(sid);
                break;
            }
        }
        match found {
            Some(id) => id,
            None => {
                eprintln!("No snippet named '{name_or_id}'");
                std::process::exit(3);
            }
        }
    };

    let client = IpcClient::new();
    let resp = client
        .send_command(&IpcCommand::RenderSnippet { id })
        .context("Failed to render snippet")?;

    if !resp.ok {
        let code = resp.error.as_ref().map_or("UNKNOWN", |e| e.code.as_str());
        let message = resp
            .error
            .as_ref()
            .map_or("(no detail)", |e| e.message.as_str());
        if code == "SNIPPET_NOT_FOUND" {
            eprintln!("Snippet id={id} not found");
        } else {
            eprintln!("Error ({code}): {message}");
        }
        std::process::exit(4);
    }

    let data = resp
        .data
        .as_ref()
        .context("RenderSnippet returned no data")?;
    let content = data
        .get("content")
        .and_then(|v| v.as_str())
        .context("RenderSnippet response missing 'content'")?
        .to_owned();
    let cursor = data
        .get("cursor_offset")
        .and_then(serde_json::Value::as_u64);

    if show_cursor_offset {
        // Machine-friendly: text<TAB>offset. Useful for piping into
        // smart-paste tooling that consumes the offset separately.
        match cursor {
            Some(off) => println!("{content}\t{off}"),
            None => println!("{content}\t"),
        }
    } else {
        println!("{content}");
    }

    if !stdout_only {
        // Copy to clipboard via the existing IPC Copy command using a
        // synthetic data path: write to the system clipboard directly
        // with wl-copy. This avoids needing a new IPC command for
        // "set clipboard to arbitrary text".
        let wlcopy = ProcessCommand::new("wl-copy")
            .arg("--type")
            .arg("text/plain")
            .stdin(Stdio::piped())
            .spawn();
        match wlcopy {
            Ok(mut child) => {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = stdin.write_all(content.as_bytes());
                }
                let status = child.wait().context("wl-copy failed")?;
                if !status.success() {
                    eprintln!(
                        "wl-copy exited with status {status}; rendered text printed but not copied"
                    );
                }
            }
            Err(e) => {
                eprintln!("wl-copy not available ({e}); rendered text printed but not copied");
            }
        }
    }

    Ok(())
}

#[allow(clippy::single_match_else)]
fn copy_item_by_id(id: i64) -> Result<()> {
    // Try IPC first
    let client = IpcClient::new();
    match client.send_command(&IpcCommand::Copy {
        id,
        mode: CopyMode::Copy,
        mime: None,
    }) {
        Ok(resp) => {
            if let Some(data) = resp.data {
                let mime_type = data
                    .get("mime_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text/plain");
                println!("Copied item {id} as {mime_type}");
            }
            Ok(())
        }
        Err(_) => {
            // Fallback to direct DB access
            eprintln!(
                "Daemon not running, using direct DB access (warning: bypasses daemon policy)"
            );
            let config = Config::load();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let item = db
                .get_by_id(id)
                .context("Failed to read clipboard item")?
                .with_context(|| format!("No clipboard item with id {id}"))?;
            let result = clipboard::set_clipboard_item(&item, &config.data_dir)
                .with_context(|| format!("Failed to copy item {id}"))?;
            println!("Copied item {id} as {}", result.mime_type);
            Ok(())
        }
    }
}

#[allow(clippy::too_many_lines, clippy::single_match_else)]
fn run_collection(action: CollectionAction) -> Result<()> {
    let client = IpcClient::new();

    match action {
        CollectionAction::List => {
            match client.send_command(&IpcCommand::ListCollections) {
                Ok(resp) => {
                    if let Some(data) = resp.data {
                        let collections = data.get("collections").and_then(|v| v.as_array());
                        if let Some(collections) = collections {
                            if collections.is_empty() {
                                println!("No collections.");
                            } else {
                                for col in collections {
                                    let name =
                                        col.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                                    let id = col.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                                    let created = col
                                        .get("created_at")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("?");
                                    println!("{id}  {name}  (created: {created})");
                                }
                            }
                        }
                    }
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    let collections = db
                        .list_collections()
                        .context("Failed to list collections")?;
                    if collections.is_empty() {
                        println!("No collections.");
                    } else {
                        for col in collections {
                            println!("{}  {}  (created: {})", col.id, col.name, col.created_at);
                        }
                    }
                    Ok(())
                }
            }
        }
        CollectionAction::Create { name } => {
            match client.send_command(&IpcCommand::CreateCollection { name: name.clone() }) {
                Ok(resp) => {
                    if let Some(data) = resp.data {
                        let id = data.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("Created collection '{name}' with id: {id}");
                    } else {
                        println!("Created collection '{name}'");
                    }
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    let id = db
                        .create_collection(&name)
                        .context("Failed to create collection")?;
                    println!("Created collection '{name}' with id: {id}");
                    Ok(())
                }
            }
        }
        CollectionAction::Delete { id } => {
            match client.send_command(&IpcCommand::DeleteCollection { id: id.clone() }) {
                Ok(_) => {
                    println!("Deleted collection: {id}");
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    db.delete_collection(&id)
                        .context("Failed to delete collection")?;
                    println!("Deleted collection: {id}");
                    Ok(())
                }
            }
        }
        CollectionAction::Rename { id, new_name } => {
            match client.send_command(&IpcCommand::RenameCollection {
                id: id.clone(),
                new_name: new_name.clone(),
            }) {
                Ok(_) => {
                    println!("Renamed collection {id} to '{new_name}'");
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    db.rename_collection(&id, &new_name)
                        .context("Failed to rename collection")?;
                    println!("Renamed collection {id} to '{new_name}'");
                    Ok(())
                }
            }
        }
        CollectionAction::Items { id, json } => {
            match client.send_command(&IpcCommand::GetCollectionItems { id: id.clone() }) {
                Ok(resp) => {
                    if json {
                        if let Some(data) = resp.data {
                            let items = data.get("items");
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&items).unwrap_or_default()
                            );
                        }
                    } else if let Some(data) = resp.data {
                        let items = data.get("items").and_then(|v| v.as_array());
                        if let Some(items) = items {
                            if items.is_empty() {
                                println!("Collection '{id}' is empty.");
                            } else {
                                for item in items {
                                    let content =
                                        item.get("content").and_then(|v| v.as_str()).unwrap_or("?");
                                    let item_id = item
                                        .get("id")
                                        .and_then(serde_json::Value::as_i64)
                                        .unwrap_or(0);
                                    let content_type = item
                                        .get("content_type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("text");
                                    println!(
                                        "[{item_id}] {content_type}: {}",
                                        &content[..content.len().min(60)]
                                    );
                                }
                            }
                        }
                    }
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    let items = db
                        .get_collection_items(&id)
                        .context("Failed to get collection items")?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&items).unwrap_or_default()
                        );
                    } else if items.is_empty() {
                        println!("Collection '{id}' is empty.");
                    } else {
                        for item in items {
                            let preview = if item.content.len() > 60 {
                                format!("{}...", &item.content[..60])
                            } else {
                                item.content.clone()
                            };
                            println!("[{}] {}: {}", item.id, item.content_type.as_str(), preview);
                        }
                    }
                    Ok(())
                }
            }
        }
        CollectionAction::Add {
            collection_id,
            item_id,
        } => {
            match client.send_command(&IpcCommand::AddToCollection {
                collection_id: collection_id.clone(),
                item_id,
            }) {
                Ok(_) => {
                    println!("Added item {item_id} to collection {collection_id}");
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    db.add_to_collection(&collection_id, item_id)
                        .context("Failed to add to collection")?;
                    println!("Added item {item_id} to collection {collection_id}");
                    Ok(())
                }
            }
        }
        CollectionAction::Remove {
            collection_id,
            item_id,
        } => {
            match client.send_command(&IpcCommand::RemoveFromCollection {
                collection_id: collection_id.clone(),
                item_id,
            }) {
                Ok(_) => {
                    println!("Removed item {item_id} from collection {collection_id}");
                    Ok(())
                }
                Err(_) => {
                    // Fallback to direct DB access
                    eprintln!("Daemon not running, using direct DB access");
                    let config = Config::load();
                    let db =
                        Database::open(&config.db_path()).context("Failed to open database")?;
                    db.remove_from_collection(&collection_id, item_id)
                        .context("Failed to remove from collection")?;
                    println!("Removed item {item_id} from collection {collection_id}");
                    Ok(())
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines, clippy::single_match_else)]
fn run_filter(action: FilterAction) -> Result<()> {
    let config = Config::load();
    let db = Database::open(&config.db_path()).context("Failed to open database")?;

    match action {
        FilterAction::List => {
            let filters = db
                .list_saved_filters()
                .context("Failed to list saved filters")?;
            if filters.is_empty() {
                println!("No saved filters.");
            } else {
                for f in filters {
                    println!("{}  {}  (query: {})", f.id, f.name, f.query);
                }
            }
            Ok(())
        }
        FilterAction::Save { name, query } => {
            let id = db
                .upsert_saved_filter(&name, &query)
                .context("Failed to save filter")?;
            println!("Saved filter '{name}' (id: {id}) with query: {query}");
            Ok(())
        }
        FilterAction::Delete { name } => {
            db.delete_saved_filter_by_name(&name)
                .context("Failed to delete filter")?;
            println!("Deleted filter: {name}");
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_external_picker(
    menu: MenuBackend,
    source: SourceArg,
    count: usize,
    prompt: &str,
    include_sensitive: bool,
    action: ActionArg,
    filter: &str,
) -> Result<()> {
    let backend = resolve_menu_backend(menu)
        .context("No picker backend found. Install wofi, rofi, or fuzzel.")?;

    let config = Config::load();
    let db = Database::open(&config.db_path()).context("Failed to open database")?;

    let options = PickerOptions {
        source: source.into(),
        limit: count,
        query: None,
        include_sensitive,
        action: action.into(),
    };

    let entries =
        picker::load_entries(&db, &config, &options).context("Failed to load picker entries")?;

    // Parse the filter arg, fall back to All on unknown.
    let filter_enum: author_clipboard_shared::picker::PickerFilter =
        filter.parse().unwrap_or_default();

    if entries.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let (entries, rows) = picker::build_external_rows(&entries, filter_enum, true);
    let labels: Vec<String> = rows.iter().map(|row| row.label.clone()).collect();

    let selected = run_menu_backend(backend, prompt, &labels)?;
    if selected.is_empty() {
        return Ok(());
    }

    if let Some(index) = picker::parse_external_row_selection(&selected, &rows, true) {
        if let Some(entry) = entries.get(index) {
            let result = picker::restore_entry(entry, &config, action.into(), include_sensitive)
                .context("Failed to restore item")?;
            println!("Copied as {} ({})", result.mime_type, result.behavior);
            return Ok(());
        }
    }

    anyhow::bail!("Could not map menu selection back to an item");
}

fn resolve_menu_backend(menu: MenuBackend) -> Option<MenuBackend> {
    match menu {
        MenuBackend::Auto => detect_menu_backend(),
        MenuBackend::Wofi | MenuBackend::Rofi | MenuBackend::Fuzzel => Some(menu),
    }
}

fn detect_menu_backend() -> Option<MenuBackend> {
    [MenuBackend::Wofi, MenuBackend::Fuzzel, MenuBackend::Rofi]
        .into_iter()
        .find(|backend| command_exists(backend.command_name()))
}

fn run_menu_backend(backend: MenuBackend, prompt: &str, labels: &[String]) -> Result<String> {
    let mut command = ProcessCommand::new(backend.command_name());
    command.args(backend.args(prompt));

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to launch {}", backend.command_name()))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(labels.join("\n").as_bytes())?;
        stdin.write_all(b"\n")?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.trim().is_empty() {
            return Ok(String::new());
        }
        anyhow::bail!("{} failed: {stderr}", backend.command_name());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn hyprland_config_text() -> String {
    [
        "# Author Clipboard - Hyprland configuration",
        "# Add these to your hyprland.conf",
        "",
        "# External menu picker (wofi/fuzzel/rofi)",
        "bind = SUPER, V, exec, author-clipboard-ctl picker --menu auto",
        "",
        "# First-party Hyprland-native picker",
        "bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker",
        "",
        "# Optional COSMIC applet toggle",
        "bind = SUPER ALT, V, exec, author-clipboard-ctl toggle",
        "",
        "# Verify app class and choose rules (if not using layer-shell):",
        "# hyprctl clients | grep -i author",
        "",
        "# Make sure the daemon is running:",
        "# systemctl --user enable --now author-clipboard-daemon",
    ]
    .join("\n")
}

const HYPR_BLOCK_START: &str = "# >>> author-clipboard managed block >>>";
const HYPR_BLOCK_END: &str = "# <<< author-clipboard managed block <<<";

fn hyprland_managed_block() -> String {
    format!(
        "{HYPR_BLOCK_START}\n{}\n{HYPR_BLOCK_END}",
        hyprland_config_text()
    )
}

fn merge_hyprland_config(existing: &str) -> Result<String> {
    let block = hyprland_managed_block();
    match (
        existing.find(HYPR_BLOCK_START),
        existing.find(HYPR_BLOCK_END),
    ) {
        (Some(start), Some(end)) if end >= start => {
            let end = end + HYPR_BLOCK_END.len();
            Ok(format!(
                "{}{}{}",
                &existing[..start],
                block,
                &existing[end..]
            ))
        }
        (None, None) => {
            let separator = if existing.is_empty() || existing.ends_with("\n\n") {
                ""
            } else if existing.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            Ok(format!("{existing}{separator}{block}\n"))
        }
        _ => anyhow::bail!("refusing to modify malformed Author Clipboard managed block"),
    }
}

fn run_hyprland_config(write: Option<&std::path::Path>) -> Result<()> {
    let Some(path) = write else {
        println!("{}", hyprland_managed_block());
        return Ok(());
    };
    let existing = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()))
        }
    };
    let merged = merge_hyprland_config(&existing)?;
    if merged == existing {
        println!("Hyprland config already up to date: {}", path.display());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            anyhow::bail!("parent directory does not exist: {}", parent.display());
        }
    }
    std::fs::write(path, merged)
        .with_context(|| format!("failed to write Hyprland config {}", path.display()))?;
    println!(
        "Updated Author Clipboard managed block in {}",
        path.display()
    );
    Ok(())
}

impl MenuBackend {
    fn command_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Wofi => "wofi",
            Self::Rofi => "rofi",
            Self::Fuzzel => "fuzzel",
        }
    }

    fn args(self, prompt: &str) -> Vec<&str> {
        match self {
            Self::Auto => vec![],
            Self::Rofi => vec!["-dmenu", "-p", prompt],
            Self::Wofi | Self::Fuzzel => vec!["--dmenu", "--prompt", prompt],
        }
    }
}

fn send_ipc(message: &IpcMessage) -> Result<()> {
    let client = IpcClient::new();
    match client.send(message) {
        Ok(Some(response)) => {
            println!("Response: {response:?}");
        }
        Ok(None) => {
            println!("OK");
        }
        Err(e) => {
            anyhow::bail!("Failed to send IPC message: {e}");
        }
    }
    Ok(())
}

fn is_applet_running() -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", "author-clipboard$"])
        .output()
        .is_ok_and(|o| o.status.success())
}

fn toggle_applet() -> Result<()> {
    if is_applet_running() {
        kill_applet()
    } else {
        launch_applet()
    }
}

fn launch_applet() -> Result<()> {
    if is_applet_running() {
        println!("Applet already running");
        return Ok(());
    }
    std::process::Command::new("author-clipboard")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to launch applet. Is author-clipboard in PATH?")?;
    println!("Applet launched");
    Ok(())
}

fn kill_applet() -> Result<()> {
    let output = std::process::Command::new("pgrep")
        .args(["-f", "author-clipboard$"])
        .output()
        .context("Failed to run pgrep")?;
    if !output.status.success() {
        println!("Applet not running");
        return Ok(());
    }

    let pids = String::from_utf8_lossy(&output.stdout);
    for pid in pids.lines() {
        let pid = pid.trim();
        if !pid.is_empty() {
            let _ = std::process::Command::new("kill").arg(pid).output();
        }
    }
    println!("Applet stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{hyprland_config_text, merge_hyprland_config, HYPR_BLOCK_START};

    #[test]
    fn hyprland_config_includes_native_and_external_binds() {
        let output = hyprland_config_text();
        assert!(output.contains("author-clipboard-ctl picker --menu auto"));
        assert!(output.contains("author-clipboard-hypr-picker"));
        assert!(output.contains("author-clipboard-ctl toggle"));
    }

    #[test]
    fn hyprland_managed_block_is_idempotent_and_preserves_user_text() {
        let original = "# user rule\nbind = SUPER, Q, killactive\n";
        let once = merge_hyprland_config(original).unwrap();
        let twice = merge_hyprland_config(&once).unwrap();
        assert_eq!(once, twice);
        assert!(once.starts_with(original));
        assert_eq!(once.matches(HYPR_BLOCK_START).count(), 1);
    }

    #[test]
    fn malformed_hyprland_managed_block_is_rejected() {
        assert!(merge_hyprland_config(HYPR_BLOCK_START).is_err());
    }
}
