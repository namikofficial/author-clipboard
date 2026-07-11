//! Global key controller. Populated in T007 (T005 in 0-indexed plan).

#![allow(dead_code, unused_imports)]

use crate::app::{reduce, Action, AppState, FocusTarget};
use gtk4::gdk;
use gtk4::gdk::Key;
use gtk4::gdk::ModifierType;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::EventControllerKey;
use std::cell::RefCell;
use std::rc::Rc;

/// Map a key + modifiers to an Action. Pure — no I/O, no GTK init required for tests.
pub fn map_key_extended(key: Key, mods: ModifierType) -> Option<Action> {
    let ctrl = mods.contains(ModifierType::CONTROL_MASK);
    let shift = mods.contains(ModifierType::SHIFT_MASK);

    #[allow(clippy::match_same_arms)] // distinct keys map to the same action
    match (key, ctrl, shift) {
        // Navigation
        (Key::Up, false, false) => Some(Action::MoveBy(-1)),
        (Key::Down, false, false) => Some(Action::MoveBy(1)),
        (Key::Home, false, false) => Some(Action::MoveTo(0)),
        (Key::End, false, false) => Some(Action::MoveTo(usize::MAX)), // clamp in reducer
        (Key::Page_Up, false, false) => Some(Action::MovePage(-1)),
        (Key::Page_Down, false, false) => Some(Action::MovePage(1)),
        // Focus
        (Key::Escape, false, false) => Some(Action::Focus(FocusTarget::List)), // runtime resolves Esc via resolve_escape
        (Key::slash | Key::KP_Delete, false, false) => Some(Action::Focus(FocusTarget::Search)),
        // ? shortcut (Shift+/)
        (Key::minus, false, true) => Some(Action::Focus(FocusTarget::Search)), // ? is -/Shift on some layouts
        // Shortcuts overlay
        (Key::question | Key::F1, false, false) => Some(Action::Focus(FocusTarget::Modal)),
        // Quick pick Ctrl+1..9
        (Key::_1, true, false) => Some(Action::MoveTo(0)),
        (Key::_2, true, false) => Some(Action::MoveTo(1)),
        (Key::_3, true, false) => Some(Action::MoveTo(2)),
        (Key::_4, true, false) => Some(Action::MoveTo(3)),
        (Key::_5, true, false) => Some(Action::MoveTo(4)),
        (Key::_6, true, false) => Some(Action::MoveTo(5)),
        (Key::_7, true, false) => Some(Action::MoveTo(6)),
        (Key::_8, true, false) => Some(Action::MoveTo(7)),
        (Key::_9, true, false) => Some(Action::MoveTo(8)),
        // Page navigation
        (Key::Tab, true, false) => Some(Action::CyclePage(1)), // Ctrl+Tab = next
        (Key::Tab, true, true) => Some(Action::CyclePage(-1)), // Ctrl+Shift+Tab = prev
        // Collection organization and quick-access filters.
        (Key::p, true, false) => Some(Action::ToggleSelectedPin),
        (Key::s, true, true) => Some(Action::ToggleSelectedStar),
        (Key::p, true, true) => Some(Action::TogglePinnedFilter),
        (Key::a, true, true) => Some(Action::ToggleStarredFilter),
        // Actions (runtime decides copy vs quick-paste based on mode)
        (Key::Return, false, false) => Some(Action::Focus(FocusTarget::List)), // Enter in list = copy; runtime wires copy
        _ => None,
    }
}

/// Install the global key controller on a top-level widget.
///
/// This controller runs in [`PropagationPhase::Capture`] so it fires
/// before any widget's built-in handler. That is what fixes the
/// "Esc doesn't close when search has focus" bug (US-001).
pub fn install(
    window: &impl IsA<gtk4::Widget>,
    state: &Rc<RefCell<AppState>>,
    effects_tx: &std::sync::mpsc::Sender<crate::Effect>,
) -> EventControllerKey {
    use crate::controller::focus::resolve_escape;
    use crate::controller::focus::EscOutcome;

    let controller = EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let st = state.clone();
    let tx = effects_tx.clone();
    controller.connect_key_pressed(move |_, key, mods, _| {
        if key == gdk::Key::Escape {
            let outcome = resolve_escape(st.borrow().focus, st.borrow().search_query.is_empty());
            match outcome {
                EscOutcome::Close
                | EscOutcome::ClearSearch
                | EscOutcome::BlurSearch
                | EscOutcome::Proceed => {
                    let _ = tx.send(crate::Effect::Quit);
                    Propagation::Stop
                }
            }
        } else if let Some(action) =
            map_key_extended(key, gdk::ModifierType::from_bits_truncate(mods))
        {
            let mut s = st.borrow_mut();
            let effects = reduce(&mut s, action);
            for eff in effects {
                let _ = tx.send(eff);
            }
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });
    window.add_controller(controller.clone());
    controller
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::gdk::Key;
    use gtk4::gdk::ModifierType;

    #[test]
    fn map_key_up_returns_move_by_minus_1() {
        assert_eq!(
            map_key_extended(Key::Up, ModifierType::empty()),
            Some(Action::MoveBy(-1))
        );
    }

    #[test]
    fn map_key_down_returns_move_by_1() {
        assert_eq!(
            map_key_extended(Key::Down, ModifierType::empty()),
            Some(Action::MoveBy(1))
        );
    }

    #[test]
    fn map_key_ctrl_1_returns_move_to_0() {
        assert_eq!(
            map_key_extended(Key::_1, ModifierType::CONTROL_MASK),
            Some(Action::MoveTo(0))
        );
    }

    #[test]
    fn map_key_ctrl_2_returns_move_to_1() {
        assert_eq!(
            map_key_extended(Key::_2, ModifierType::CONTROL_MASK),
            Some(Action::MoveTo(1))
        );
    }

    #[test]
    fn map_key_ctrl_tab_returns_cycle_page_1() {
        assert_eq!(
            map_key_extended(Key::Tab, ModifierType::CONTROL_MASK),
            Some(Action::CyclePage(1))
        );
    }

    #[test]
    fn map_key_ctrl_shift_tab_returns_cycle_page_neg_1() {
        assert_eq!(
            map_key_extended(
                Key::Tab,
                ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
            ),
            Some(Action::CyclePage(-1))
        );
    }

    #[test]
    fn map_key_slash_without_ctrl_returns_focus_search() {
        assert_eq!(
            map_key_extended(Key::slash, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::Search))
        );
    }

    #[test]
    fn map_key_escape_returns_focus_list() {
        assert_eq!(
            map_key_extended(Key::Escape, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::List))
        );
    }

    #[test]
    fn map_key_question_opens_shortcuts_overlay() {
        assert_eq!(
            map_key_extended(Key::question, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::Modal))
        );
    }

    #[test]
    fn map_key_f1_opens_shortcuts_overlay() {
        assert_eq!(
            map_key_extended(Key::F1, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::Modal))
        );
    }

    #[test]
    fn map_key_return_without_modifiers_focuses_list() {
        assert_eq!(
            map_key_extended(Key::Return, ModifierType::empty()),
            Some(Action::Focus(FocusTarget::List))
        );
    }

    #[test]
    fn map_key_home_goes_to_first() {
        assert_eq!(
            map_key_extended(Key::Home, ModifierType::empty()),
            Some(Action::MoveTo(0))
        );
    }

    #[test]
    fn map_key_page_up_moves_page_backward() {
        assert_eq!(
            map_key_extended(Key::Page_Up, ModifierType::empty()),
            Some(Action::MovePage(-1))
        );
    }

    #[test]
    fn map_key_page_down_moves_page_forward() {
        assert_eq!(
            map_key_extended(Key::Page_Down, ModifierType::empty()),
            Some(Action::MovePage(1))
        );
    }

    #[test]
    fn collection_shortcuts_map_to_selected_actions_and_quick_filters() {
        assert_eq!(
            map_key_extended(Key::p, ModifierType::CONTROL_MASK),
            Some(Action::ToggleSelectedPin)
        );
        assert_eq!(
            map_key_extended(
                Key::s,
                ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
            ),
            Some(Action::ToggleSelectedStar)
        );
        assert_eq!(
            map_key_extended(
                Key::p,
                ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
            ),
            Some(Action::TogglePinnedFilter)
        );
        assert_eq!(
            map_key_extended(
                Key::a,
                ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
            ),
            Some(Action::ToggleStarredFilter)
        );
    }
}
