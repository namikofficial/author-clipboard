//! author-clipboard: unified GTK4 clipboard manager.
//!
//! Thin binary: parses CLI args and forwards to [`ui_gtk::run_popup`]
//! or [`ui_gtk::run_manager`]. The real UI is in the `ui-gtk` crate.
//!
//! Bug fixes delivered here:
//!
//! * US-001: Esc always closes (in `ui_gtk::controller::focus`).
//! * US-002: Popup opens with the list focused, not search.
//! * US-003: CLI launch opens a real `AdwApplicationWindow`.

use clap::{Parser, ValueEnum};
use ui_gtk::{ManagerConfig, PickerAction, PickerFilter, PickerSource, PopupConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// Layer-shell popup (default for keybind launch).
    Popup,
    /// XDG manager window (default for `.desktop` / terminal launch).
    Manager,
}

#[derive(Parser, Debug)]
#[command(name = "author-clipboard", about = "Unified clipboard manager")]
struct Args {
    /// Windowing mode. Defaults to popup when no TTY, manager when
    /// launched from a terminal.
    #[arg(long, value_enum)]
    mode: Option<Mode>,

    /// Initial picker source.
    #[arg(long, value_enum, default_value_t = SourceArg::History)]
    source: SourceArg,

    /// Initial filter chip.
    #[arg(long, default_value = "all")]
    filter: String,

    /// Pre-fill search.
    #[arg(long)]
    query: Option<String>,

    /// Action on Enter.
    #[arg(long, value_enum, default_value_t = ActionArg::Copy)]
    action: ActionArg,

    /// Max items to load.
    #[arg(long, default_value_t = 50)]
    count: usize,

    /// Include sensitive items.
    #[arg(long)]
    include_sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SourceArg {
    History,
    Emoji,
    Symbols,
    Kaomoji,
    Snippets,
    All,
}

impl From<SourceArg> for PickerSource {
    fn from(s: SourceArg) -> Self {
        match s {
            SourceArg::History => PickerSource::History,
            SourceArg::Emoji => PickerSource::Emoji,
            SourceArg::Symbols => PickerSource::Symbols,
            SourceArg::Kaomoji => PickerSource::Kaomoji,
            SourceArg::Snippets => PickerSource::Snippets,
            SourceArg::All => PickerSource::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ActionArg {
    Copy,
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

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Auto-detect mode: if we have a controlling TTY, prefer the
    // manager window; otherwise the popup (keybind launch).
    let mode = args.mode.unwrap_or_else(|| {
        if atty_stdin() {
            Mode::Manager
        } else {
            Mode::Popup
        }
    });

    let filter: PickerFilter = args.filter.parse().unwrap_or_default();

    tracing::info!(?mode, ?filter, "author-clipboard starting");

    match mode {
        Mode::Popup => {
            let cfg = PopupConfig {
                layer_shell: true,
                source: args.source.into(),
                filter,
                query: args.query,
                action: args.action.into(),
                count: args.count,
                include_sensitive: args.include_sensitive,
            };
            ui_gtk::run_popup(cfg)
        }
        Mode::Manager => {
            let cfg = ManagerConfig {
                initial_page: Some(args.source.into()),
            };
            ui_gtk::run_manager(cfg)
        }
    }
}

#[cfg(unix)]
fn atty_stdin() -> bool {
    extern "C" {
        fn isatty(fd: i32) -> i32;
    }
    // SAFETY: `isatty` is async-signal-safe; passing 0 (stdin) is valid.
    #[allow(unsafe_code)]
    unsafe {
        isatty(0) != 0
    }
}

#[cfg(not(unix))]
fn atty_stdin() -> bool {
    false
}
