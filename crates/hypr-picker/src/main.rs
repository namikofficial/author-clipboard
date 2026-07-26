//! author-clipboard-hypr-picker: first-party Hyprland/wlroots picker.
//!
//! Thin binary that preserves the legacy CLI flags (for Hyprland
//! keybinds) and forwards to [`ui_gtk::run_popup`]. The actual UI
//! lives in the `ui-gtk` crate.

use clap::{Parser, ValueEnum};
use std::str::FromStr;
use ui_gtk::{PickerAction, PickerFilter, PickerSource, PopupConfig};

#[derive(Parser, Debug)]
#[command(
    name = "author-clipboard-hypr-picker",
    about = "First-party Hyprland/wlroots clipboard picker"
)]
struct Args {
    /// Initial source.
    #[arg(short, long, default_value = "history", value_enum)]
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
    /// Filter chip (all, text, images, files, pinned, starred, sensitive).
    #[arg(short, long, default_value = "all")]
    filter: String,

    /// Force XDG window mode instead of layer-shell (debugging fallback).
    ///
    /// When set, the picker runs as a normal resizable window that can be
    /// tiled, resized, and moved freely. Useful when layer-shell is causing
    /// issues or when testing on non-layer-shell compositors.
    #[arg(long)]
    xdg_window: bool,

    /// Deprecated: layer-shell is now the default. Kept for backward
    /// compatibility with existing Hyprland keybinds. Ignored.
    #[arg(long, hide = true)]
    layer_shell: bool,
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

    let filter = PickerFilter::from_str(&args.filter).unwrap_or(PickerFilter::All);
    let use_layer_shell = !args.xdg_window;
    let cfg = PopupConfig {
        layer_shell: use_layer_shell,
        source: args.source.into(),
        filter,
        query: args.query,
        action: args.action.into(),
        count: args.count,
        include_sensitive: args.include_sensitive,
    };

    ui_gtk::run_popup(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::str::FromStr;

    #[test]
    fn default_args_enable_layer_shell() {
        let args = Args::parse_from(["hypr-picker"]);
        assert!(!args.xdg_window);
        let use_layer_shell = !args.xdg_window;
        assert!(use_layer_shell);
    }

    #[test]
    fn xdg_window_flag_disables_layer_shell() {
        let args = Args::parse_from(["hypr-picker", "--xdg-window"]);
        assert!(args.xdg_window);
        let use_layer_shell = !args.xdg_window;
        assert!(!use_layer_shell);
    }

    #[test]
    fn deprecated_layer_shell_flag_is_ignored() {
        let args = Args::parse_from(["hypr-picker", "--layer-shell"]);
        // --layer-shell is hidden/deprecated; xdg_window stays false
        assert!(!args.xdg_window);
        let use_layer_shell = !args.xdg_window;
        assert!(use_layer_shell);
    }

    #[test]
    fn source_defaults_to_history() {
        let args = Args::parse_from(["hypr-picker"]);
        assert_eq!(args.source, SourceArg::History);
    }

    #[test]
    fn action_defaults_to_copy() {
        let args = Args::parse_from(["hypr-picker"]);
        assert_eq!(args.action, ActionArg::Copy);
    }

    #[test]
    fn filter_defaults_to_all() {
        let args = Args::parse_from(["hypr-picker"]);
        assert_eq!(args.filter, "all");
    }

    #[test]
    fn count_defaults_to_50() {
        let args = Args::parse_from(["hypr-picker"]);
        assert_eq!(args.count, 50);
    }

    #[test]
    fn popup_config_reflects_xdg_window_flag() {
        let args = Args::parse_from(["hypr-picker", "--xdg-window"]);
        let use_layer_shell = !args.xdg_window;
        let cfg = PopupConfig {
            layer_shell: use_layer_shell,
            ..Default::default()
        };
        assert!(!cfg.layer_shell);
    }

    #[test]
    fn popup_config_layer_shell_by_default() {
        let args = Args::parse_from(["hypr-picker"]);
        let use_layer_shell = !args.xdg_window;
        let cfg = PopupConfig {
            layer_shell: use_layer_shell,
            ..Default::default()
        };
        assert!(cfg.layer_shell);
    }

    #[test]
    fn include_sensitive_defaults_to_false() {
        let args = Args::parse_from(["hypr-picker"]);
        assert!(!args.include_sensitive);
    }

    #[test]
    fn include_sensitive_flag_works() {
        let args = Args::parse_from(["hypr-picker", "--include-sensitive"]);
        assert!(args.include_sensitive);
    }

    #[test]
    fn query_defaults_to_none() {
        let args = Args::parse_from(["hypr-picker"]);
        assert!(args.query.is_none());
    }

    #[test]
    fn query_parses() {
        let args = Args::parse_from(["hypr-picker", "--query", "search term"]);
        assert_eq!(args.query.as_deref(), Some("search term"));
    }

    #[test]
    fn action_can_be_quick_paste() {
        let args = Args::parse_from(["hypr-picker", "--action", "quick-paste"]);
        assert_eq!(args.action, ActionArg::QuickPaste);
    }

    #[test]
    fn action_can_be_copy() {
        let args = Args::parse_from(["hypr-picker", "--action", "copy"]);
        assert_eq!(args.action, ActionArg::Copy);
    }

    #[test]
    fn source_snippets_parses() {
        let args = Args::parse_from(["hypr-picker", "--source", "snippets"]);
        assert_eq!(args.source, SourceArg::Snippets);
    }

    #[test]
    fn source_emoji_parses() {
        let args = Args::parse_from(["hypr-picker", "--source", "emoji"]);
        assert_eq!(args.source, SourceArg::Emoji);
    }

    #[test]
    fn popup_config_reflects_source() {
        let args = Args::parse_from(["hypr-picker", "--source", "emoji"]);
        let filter = PickerFilter::from_str(&args.filter).unwrap_or(PickerFilter::All);
        let cfg = PopupConfig {
            layer_shell: !args.xdg_window,
            source: args.source.into(),
            filter,
            query: args.query,
            action: args.action.into(),
            count: args.count,
            include_sensitive: args.include_sensitive,
        };
        assert_eq!(cfg.source, PickerSource::Emoji);
        assert_eq!(cfg.action, PickerAction::Copy);
        assert!(!cfg.include_sensitive);
    }

    #[test]
    fn popup_config_reflects_action() {
        let args = Args::parse_from(["hypr-picker", "--action", "quick-paste"]);
        let filter = PickerFilter::from_str(&args.filter).unwrap_or(PickerFilter::All);
        let cfg = PopupConfig {
            layer_shell: !args.xdg_window,
            source: args.source.into(),
            filter,
            query: args.query,
            action: args.action.into(),
            count: args.count,
            include_sensitive: args.include_sensitive,
        };
        assert_eq!(cfg.action, PickerAction::QuickPaste);
    }

    #[test]
    fn popup_config_reflects_include_sensitive() {
        let args = Args::parse_from(["hypr-picker", "--include-sensitive"]);
        let filter = PickerFilter::from_str(&args.filter).unwrap_or(PickerFilter::All);
        let cfg = PopupConfig {
            layer_shell: !args.xdg_window,
            source: args.source.into(),
            filter,
            query: args.query,
            action: args.action.into(),
            count: args.count,
            include_sensitive: args.include_sensitive,
        };
        assert!(cfg.include_sensitive);
    }

    #[test]
    fn popup_config_reflects_query() {
        let args = Args::parse_from(["hypr-picker", "--query", "test"]);
        let filter = PickerFilter::from_str(&args.filter).unwrap_or(PickerFilter::All);
        let cfg = PopupConfig {
            layer_shell: !args.xdg_window,
            source: args.source.into(),
            filter,
            query: args.query,
            action: args.action.into(),
            count: args.count,
            include_sensitive: args.include_sensitive,
        };
        assert_eq!(cfg.query.as_deref(), Some("test"));
    }

    #[test]
    fn source_arg_roundtrip() {
        for src in [
            SourceArg::History,
            SourceArg::Snippets,
            SourceArg::Emoji,
            SourceArg::Symbols,
            SourceArg::Kaomoji,
            SourceArg::All,
        ] {
            let picker: PickerSource = src.into();
            assert!(matches!(
                (&src, picker),
                (SourceArg::History, PickerSource::History)
                    | (SourceArg::Snippets, PickerSource::Snippets)
                    | (SourceArg::Emoji, PickerSource::Emoji)
                    | (SourceArg::Symbols, PickerSource::Symbols)
                    | (SourceArg::Kaomoji, PickerSource::Kaomoji)
                    | (SourceArg::All, PickerSource::All)
            ));
        }
    }

    #[test]
    fn action_arg_roundtrip() {
        for act in [ActionArg::Copy, ActionArg::QuickPaste] {
            let picker: PickerAction = act.into();
            assert!(matches!(
                (act, picker),
                (ActionArg::Copy, PickerAction::Copy)
                    | (ActionArg::QuickPaste, PickerAction::QuickPaste)
            ));
        }
    }
}
