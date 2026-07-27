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
//!   input has focus. The global Esc controller runs in `Bubble`
//!   phase and is notified before widget defaults. Text inputs
//!   are handled via `FocusTarget::TextInput` resolution so that
//!   in-field Esc clears/leaves the field first.
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
pub mod service;
pub mod settings;
pub mod theme;
pub mod widgets;
pub mod window;

/// Configuration passed to [`run_popup`].
#[derive(Debug, Clone, PartialEq)]
pub struct PopupConfig {
    /// Use layer-shell overlay mode instead of a normal resizable window.
    pub layer_shell: bool,
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
            layer_shell: true,
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
#[derive(Debug, Clone, PartialEq)]
pub struct ManagerConfig {
    /// Optional deep-link to a specific page by `PageId` name.
    /// Supports all page IDs: clipboard, home, collections, emoji,
    /// symbols, kaomoji, snippets, settings.
    pub initial_page: Option<crate::app::PageId>,
    /// Initial source for the clipboard/history page.
    pub clipboard_source: PickerSource,
    /// Initial filter chip for the clipboard/history page.
    pub clipboard_filter: PickerFilter,
    /// Pre-fill search query for the clipboard/history page.
    pub clipboard_query: Option<String>,
    /// Action on Enter for the clipboard/history page.
    pub clipboard_action: PickerAction,
    /// Maximum items to load on the clipboard/history page.
    pub clipboard_count: usize,
    /// Include sensitive items on the clipboard/history page.
    pub clipboard_include_sensitive: bool,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            initial_page: None,
            clipboard_source: PickerSource::History,
            clipboard_filter: PickerFilter::All,
            clipboard_query: None,
            clipboard_action: PickerAction::Copy,
            clipboard_count: 50,
            clipboard_include_sensitive: false,
        }
    }
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
