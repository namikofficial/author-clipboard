//! Focus chain + Esc handler. Populated in T007 (T005 in 0-indexed plan).
//!
//! This module implements the US-001 bug fix: Esc always closes the
//! popup (or clears search first, then closes).

use gtk4::gdk;
use gtk4::gdk::Key;
use gtk4::gdk::ModifierType;
use std::cell::RefCell;
use std::rc::Rc;

pub use crate::app::FocusTarget as FocusTargetReexport;
pub use crate::app::FocusTarget;

/// What to do when Esc is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscOutcome {
    /// Close the window.
    Close,
    /// Clear the search query and refocus the list.
    ClearSearch,
    /// Just move focus back to the list.
    BlurSearch,
    /// No-op (let the widget handle it).
    Proceed,
}

/// Decide what to do on Esc given the current focus and search state.
pub fn resolve_escape(focus: FocusTarget, search_query_empty: bool) -> EscOutcome {
    match focus {
        FocusTarget::Search if !search_query_empty => EscOutcome::ClearSearch,
        FocusTarget::Search => EscOutcome::BlurSearch,
        FocusTarget::Modal | FocusTarget::None => EscOutcome::Proceed,
        FocusTarget::List => EscOutcome::Close,
    }
}

/// Map a key + modifiers to an Action. Pure — no I/O, no GTK init required for tests.
pub fn map_key_extended(key: Key, mods: ModifierType) -> Option<crate::app::Action> {
    let ctrl = mods.contains(ModifierType::CONTROL_MASK);
    let shift = mods.contains(ModifierType::SHIFT_MASK);

    #[allow(clippy::match_same_arms)] // distinct keys map to the same action
    match (key, ctrl, shift) {
        // Navigation
        (Key::Up, false, false) => Some(crate::app::Action::MoveBy(-1)),
        (Key::Down, false, false) => Some(crate::app::Action::MoveBy(1)),
        (Key::Home, false, false) => Some(crate::app::Action::MoveTo(0)),
        (Key::End, false, false) => Some(crate::app::Action::MoveTo(usize::MAX)),
        (Key::Page_Up, false, false) => Some(crate::app::Action::MovePage(-1)),
        (Key::Page_Down, false, false) => Some(crate::app::Action::MovePage(1)),
        // Focus
        (Key::Escape, false, false) => Some(crate::app::Action::Focus(FocusTarget::List)),
        (Key::slash | Key::KP_Delete, false, false) => {
            Some(crate::app::Action::Focus(FocusTarget::Search))
        }
        // ? shortcut (Shift+/)
        (Key::minus, false, true) => Some(crate::app::Action::Focus(FocusTarget::Search)), // ? is -/Shift on some layouts
        // Shortcuts overlay
        (Key::question | Key::F1, false, false) => {
            Some(crate::app::Action::Focus(FocusTarget::Modal))
        }
        // Quick pick Ctrl+1..9
        (Key::_1, true, false) => Some(crate::app::Action::MoveTo(0)),
        (Key::_2, true, false) => Some(crate::app::Action::MoveTo(1)),
        (Key::_3, true, false) => Some(crate::app::Action::MoveTo(2)),
        (Key::_4, true, false) => Some(crate::app::Action::MoveTo(3)),
        (Key::_5, true, false) => Some(crate::app::Action::MoveTo(4)),
        (Key::_6, true, false) => Some(crate::app::Action::MoveTo(5)),
        (Key::_7, true, false) => Some(crate::app::Action::MoveTo(6)),
        (Key::_8, true, false) => Some(crate::app::Action::MoveTo(7)),
        (Key::_9, true, false) => Some(crate::app::Action::MoveTo(8)),
        // Page navigation
        (Key::Tab, true, false) => Some(crate::app::Action::CyclePage(1)),
        (Key::Tab, true, true) => Some(crate::app::Action::CyclePage(-1)),
        // Actions (runtime decides copy vs quick-paste based on mode)
        (Key::Return, false, false) => Some(crate::app::Action::Focus(FocusTarget::List)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::gdk::Key;
    use gtk4::gdk::ModifierType;

    #[test]
    fn esc_with_filled_search_clears() {
        assert_eq!(
            resolve_escape(FocusTarget::Search, false),
            EscOutcome::ClearSearch
        );
    }

    #[test]
    fn esc_with_empty_search_blurs() {
        assert_eq!(
            resolve_escape(FocusTarget::Search, true),
            EscOutcome::BlurSearch
        );
    }

    #[test]
    fn esc_with_list_focus_closes() {
        assert_eq!(resolve_escape(FocusTarget::List, true), EscOutcome::Close);
        assert_eq!(resolve_escape(FocusTarget::List, false), EscOutcome::Close);
    }

    #[test]
    fn esc_in_modal_proceeds() {
        assert_eq!(
            resolve_escape(FocusTarget::Modal, true),
            EscOutcome::Proceed
        );
    }

    #[test]
    fn esc_with_no_focus_proceeds() {
        // When no widget has focus yet (e.g. before window maps), Esc is a no-op.
        assert_eq!(resolve_escape(FocusTarget::None, true), EscOutcome::Proceed);
        assert_eq!(
            resolve_escape(FocusTarget::None, false),
            EscOutcome::Proceed
        );
    }

    #[test]
    fn map_key_extended_unknown_key_returns_none() {
        assert_eq!(map_key_extended(Key::A, ModifierType::empty()), None);
    }
}
