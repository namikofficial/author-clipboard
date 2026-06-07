use anyhow::{Context, Result};
use author_clipboard_shared::clipboard;
use author_clipboard_shared::compositor::{detect_display_server, probe_wayland_protocols};
use author_clipboard_shared::config::Config;
use author_clipboard_shared::db::Database;
use author_clipboard_shared::ipc::{IpcClient, IpcMessage};
use author_clipboard_shared::picker::{self, PickerAction, PickerOptions, PickerSource};
use clap::{Parser, Subcommand, ValueEnum};
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
    Status,
    /// List recent clipboard items
    History {
        /// Number of items to show (default: 10)
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Clear all unpinned clipboard items
    Clear,
    /// Export clipboard history to JSON
    Export {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show current configuration
    Config,
    /// Probe compositor and clipboard protocol support
    Doctor,
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
    },
    /// Print recommended Hyprland config for keybinds and window rules
    HyprlandConfig,
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

impl From<ActionArg> for PickerAction {
    fn from(a: ActionArg) -> Self {
        match a {
            ActionArg::Copy => PickerAction::Copy,
            ActionArg::QuickPaste => PickerAction::QuickPaste,
        }
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Toggle => toggle_applet()?,
        Command::Show => launch_applet()?,
        Command::Hide => kill_applet()?,
        Command::ShowAt { x, y } => send_ipc(&IpcMessage::ShowAt { x, y })?,
        Command::Ping => {
            match send_ipc(&IpcMessage::Ping) {
                Ok(()) => println!("Daemon is running"),
                Err(e) => {
                    eprintln!("Daemon is not running: {e}");
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Command::Status => {
            let config = Config::load();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let stats = db.get_stats().context("Failed to get stats")?;
            println!("Items: {}", stats.total_items);
            println!("Pinned: {}", stats.pinned_items);
            #[allow(clippy::cast_precision_loss)]
            let size_kb = stats.total_size_bytes as f64 / 1024.0;
            println!("Size: {size_kb:.1} KB");
            println!("Database: {}", config.db_path().display());
            let client = IpcClient::new();
            match client.send(&IpcMessage::Ping) {
                Ok(_) => println!("Daemon: running"),
                Err(_) => println!("Daemon: not running"),
            }
        }
        Command::History { count } => {
            let config = Config::load();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let items = db.get_recent(count).context("Failed to get items")?;
            if items.is_empty() {
                println!("No clipboard items.");
            } else {
                for item in &items {
                    let preview = if item.content.len() > 80 {
                        format!("{}...", &item.content[..80])
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
        Command::Clear => {
            let config = Config::load();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let count = db.clear_unpinned().context("Failed to clear items")?;
            println!("Cleared {count} unpinned items.");
        }
        Command::Export { output } => {
            let config = Config::load();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let json = db.export_items().context("Failed to export")?;
            if let Some(path) = output {
                std::fs::write(&path, &json)
                    .with_context(|| format!("Failed to write to {path}"))?;
                println!("Exported to {path}");
            } else {
                println!("{json}");
            }
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
            println!(
                "content_regex_denylist: {:?}",
                config.content_regex_denylist
            );
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
        Command::Doctor => run_doctor(),
        Command::Copy { id } => copy_item_by_id(id)?,
        Command::Picker {
            menu,
            source,
            count,
            prompt,
            include_sensitive,
            action,
        } => run_external_picker(menu, source, count, &prompt, include_sensitive, action)?,
        Command::HyprlandConfig => print_hyprland_config(),
    }
    Ok(())
}

fn run_doctor() {
    let server = detect_display_server();
    let protocols = probe_wayland_protocols();
    println!("Display: {server:?}");
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
}

fn copy_item_by_id(id: i64) -> Result<()> {
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

#[allow(clippy::too_many_arguments)]
fn run_external_picker(
    menu: MenuBackend,
    source: SourceArg,
    count: usize,
    prompt: &str,
    include_sensitive: bool,
    action: ActionArg,
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

    if entries.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let rows = picker::build_external_rows(&entries, true);
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

fn command_exists(name: &str) -> bool {
    ProcessCommand::new("which")
        .arg(name)
        .output()
        .is_ok_and(|output| output.status.success())
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

fn print_hyprland_config() {
    println!("{}", hyprland_config_text());
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
    use super::hyprland_config_text;

    #[test]
    fn hyprland_config_includes_native_and_external_binds() {
        let output = hyprland_config_text();
        assert!(output.contains("author-clipboard-ctl picker --menu auto"));
        assert!(output.contains("author-clipboard-hypr-picker"));
        assert!(output.contains("author-clipboard-ctl toggle"));
    }
}
