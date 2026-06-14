//! Unified GTK4 UI state machine.
//!
//! `AppState`, `Action`, `Effect`, and a pure `reduce()` function.
//! GTK-free — no `gtk::init`, no `glib::Property`, no async, no I/O.

use std::fmt::{self, Display};
use std::str::FromStr;

use author_clipboard_shared::ipc::CopyMode;
use author_clipboard_shared::types::{ClipboardItem, Snippet};
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
#[derive(Debug, Clone)]
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
    /// Whether redacted content is currently shown (Ctrl+Shift+R).
    pub show_redacted: bool,
    /// Countdown in seconds for the sensitive reveal timer (0 = not active).
    pub reveal_countdown: u8,
    /// Whether the daemon is reachable (updated via `SetDaemonRunning`).
    pub daemon_running: bool,
    /// Whether incognito mode is active (sentinel file check in PR 4, not here).
    pub incognito: bool,
    /// In-memory clipboard item cache (filled by `ItemsLoaded`).
    pub items: Vec<ClipboardItem>,
    /// In-memory snippets cache (filled by `SnippetsLoaded`).
    pub snippets: Vec<Snippet>,
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
            show_redacted: false,
            reveal_countdown: 0,
            daemon_running: true,
            incognito: false,
            items: Vec::new(),
            snippets: Vec::new(),
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

/// All possible state-machine actions.
///
/// Variants added in PR 3B: pin/star/delete/reveal/window/settings/snippets/daemon.
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
    // ── PR 3B variants ──────────────────────────────────────────────
    /// Request a copy of the item at the given index in `items`.
    CopyRequested,
    /// Request a quick-paste of the item at the given index.
    QuickPasteRequested,
    /// Toggle the pinned flag on an item by its database id.
    TogglePin(i64),
    /// Toggle the starred flag on an item by its database id.
    ToggleStar(i64),
    /// Delete an item by its database id.
    Delete(i64),
    /// User activated the "reveal redacted" action.
    RevealRedacted,
    /// Hide the redacted content again.
    HideRedacted,
    /// Tick the reveal countdown by one second.
    RevealTick,
    /// Daemon reachability changed.
    SetDaemonRunning(bool),
    /// IPC returned a fresh item list (replace `state.items`).
    ItemsLoaded(Vec<ClipboardItem>),
    /// IPC returned a fresh snippets list (replace `state.snippets`).
    SnippetsLoaded(Vec<Snippet>),
    /// Show a toast message to the user.
    Toast(String),
    /// Quit the application.
    Quit,
    /// Toggle incognito mode.
    IncognitoToggled(bool),
    /// Window was resized (in pixels).
    WindowResized(i32, i32),
    /// User navigated to a page via Ctrl+Tab.
    WindowPageChanged(PageId),
}

/// Side-effects emitted by [`reduce`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Effect {
    /// Re-fetch items from the database with current filter + query.
    RefreshItems,
    /// Persist changed settings to `GSettings`.
    PersistGSettings,
    // ── PR 3B variants ──────────────────────────────────────────────
    /// Copy an item to the clipboard.
    CopyItem {
        id: i64,
        mode: CopyMode,
        mime: Option<String>,
    },
    /// Quick-paste an item.
    QuickPasteItem { id: i64, mime: Option<String> },
    /// Pin an item.
    PinItem(i64),
    /// Unpin an item.
    UnpinItem(i64),
    /// Star an item.
    StarItem(i64),
    /// Unstar an item.
    UnstarItem(i64),
    /// Delete an item.
    DeleteItem(i64),
    /// Clear all unpinned items.
    ClearUnpinned,
    /// Refresh the snippets list from DB.
    RefreshSnippets,
    /// Show a toast overlay message.
    AddToast(String),
    /// Persist current config to `GSettings`.
    PersistConfig,
    /// Quit the application.
    Quit,
    /// Hide redacted content.
    HideRedacted,
}

/// Pure state reducer. Deterministic — no I/O, no async, no `GLib`.
///
/// # Panics
///
/// Does not panic. `MoveBy` / `MovePage` on an empty selection is a no-op.
#[allow(clippy::too_many_lines)]
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

        // ── PR 3B arms ──────────────────────────────────────────────
        Action::CopyRequested => {
            let Some(idx) = state.selected_index else {
                return vec![];
            };
            if idx >= state.items.len() {
                return vec![];
            }
            let id = state.items[idx].id;
            let mime = if state.items[idx].mime_type == "text/plain" {
                None
            } else {
                Some(state.items[idx].mime_type.clone())
            };
            vec![Effect::CopyItem {
                id,
                mode: CopyMode::Copy,
                mime,
            }]
        }

        Action::QuickPasteRequested => {
            let Some(idx) = state.selected_index else {
                return vec![];
            };
            if idx >= state.items.len() {
                return vec![];
            }
            let id = state.items[idx].id;
            let mime = if state.items[idx].mime_type == "text/plain" {
                None
            } else {
                Some(state.items[idx].mime_type.clone())
            };
            vec![Effect::QuickPasteItem { id, mime }]
        }

        Action::TogglePin(id) => {
            let item = state.items.iter_mut().find(|item| item.id == id);
            match item {
                Some(item) => {
                    item.pinned = !item.pinned;
                    if item.pinned {
                        vec![Effect::PinItem(id)]
                    } else {
                        vec![Effect::UnpinItem(id)]
                    }
                }
                None => vec![],
            }
        }

        Action::ToggleStar(id) => {
            let item = state.items.iter_mut().find(|item| item.id == id);
            match item {
                Some(item) => {
                    item.starred = !item.starred;
                    if item.starred {
                        vec![Effect::StarItem(id)]
                    } else {
                        vec![Effect::UnstarItem(id)]
                    }
                }
                None => vec![],
            }
        }

        Action::Delete(id) => {
            let pos = state.items.iter().position(|item| item.id == id);
            match pos {
                Some(idx) => {
                    state.items.remove(idx);
                    vec![Effect::DeleteItem(id)]
                }
                None => vec![],
            }
        }

        Action::RevealRedacted => {
            state.show_redacted = true;
            state.reveal_countdown = 5;
            vec![Effect::PersistConfig]
        }

        Action::HideRedacted => {
            state.show_redacted = false;
            state.reveal_countdown = 0;
            vec![]
        }

        Action::RevealTick => {
            let was_active = state.reveal_countdown > 0;
            if state.reveal_countdown > 0 {
                state.reveal_countdown -= 1;
            }
            if was_active && state.reveal_countdown == 0 {
                state.show_redacted = false;
                return vec![Effect::HideRedacted];
            }
            vec![]
        }

        Action::SetDaemonRunning(b) => {
            state.daemon_running = b;
            vec![]
        }

        Action::ItemsLoaded(items) => {
            state.items = items;
            vec![]
        }

        Action::SnippetsLoaded(snippets) => {
            state.snippets = snippets;
            vec![Effect::RefreshSnippets]
        }

        Action::Toast(msg) => {
            vec![Effect::AddToast(msg)]
        }

        Action::Quit => {
            vec![Effect::Quit]
        }

        Action::IncognitoToggled(b) => {
            state.incognito = b;
            vec![]
        }

        Action::WindowResized(_w, _h) => {
            // Debouncing belongs in the runtime — reducer just records dimensions.
            vec![]
        }

        Action::WindowPageChanged(p) => {
            state.active_page = p;
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

        // Both should clear search_query and selected_index.
        assert_eq!(s1.search_query, s2.search_query);
        assert_eq!(s1.selected_index, s2.selected_index);
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

    // ── CopyRequested / QuickPasteRequested ──────────────────────────

    fn make_item(id: i64) -> ClipboardItem {
        ClipboardItem {
            id,
            content_hash: 0,
            content: "test".into(),
            mime_type: "text/plain".into(),
            content_type: author_clipboard_shared::types::ContentType::Text,
            timestamp: chrono::Utc::now(),
            pinned: false,
            starred: false,
            source_app: None,
            sensitive: false,
            plain_text: None,
            encrypted: false,
            encryption_version: None,
            redacted_preview: None,
        }
    }

    #[test]
    fn copy_requested_emits_copy_item_effect() {
        let mut state = fresh_state();
        state.items = vec![make_item(42), make_item(99)];
        state.selected_index = Some(0);
        let effects = reduce(&mut state, Action::CopyRequested);
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0],
            Effect::CopyItem {
                id: 42,
                mode: CopyMode::Copy,
                mime: None
            }
        );
    }

    #[test]
    fn quick_paste_requested_emits_quick_paste_item_effect() {
        let mut state = fresh_state();
        state.items = vec![make_item(7)];
        state.selected_index = Some(0);
        let effects = reduce(&mut state, Action::QuickPasteRequested);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0], Effect::QuickPasteItem { id: 7, mime: None });
    }

    #[test]
    fn copy_requested_with_no_items_is_noop() {
        let mut state = fresh_state();
        state.selected_index = Some(0);
        let effects = reduce(&mut state, Action::CopyRequested);
        assert!(effects.is_empty());
    }

    // ── TogglePin / ToggleStar ───────────────────────────────────────

    #[test]
    fn toggle_pin_pinned_item_emits_unpin_item() {
        let mut state = fresh_state();
        let mut item = make_item(5);
        item.pinned = true;
        state.items = vec![item];
        let effects = reduce(&mut state, Action::TogglePin(5));
        assert_eq!(effects, vec![Effect::UnpinItem(5)]);
        assert!(!state.items[0].pinned);
    }

    #[test]
    fn toggle_pin_unpinned_item_emits_pin_item() {
        let mut state = fresh_state();
        state.items = vec![make_item(5)];
        let effects = reduce(&mut state, Action::TogglePin(5));
        assert_eq!(effects, vec![Effect::PinItem(5)]);
        assert!(state.items[0].pinned);
    }

    #[test]
    fn toggle_star_starred_item_emits_unstar_item() {
        let mut state = fresh_state();
        let mut item = make_item(3);
        item.starred = true;
        state.items = vec![item];
        let effects = reduce(&mut state, Action::ToggleStar(3));
        assert_eq!(effects, vec![Effect::UnstarItem(3)]);
        assert!(!state.items[0].starred);
    }

    #[test]
    fn toggle_pin_with_unknown_id_is_noop() {
        let mut state = fresh_state();
        state.items = vec![make_item(1), make_item(2)];
        let effects = reduce(&mut state, Action::TogglePin(999));
        assert!(effects.is_empty());
    }

    // ── Delete ───────────────────────────────────────────────────────

    #[test]
    fn delete_item_removes_from_items_and_emits_delete_item() {
        let mut state = fresh_state();
        state.items = vec![make_item(1), make_item(2), make_item(3)];
        state.selected_index = Some(1);
        let effects = reduce(&mut state, Action::Delete(2));
        assert_eq!(effects, vec![Effect::DeleteItem(2)]);
        assert_eq!(state.items.len(), 2);
        assert_eq!(state.items[0].id, 1);
        assert_eq!(state.items[1].id, 3);
    }

    #[test]
    fn delete_item_with_unknown_id_is_noop() {
        let mut state = fresh_state();
        state.items = vec![make_item(1), make_item(2)];
        let effects = reduce(&mut state, Action::Delete(999));
        assert!(effects.is_empty());
        assert_eq!(state.items.len(), 2);
    }

    // ── RevealRedacted / HideRedacted / RevealTick ───────────────────

    #[test]
    fn reveal_redacted_sets_show_redacted_and_countdown() {
        let mut state = fresh_state();
        assert!(!state.show_redacted);
        assert_eq!(state.reveal_countdown, 0);
        let effects = reduce(&mut state, Action::RevealRedacted);
        assert!(state.show_redacted);
        assert_eq!(state.reveal_countdown, 5);
        assert!(effects.contains(&Effect::PersistConfig));
    }

    #[test]
    fn hide_redacted_resets_show_redacted_and_countdown() {
        let mut state = fresh_state();
        state.show_redacted = true;
        state.reveal_countdown = 3;
        let effects = reduce(&mut state, Action::HideRedacted);
        assert!(!state.show_redacted);
        assert_eq!(state.reveal_countdown, 0);
        assert!(effects.is_empty());
    }

    #[test]
    fn reveal_tick_decrements_countdown() {
        let mut state = fresh_state();
        state.show_redacted = true;
        state.reveal_countdown = 3;
        let effects = reduce(&mut state, Action::RevealTick);
        assert_eq!(state.reveal_countdown, 2);
        assert!(state.show_redacted);
        assert!(effects.is_empty());
    }

    #[test]
    fn reveal_tick_emits_hide_redacted_at_zero() {
        let mut state = fresh_state();
        state.show_redacted = true;
        state.reveal_countdown = 1;
        let effects = reduce(&mut state, Action::RevealTick);
        assert_eq!(state.reveal_countdown, 0);
        assert!(!state.show_redacted);
        assert!(effects.contains(&Effect::HideRedacted));
    }

    #[test]
    fn reveal_tick_at_zero_does_not_underflow() {
        let mut state = fresh_state();
        state.show_redacted = false;
        state.reveal_countdown = 0;
        let effects = reduce(&mut state, Action::RevealTick);
        assert_eq!(state.reveal_countdown, 0);
        assert!(!state.show_redacted);
        assert!(effects.is_empty());
    }

    #[test]
    fn reveal_tick_leaves_show_redacted_true_until_zero() {
        let mut state = fresh_state();
        state.show_redacted = true;
        state.reveal_countdown = 1;
        reduce(&mut state, Action::RevealTick);
        assert!(!state.show_redacted); // countdown hit 0 → set false
        assert_eq!(state.reveal_countdown, 0);
    }

    // ── SetDaemonRunning / IncognitoToggled ──────────────────────────

    #[test]
    fn set_daemon_running_updates_state() {
        let mut state = fresh_state();
        assert!(state.daemon_running); // default
        reduce(&mut state, Action::SetDaemonRunning(false));
        assert!(!state.daemon_running);
        reduce(&mut state, Action::SetDaemonRunning(true));
        assert!(state.daemon_running);
    }

    #[test]
    fn incognito_toggled_flips_state() {
        let mut state = fresh_state();
        assert!(!state.incognito);
        reduce(&mut state, Action::IncognitoToggled(true));
        assert!(state.incognito);
        reduce(&mut state, Action::IncognitoToggled(false));
        assert!(!state.incognito);
    }

    // ── ItemsLoaded / SnippetsLoaded ─────────────────────────────────

    #[test]
    fn items_loaded_replaces_state_items() {
        let mut state = fresh_state();
        state.items = vec![make_item(1)];
        let new_items = vec![make_item(10), make_item(20)];
        reduce(&mut state, Action::ItemsLoaded(new_items.clone()));
        assert_eq!(state.items.len(), new_items.len());
        assert_eq!(state.items[0].id, 10);
        assert_eq!(state.items[1].id, 20);
    }

    #[test]
    fn snippets_loaded_replaces_state_snippets_and_emits_refresh_snippets() {
        let mut state = fresh_state();
        let snippet = Snippet {
            id: 1,
            name: "test".into(),
            content: "hello".into(),
            updated_at: chrono::Utc::now(),
        };
        let effects = reduce(&mut state, Action::SnippetsLoaded(vec![snippet.clone()]));
        assert_eq!(state.snippets.len(), 1);
        assert_eq!(state.snippets[0].id, 1);
        assert!(effects.contains(&Effect::RefreshSnippets));
    }

    #[test]
    fn items_loaded_with_empty_vec_clears_items() {
        let mut state = fresh_state();
        state.items = vec![make_item(1), make_item(2)];
        reduce(&mut state, Action::ItemsLoaded(vec![]));
        assert!(state.items.is_empty());
    }

    // ── Toast / Quit ─────────────────────────────────────────────────

    #[test]
    fn toast_action_emits_add_toast_effect() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::Toast("Hello world".into()));
        assert_eq!(effects, vec![Effect::AddToast("Hello world".into())]);
    }

    #[test]
    fn quit_action_emits_quit_effect() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::Quit);
        assert_eq!(effects, vec![Effect::Quit]);
    }

    // ── WindowResized / WindowPageChanged ────────────────────────────

    #[test]
    fn window_resized_does_not_emit_any_effect() {
        let mut state = fresh_state();
        let effects = reduce(&mut state, Action::WindowResized(800, 600));
        assert!(effects.is_empty());
        // State fields w/h are not stored, so just verify no effect.
        let effects2 = reduce(&mut state, Action::WindowResized(1920, 1080));
        assert!(effects2.is_empty());
    }

    #[test]
    fn window_page_changed_updates_active_page() {
        let mut state = fresh_state();
        reduce(&mut state, Action::WindowPageChanged(PageId::Snippets));
        assert_eq!(state.active_page, PageId::Snippets);
        reduce(&mut state, Action::WindowPageChanged(PageId::Settings));
        assert_eq!(state.active_page, PageId::Settings);
    }

    // ── ShowRedacted / Incognito / Daemon defaults ───────────────────

    #[test]
    fn default_state_show_redacted_is_false() {
        let state = fresh_state();
        assert!(!state.show_redacted);
    }

    #[test]
    fn default_state_daemon_running_is_true() {
        let state = fresh_state();
        assert!(state.daemon_running);
    }

    // ── MoveBy after ItemsLoaded ─────────────────────────────────────

    #[test]
    fn move_by_works_when_items_are_loaded() {
        let mut state = fresh_state();
        state.items = vec![make_item(1), make_item(2), make_item(3)];
        state.selected_index = Some(1);
        let effects = reduce(&mut state, Action::MoveBy(1));
        assert_eq!(state.selected_index, Some(2));
        assert!(effects.is_empty());
    }
}
