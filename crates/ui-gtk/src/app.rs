//! Unified GTK4 UI state machine.
//!
//! `AppState`, `Action`, `Effect`, and a pure `reduce()` function.
//! GTK-free — no `gtk::init`, no `glib::Property`, no async, no I/O.

use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{ManagerConfig, PopupConfig};

/// Pages in the navigation sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageId {
    /// Clipboard history page.
    #[default]
    Clipboard,
    /// Emoji picker page.
    Emoji,
    /// Symbol picker page.
    Symbols,
    /// Kaomoji page.
    Kaomoji,
    /// Text snippets page.
    Snippets,
    /// Settings page.
    Settings,
}

impl PageId {
    /// All known pages, in navigation order.
    pub const ALL: &'static [PageId] = &[
        PageId::Clipboard,
        PageId::Emoji,
        PageId::Symbols,
        PageId::Kaomoji,
        PageId::Snippets,
        PageId::Settings,
    ];
}

impl Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PageId::Clipboard => "clipboard",
            PageId::Emoji => "emoji",
            PageId::Symbols => "symbols",
            PageId::Kaomoji => "kaomoji",
            PageId::Snippets => "snippets",
            PageId::Settings => "settings",
        };
        write!(f, "{s}")
    }
}

impl FromStr for PageId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clipboard" => Ok(PageId::Clipboard),
            "emoji" => Ok(PageId::Emoji),
            "symbols" => Ok(PageId::Symbols),
            "kaomoji" => Ok(PageId::Kaomoji),
            "snippets" => Ok(PageId::Snippets),
            "settings" => Ok(PageId::Settings),
            other => Err(format!("unknown PageId: {other}")),
        }
    }
}

/// Whether the app is running as a popup or a full manager window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Popup overlay (layer-shell).
    #[default]
    Popup,
    /// Full manager window (xdg-shell).
    Manager,
}

/// Where keyboard focus is currently directed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusTarget {
    /// The main item list has focus. Enter = copy. Esc = close.
    #[default]
    List,
    /// The search entry has focus. Esc clears or blurs, not closes.
    Search,
    /// A modal dialog (shortcuts overlay, error) has focus.
    Modal,
    /// No widget has focus (e.g. before the window is mapped).
    None,
}

/// The top-level application state.
///
/// Plain Rust struct — **no `glib::Properties`**. That comes in PR 4
/// when `GSettings` bindings are wired up.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    /// `Popup` or `Manager`.
    pub mode: AppMode,
    /// Which navigation page is active.
    pub active_page: PageId,
    /// Active filter chip (All / Text / Images / …).
    pub filter: crate::PickerFilter,
    /// Sort order for the list.
    pub sort: SortOrder,
    /// Live search query string.
    pub search_query: String,
    /// Selected row index in the current page's item list.
    pub selected_index: Option<usize>,
    /// Current focus target.
    pub focus: FocusTarget,
    /// Popup-specific configuration (source, filter, action, …).
    pub config: PopupConfig,
    /// Manager-specific configuration.
    pub manager_config: ManagerConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Popup,
            active_page: PageId::default(),
            filter: crate::PickerFilter::All,
            sort: SortOrder::NewestFirst,
            search_query: String::new(),
            selected_index: None,
            focus: FocusTarget::default(),
            config: PopupConfig::default(),
            manager_config: ManagerConfig::default(),
        }
    }
}

/// Sort order for the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Most recent first. Default.
    NewestFirst,
    /// Oldest first.
    OldestFirst,
    /// Most frequently used first. (Not yet implemented.)
    MostUsed,
}

impl SortOrder {
    /// Short label for chip text.
    pub fn label(self) -> &'static str {
        match self {
            Self::NewestFirst => "Newest",
            Self::OldestFirst => "Oldest",
            Self::MostUsed => "Most used",
        }
    }
}

/// All possible state-machine actions (foundation slice — PR 3A).
///
/// Variants added in later PRs: pin/star/delete/reveal/window/snippets/daemon.
#[derive(Debug, Clone)]
pub enum Action {
    /// User typed in the search box.
    QueryChanged(String),
    /// User pressed Esc / clicked the clear button.
    QueryCleared,
    /// User selected a different filter chip.
    FilterChanged(crate::PickerFilter),
    /// User clicked a navigation page.
    PageChanged(PageId),
    /// Cycle through pages by `n` steps (+1 = forward, -1 = backward).
    CyclePage(i32),
    /// Move keyboard focus to a widget.
    Focus(FocusTarget),
    /// Select a row by its database id.
    ///
    /// The reducer maps this to a `selected_index` via a lookup placeholder;
    /// the runtime fills in the real lookup in PR 3B.
    Select(Option<u32>),
    /// Move selection by `d` rows (negative = up, positive = down).
    MoveBy(i32),
    /// Jump to an absolute row index.
    MoveTo(usize),
    /// Move by one "page" worth of rows.
    MovePage(i32),
    /// `GSettings` (or config file) has loaded new `PopupConfig`.
    ConfigLoaded(PopupConfig),
    /// `GSettings` has loaded new `ManagerConfig`.
    ManagerConfigLoaded(ManagerConfig),
}

/// Side-effects emitted by [`reduce`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Re-fetch items from the database with current filter + query.
    RefreshItems,
    /// Persist changed settings to `GSettings`.
    PersistGSettings,
}

/// Pure state reducer. Deterministic — no I/O, no async, no `GLib`.
///
/// # Panics
///
/// Does not panic. `MoveBy` / `MovePage` on an empty selection is a no-op.
pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::QueryChanged(s) => {
            if s.is_empty() {
                state.search_query = String::new();
                state.selected_index = None;
                vec![Effect::RefreshItems]
            } else {
                state.search_query = s;
                vec![Effect::RefreshItems]
            }
        }

        Action::QueryCleared => {
            state.search_query = String::new();
            state.selected_index = None;
            vec![Effect::RefreshItems]
        }

        Action::FilterChanged(f) => {
            state.filter = f;
            state.selected_index = None;
            vec![Effect::RefreshItems, Effect::PersistGSettings]
        }

        Action::PageChanged(p) => {
            state.active_page = p;
            state.selected_index = None;
            vec![Effect::PersistGSettings]
        }

        Action::CyclePage(n) => {
            let len = PageId::ALL.len();
            if len == 0 {
                return vec![];
            }
            let current = PageId::ALL
                .iter()
                .position(|&p| p == state.active_page)
                .unwrap_or(0);
            // Signed modulo: ((n % len) + current + len) % len, using i32 throughout.
            let len_i32 = i32::try_from(len).unwrap_or(i32::MAX);
            let new_idx = ((n.rem_euclid(len_i32) as usize) + current) % len;
            state.active_page = PageId::ALL[new_idx];
            vec![Effect::PersistGSettings]
        }

        Action::Focus(t) => {
            state.focus = t;
            vec![]
        }

        Action::Select(Some(_)) => {
            // Placeholder: PR 3B fills in the real id→index lookup.
            // For the foundation, any non-None id maps to index 0.
            state.selected_index = Some(0);
            vec![]
        }

        Action::Select(None) => {
            state.selected_index = None;
            vec![]
        }

        Action::MoveBy(d) => {
            let Some(i) = state.selected_index else {
                // Empty selection — no-op, no panic.
                return vec![];
            };
            let i_i32 = i32::try_from(i).unwrap_or(i32::MAX);
            let new_i = i_i32.saturating_add(d);
            if new_i < 0 {
                state.selected_index = Some(0);
            } else {
                state.selected_index = Some(usize::try_from(new_i).unwrap_or(usize::MAX));
            }
            vec![]
        }

        Action::MoveTo(i) => {
            state.selected_index = Some(i);
            vec![]
        }

        Action::MovePage(d) => {
            // One page = 10 rows.
            const PAGE_SIZE: i32 = 10;
            let Some(i) = state.selected_index else {
                return vec![];
            };
            let i_i32 = i32::try_from(i).unwrap_or(i32::MAX);
            let new_i = i_i32.saturating_add(d * PAGE_SIZE);
            if new_i < 0 {
                state.selected_index = Some(0);
            } else {
                state.selected_index = Some(usize::try_from(new_i).unwrap_or(usize::MAX));
            }
            vec![]
        }

        Action::ConfigLoaded(c) => {
            state.config = c;
            vec![]
        }

        Action::ManagerConfigLoaded(c) => {
            state.manager_config = c;
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> AppState {
        AppState::default()
    }

    // ── QueryChanged ────────────────────────────────────────────────

    #[test]
    fn query_changed_updates_search_query_and_emits_refresh() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::QueryChanged("git".into()));
        assert_eq!(state.search_query, "git");
        assert_eq!(effects, vec![Effect::RefreshItems]);
    }

    #[test]
    fn query_changed_empty_clears_selection() {
        let mut state = fresh_state();
        state.selected_index = Some(5);
        reduce(&mut state, Action::QueryChanged(String::new()));
        assert_eq!(state.selected_index, None);
    }

    #[test]
    fn query_changed_empty_is_equivalent_to_query_cleared() {
        let mut s1 = fresh_state();
        let mut s2 = fresh_state();
        s1.selected_index = Some(3);
        s2.selected_index = Some(3);

        reduce(&mut s1, Action::QueryChanged(String::new()));
        reduce(&mut s2, Action::QueryCleared);

        assert_eq!(s1, s2);
    }

    // ── FilterChanged ───────────────────────────────────────────────

    #[test]
    fn filter_changed_persists_to_gsettings() {
        let mut state = fresh_state();
        let effects = reduce(
            &mut state,
            Action::FilterChanged(crate::PickerFilter::Pinned),
        );
        assert_eq!(state.filter, crate::PickerFilter::Pinned);
        assert!(effects.contains(&Effect::PersistGSettings));
        assert!(effects.contains(&Effect::RefreshItems));
    }

    #[test]
    fn filter_changed_clears_selection() {
        let mut state = fresh_state();
        state.selected_index = Some(7);
        reduce(&mut state, Action::FilterChanged(crate::PickerFilter::All));
        assert_eq!(state.selected_index, None);
    }

    // ── PageChanged ─────────────────────────────────────────────────

    #[test]
    fn page_changed_updates_active_page() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::PageChanged(PageId::Emoji));
        assert_eq!(state.active_page, PageId::Emoji);
        assert!(effects.contains(&Effect::PersistGSettings));
    }

    // ── CyclePage ───────────────────────────────────────────────────

    #[test]
    fn cycle_page_forward_wraps() {
        let mut state = fresh_state();
        state.active_page = PageId::Settings;
        let effects = reduce(&mut state, Action::CyclePage(1));
        assert_eq!(state.active_page, PageId::Clipboard);
        assert!(effects.contains(&Effect::PersistGSettings));
    }

    #[test]
    fn cycle_page_backward_wraps() {
        let mut state = fresh_state();
        state.active_page = PageId::Clipboard;
        let effects = reduce(&mut state, Action::CyclePage(-1));
        assert_eq!(state.active_page, PageId::Settings);
        assert!(effects.contains(&Effect::PersistGSettings));
    }

    #[test]
    fn cycle_page_zero_is_noop() {
        let mut state = fresh_state();
        state.active_page = PageId::Kaomoji;
        let effects = reduce(&mut state, Action::CyclePage(0));
        // Page doesn't change, but the action is still acknowledged with PersistGSettings.
        assert_eq!(state.active_page, PageId::Kaomoji);
        assert!(effects.contains(&Effect::PersistGSettings));
    }

    // ── Focus ───────────────────────────────────────────────────────

    #[test]
    fn focus_changes_focus_target() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::Focus(FocusTarget::Search));
        assert_eq!(state.focus, FocusTarget::Search);
        assert!(effects.is_empty());
    }

    // ── Select ──────────────────────────────────────────────────────

    #[test]
    fn select_some_sets_index() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::Select(Some(7)));
        assert_eq!(state.selected_index, Some(0)); // placeholder mapping
        assert!(effects.is_empty());
    }

    #[test]
    fn select_none_clears_index() {
        let mut state = fresh_state();
        state.selected_index = Some(4);
        let effects = reduce(&mut state, Action::Select(None));
        assert_eq!(state.selected_index, None);
        assert!(effects.is_empty());
    }

    // ── MoveBy ──────────────────────────────────────────────────────

    #[test]
    fn move_by_on_empty_selection_is_noop() {
        let mut state = fresh_state();
        state.selected_index = None;
        let effects = reduce(&mut state, Action::MoveBy(3));
        assert_eq!(state.selected_index, None);
        assert!(effects.is_empty());
    }

    #[test]
    fn move_to_sets_index() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::MoveTo(5));
        assert_eq!(state.selected_index, Some(5));
        assert!(effects.is_empty());
    }

    // ── ConfigLoaded ────────────────────────────────────────────────

    #[test]
    fn config_loaded_replaces_state_config() {
        let mut state = fresh_state();
        let new_config = PopupConfig {
            source: crate::PickerSource::Snippets,
            filter: crate::PickerFilter::Text,
            query: Some("hello".into()),
            action: crate::PickerAction::QuickPaste,
            count: 100,
            include_sensitive: true,
        };
        reduce(&mut state, Action::ConfigLoaded(new_config.clone()));
        assert_eq!(state.config, new_config);
    }
}
