//! Global key controller. Populated in T007 (T005 in 0-indexed plan).

#![allow(dead_code, unused_imports)]

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

/// Suppress a glib warning about unused symbols.
#[allow(dead_code)]
fn _suppress(_: glib::Propagation) {}
