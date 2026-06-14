//! Focus chain + Esc handler. Populated in T007 (T005 in 0-indexed plan).
//!
//! This module implements the US-001 bug fix: Esc always closes the
//! popup (or clears search first, then closes).

#![allow(dead_code, unused_imports)]

use gtk4::prelude::*;

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
}

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
        FocusTarget::Modal => EscOutcome::Proceed,
        FocusTarget::List => EscOutcome::Close,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
