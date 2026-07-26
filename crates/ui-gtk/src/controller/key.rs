//! Window-level key controller.
//!
//! Runs in the default (bubble) propagation phase so child widgets
//! (`SearchEntry`, `ListBox`, modal dialogs) handle their own keys first.
//! The controller only sees keys that no child widget consumed.
//!
//! Ownership rules:
//! - GTK `ListBox` owns Up/Down/Home/End/PageUp/PageDown/Enter.
//! - `SearchEntry2` owns first-Esc-clear and text input.
//! - This controller owns /, Ctrl+Enter, Ctrl+Tab, shortcuts overlay,
//!   organization shortcuts (Ctrl+P/Shift+P/Shift+S/Shift+A),
//!   and second-Esc-list-focus / popup-close.
//! - Quit is never emitted from the key controller — Esc close is
//!   driven by a callback so popup and manager behave differently.

use crate::app::{Action, AppState, FocusTarget};
use gtk4::gdk;
use gtk4::gdk::Key;
use gtk4::gdk::ModifierType;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::EventControllerKey;
use std::cell::RefCell;
use std::rc::Rc;

/// Map a key + modifiers to an action for the window-level controller.
///
/// Returns `None` for keys that should bubble to child widgets (navigation,
/// Enter, text input). The caller decides what to do with the action
/// (reduce state, focus widgets, close window).
pub fn map_window_key(key: Key, mods: ModifierType) -> Option<Action> {
    let ctrl = mods.contains(ModifierType::CONTROL_MASK);
    let shift = mods.contains(ModifierType::SHIFT_MASK);

    #[allow(clippy::match_same_arms)]
    match (key, ctrl, shift) {
        // Focus search
        (Key::slash, false, false) => Some(Action::Focus(FocusTarget::Search)),
        // ? shortcut (Shift+/) — same target on some layouts
        (Key::minus, false, true) => Some(Action::Focus(FocusTarget::Search)),
        // Shortcuts overlay
        (Key::question | Key::F1, false, false) => Some(Action::Focus(FocusTarget::Modal)),
        // Page navigation
        (Key::Tab, true, false) => Some(Action::CyclePage(1)),
        (Key::Tab, true, true) => Some(Action::CyclePage(-1)),
        // Organization shortcuts
        (Key::p, true, false) => Some(Action::ToggleSelectedPin),
        (Key::s, true, true) => Some(Action::ToggleSelectedStar),
        // Ctrl+1..9 quick pick — handled at page level via list_box.select_row()
        // (not mapped here since the window controller can't directly select ListBox rows).
        _ => None,
    }
}

/// Install the window-level key controller.
///
/// `close_window` is called when Esc should close the popup (not the manager).
/// The controller runs in the default (bubble) phase, so child widgets
/// (`SearchEntry`, `ListBox`, dialogs) process their own keys first.
///
/// # Parameters
/// * `widget` — the top-level window or parent widget.
/// * `state` — shared `AppState`.
/// * `effects_tx` — channel to send effects.
/// * `close_window` — closure invoked to close the window (popup only).
/// * `search_entry` — optional reference to the search entry for focus.
/// * `list_box` — optional reference to the list box for focus-on-Esc.
pub fn install(
    widget: &impl IsA<gtk4::Widget>,
    state: &Rc<RefCell<AppState>>,
    effects_tx: &std::sync::mpsc::Sender<crate::Effect>,
    close_window: Option<Box<dyn Fn() + 'static>>,
    search_entry: Option<&gtk4::SearchEntry>,
    list_box: Option<&gtk4::ListBox>,
) -> EventControllerKey {
    use crate::controller::focus::resolve_escape;
    use crate::controller::focus::EscOutcome;

    let controller = EventControllerKey::new();
    let st = state.clone();
    let tx = effects_tx.clone();

    // Keep handles to search and list for focus management.
    let search = search_entry.cloned();
    let list = list_box.cloned();

    controller.connect_key_pressed(move |_, key, _keycode, mods| {
        // ── Esc handling ──────────────────────────────────────────
        if key == gdk::Key::Escape {
            let s = st.borrow();
            let outcome = resolve_escape(s.focus, s.search_query.is_empty());
            drop(s);

            // If Esc was handled by a child widget (SearchEntry cleared
            // text and stopped propagation), we don't see it here in bubble
            // phase. So we always handle it: when search text is non-empty,
            // clear search. When search text is empty and search has focus,
            // focus list. When list has focus, close popup.
            match outcome {
                EscOutcome::ClearSearch => {
                    // SearchEntry2 already handles this and stops propagation,
                    // so we should only reach this branch if the search widget
                    // didn't consume it (defensive: still handle it here).
                    if let Some(ref entry) = search {
                        entry.set_text("");
                        st.borrow_mut().search_query = String::new();
                        let _ = tx.send(crate::Effect::RefreshItems);
                    }
                    Propagation::Stop
                }
                EscOutcome::BlurSearch => {
                    // Search text is empty; transfer focus to the list.
                    if let Some(ref lb) = list {
                        lb.grab_focus();
                    }
                    st.borrow_mut().focus = FocusTarget::List;
                    Propagation::Stop
                }
                EscOutcome::Close => {
                    // List has focus; close the popup (or no-op in manager).
                    if let Some(ref close) = close_window {
                        close();
                    }
                    Propagation::Stop
                }
                EscOutcome::Proceed => {
                    // Modal or no focus: let dialog handle its own Esc.
                    Propagation::Proceed
                }
            }
        }
        // ── Org shortcuts (Ctrl+Tab, Ctrl+P, etc.) ───────────────
        else if let Some(action) = map_window_key(key, mods) {
            // `/` focuses the search entry — handle before moving action.
            if matches!(action, Action::Focus(FocusTarget::Search)) {
                let mut s = st.borrow_mut();
                s.focus = FocusTarget::Search;
                drop(s);
                if let Some(ref entry) = search {
                    entry.grab_focus();
                }
                return Propagation::Stop;
            }
            // All other actions go through the reducer.
            let is_cycle = matches!(action, Action::CyclePage(_));
            let mut s = st.borrow_mut();
            let effects = crate::app::reduce(&mut s, action);
            drop(s);
            for eff in effects {
                let _ = tx.send(eff);
            }
            // Ctrl+Tab continues to page, others stop here.
            if is_cycle {
                Propagation::Proceed
            } else {
                Propagation::Stop
            }
        } else {
            Propagation::Proceed
        }
    });

    widget.add_controller(controller.clone());
    controller
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::gdk::Key;
    use gtk4::gdk::ModifierType;

    #[test]
    fn map_window_key_slash_returns_focus_search() {
        assert_eq!(
            map_window_key(Key::slash, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::Search))
        );
    }

    #[test]
    fn map_window_key_ctrl_tab_returns_cycle_page_1() {
        assert_eq!(
            map_window_key(Key::Tab, ModifierType::CONTROL_MASK),
            Some(Action::CyclePage(1))
        );
    }

    #[test]
    fn map_window_key_ctrl_shift_tab_returns_cycle_page_neg_1() {
        assert_eq!(
            map_window_key(
                Key::Tab,
                ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
            ),
            Some(Action::CyclePage(-1))
        );
    }

    #[test]
    fn map_window_key_question_opens_shortcuts() {
        assert_eq!(
            map_window_key(Key::question, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::Modal))
        );
    }

    #[test]
    fn map_window_key_ctrl_p_returns_toggle_pin() {
        assert_eq!(
            map_window_key(Key::p, ModifierType::CONTROL_MASK),
            Some(Action::ToggleSelectedPin)
        );
    }

    #[test]
    fn map_window_key_ctrl_shift_s_returns_toggle_star() {
        assert_eq!(
            map_window_key(
                Key::s,
                ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
            ),
            Some(Action::ToggleSelectedStar)
        );
    }

    #[test]
    fn map_window_key_navigation_keys_return_none() {
        // Navigation is handled by ListBox, not the window controller.
        assert_eq!(map_window_key(Key::Up, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::Down, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::Home, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::End, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::Page_Up, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::Page_Down, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::Return, ModifierType::empty()), None);
        assert_eq!(map_window_key(Key::Escape, ModifierType::empty()), None);
    }
}
