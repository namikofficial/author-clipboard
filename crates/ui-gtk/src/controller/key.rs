//! Global key controller. Populated in T007 (T005 in 0-indexed plan).

#![allow(dead_code, unused_imports)]

use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{EventControllerKey, PropagationPhase, Widget};

/// Install the global key controller on a top-level widget.
///
/// This controller runs in [`PropagationPhase::Capture`] so it fires
/// before any widget's built-in handler. That is what fixes the
/// "Esc doesn't close when search has focus" bug (US-001).
pub fn install(window: &impl IsA<Widget>) -> EventControllerKey {
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(PropagationPhase::Capture);
    window.add_controller(controller.clone());
    controller
}

/// Map a [`gdk::Key`] + modifiers to a high-level action.
pub fn map_key(key: gdk::Key) -> Option<crate::app::KeyAction> {
    use gdk::Key;
    match key {
        Key::Escape => Some(crate::app::KeyAction::Escape),
        Key::slash | Key::question => Some(crate::app::KeyAction::FocusSearch),
        Key::Up => Some(crate::app::KeyAction::MoveUp),
        Key::Down => Some(crate::app::KeyAction::MoveDown),
        Key::Left => Some(crate::app::KeyAction::MoveLeft),
        Key::Right => Some(crate::app::KeyAction::MoveRight),
        Key::Home => Some(crate::app::KeyAction::First),
        Key::End => Some(crate::app::KeyAction::Last),
        Key::Page_Up => Some(crate::app::KeyAction::PageUp),
        Key::Page_Down => Some(crate::app::KeyAction::PageDown),
        Key::Return => Some(crate::app::KeyAction::Enter),
        _ => None,
    }
}

/// Suppress a glib warning about unused symbols.
#[allow(dead_code)]
fn _suppress(_: glib::Propagation) {}
