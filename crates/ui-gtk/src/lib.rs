//! Unified GTK4 + libadwaita UI library for author-clipboard.
//!
//! This crate is the **single source of truth** for every widget,
//! state machine, and visual token. Two binaries (`applet`,
//! `hypr-picker`) call into [`run_popup`] and [`run_manager`]; the
//! external `wofi/rofi/fuzzel` picker in `ctl` is independent.
//!
//! Bug fixes baked in:
//!
//! * US-001 — Esc always closes the popup, even when the search
//!   input has focus. The Esc controller runs in `Capture` phase so
//!   it wins over the widget's built-in handler.
//! * US-002 — Popup opens with the **list** focused, not the search
//!   input. `/` focuses the search; `Esc` clears it.
//! * US-003 — The manager window is a real `AdwApplicationWindow`
//!   with headerbar, sidebar, and preferences page.

#![warn(missing_docs)]

pub mod actions;
pub mod app;
pub mod controller;
pub mod model;
pub mod pages;
pub mod settings;
pub mod theme;
pub mod widgets;
pub mod window;

/// Configuration passed to [`run_popup`].
#[derive(Debug, Clone, PartialEq)]
pub struct PopupConfig {
    /// Initial source for the picker.
    pub source: PickerSource,
    /// Initial filter chip.
    pub filter: PickerFilter,
    /// Pre-fill search query.
    pub query: Option<String>,
    /// Action on Enter.
    pub action: PickerAction,
    /// Maximum items to load.
    pub count: usize,
    /// Include sensitive items.
    pub include_sensitive: bool,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            source: PickerSource::History,
            filter: PickerFilter::All,
            query: None,
            action: PickerAction::Copy,
            count: 50,
            include_sensitive: false,
        }
    }
}

/// Configuration passed to [`run_manager`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ManagerConfig {
    /// Optional deep-link to a specific page.
    pub initial_page: Option<PickerSource>,
}

/// Run the GTK4 layer-shell popup. Blocks until the popup closes.
///
/// Used by the `author-clipboard --popup` and
/// `author-clipboard-hypr-picker` binaries.
pub fn run_popup(config: PopupConfig) -> anyhow::Result<()> {
    crate::window::popup::run(config)
}

/// Run the GTK4 XDG manager window. Blocks until the window closes.
///
/// Used by the `author-clipboard --manager` binary and the
/// `.desktop` file launcher.
pub fn run_manager(config: ManagerConfig) -> anyhow::Result<()> {
    crate::window::manager::run(config)
}

// Re-export shared types so binary crates don't need to depend on
// `shared` directly.
pub use author_clipboard_shared::picker::{PickerAction, PickerFilter, PickerSource};

// Re-export the state machine surface.
pub use crate::app::{reduce, Action, AppMode, AppState, Effect, FocusTarget, PageId, SortOrder};
