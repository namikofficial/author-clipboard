//! author-clipboard-hypr-picker: first-party Hyprland/wlroots picker.
//!
//! Thin binary that preserves the legacy CLI flags (for Hyprland
//! keybinds) and forwards to [`ui_gtk::run_popup`]. The actual UI
//! lives in the `ui-gtk` crate.

use clap::{Parser, ValueEnum};
use ui_gtk::{PickerAction, PickerSource, PopupConfig};

#[derive(Parser, Debug)]
#[command(
    name = "author-clipboard-hypr-picker",
    about = "First-party Hyprland/wlroots clipboard picker"
)]
struct Args {
    /// Initial source.
    #[arg(
        short,
        long,
        default_value = "history",
        value_enum
    )]
    source: SourceArg,
    /// Max items to load.
    #[arg(short, long, default_value_t = 50)]
    count: usize,
    /// Include sensitive items.
    #[arg(long)]
    include_sensitive: bool,
    /// Action on Enter.
    #[arg(short, long, default_value_t = ActionArg::Copy, value_enum)]
    action: ActionArg,
    /// Pre-fill search.
    #[arg(short, long)]
    query: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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
    tracing::info!(?args, "hypr-picker starting");

    let cfg = PopupConfig {
        source: args.source.into(),
        filter: "all".to_string(),
        query: args.query,
        action: args.action.into(),
        count: args.count,
        include_sensitive: args.include_sensitive,
    };

    ui_gtk::run_popup(cfg)
}
