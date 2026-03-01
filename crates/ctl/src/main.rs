use anyhow::{Context, Result};
use author_clipboard_shared::config::Config;
use author_clipboard_shared::db::Database;
use author_clipboard_shared::ipc::{IpcClient, IpcMessage};
use clap::{Parser, Subcommand};

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
}

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
            let config = Config::default();
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
            let config = Config::default();
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
            let config = Config::default();
            let db = Database::open(&config.db_path()).context("Failed to open database")?;
            let count = db.clear_unpinned().context("Failed to clear items")?;
            println!("Cleared {count} unpinned items.");
        }
        Command::Export { output } => {
            let config = Config::default();
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
            let config = Config::default();
            println!("max_items: {}", config.max_items);
            println!("max_item_size: {}", config.max_item_size);
            println!("ttl_seconds: {}", config.ttl_seconds);
            println!("cleanup_interval: {}s", config.cleanup_interval_seconds);
            println!("keyboard_shortcut: {}", config.keyboard_shortcut);
            println!("encrypt_sensitive: {}", config.encrypt_sensitive);
            println!("clear_on_lock: {}", config.clear_on_lock);
            println!("data_dir: {}", config.data_dir.display());
            println!("db_path: {}", config.db_path().display());
        }
    }
    Ok(())
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
