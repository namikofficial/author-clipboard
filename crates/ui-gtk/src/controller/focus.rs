//! Focus chain + Esc handler.
//!
//! Returns what to do on Esc based on focus and search state.
//! GTK widget focus is authoritative — this module only provides
//! pure resolution logic used by the window-level controller.

pub use crate::app::FocusTarget;

/// What to do when Esc is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscOutcome {
    /// Close the window.
    Close,
    /// Clear the search query and refocus the list.
    ClearSearch,
    /// Just move focus back to the list (search text empty, so no clear needed).
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

    #[test]
    fn esc_with_no_focus_proceeds() {
        assert_eq!(resolve_escape(FocusTarget::None, true), EscOutcome::Proceed);
        assert_eq!(
            resolve_escape(FocusTarget::None, false),
            EscOutcome::Proceed
        );
    }
}
