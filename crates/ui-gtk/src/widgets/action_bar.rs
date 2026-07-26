//! Selected-result command rail for the popup.

use gtk4::{glib, prelude::*};

use crate::app::{selected_command_available, AppState, SelectedItemCommand};

/// Commands exposed by the popup action rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailAction {
    /// Copy using the item's native MIME type.
    Copy,
    /// Copy and type into the previously focused window.
    QuickPaste,
    /// Copy a formatting-free text representation.
    PlainText,
    /// Apply a content-aware shared transformation and copy its result.
    Transform,
    /// Save the selected content as a snippet.
    CreateSnippet,
    /// Add the selected item to a collection.
    AddToCollection,
    /// Explicitly reveal a protected selection for the configured timeout.
    Reveal,
    /// Toggle retention pinning.
    Pin,
    /// Toggle priority starring.
    Star,
    /// Delete the selected history item.
    Delete,
}

/// Map a visible rail action to the typed state command it represents.
pub fn command_for(action: RailAction) -> SelectedItemCommand {
    match action {
        RailAction::Copy => SelectedItemCommand::Copy,
        RailAction::QuickPaste => SelectedItemCommand::QuickPaste,
        RailAction::PlainText => SelectedItemCommand::CopyPlainText,
        RailAction::Transform => SelectedItemCommand::Transform,
        RailAction::CreateSnippet => SelectedItemCommand::CreateSnippet,
        RailAction::AddToCollection => SelectedItemCommand::AddToCollection,
        RailAction::Reveal => SelectedItemCommand::Reveal,
        RailAction::Pin => SelectedItemCommand::Pin,
        RailAction::Star => SelectedItemCommand::Star,
        RailAction::Delete => SelectedItemCommand::Delete,
    }
}

/// Build a compact, keyboard-labelled command rail.
pub fn build(on_action: impl Fn(RailAction) + 'static) -> gtk4::Box {
    build_internal(None, on_action)
}

/// Build a rail whose controls track the authoritative selected item.
pub fn build_with_state(
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    on_action: impl Fn(RailAction) + 'static,
) -> gtk4::Box {
    build_internal(Some(state), on_action)
}

fn build_internal(
    state: Option<std::rc::Rc<std::cell::RefCell<AppState>>>,
    on_action: impl Fn(RailAction) + 'static,
) -> gtk4::Box {
    let rail = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    rail.add_css_class("popup-action-rail");
    rail.set_accessible_role(gtk4::AccessibleRole::Toolbar);
    let callback = std::rc::Rc::new(on_action);
    let buttons = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    for (label, tooltip, action) in [
        ("Copy", "Copy selected item (Enter)", RailAction::Copy),
        ("Paste", "Quick paste selected item", RailAction::QuickPaste),
        ("Text", "Copy as plain text", RailAction::PlainText),
        (
            "Transform",
            "Transform selected content",
            RailAction::Transform,
        ),
        (
            "Snippet",
            "Create snippet from selection",
            RailAction::CreateSnippet,
        ),
        (
            "Collection",
            "Add selection to a collection",
            RailAction::AddToCollection,
        ),
        ("Reveal", "Reveal protected selection", RailAction::Reveal),
        ("Pin", "Pin or unpin selected item", RailAction::Pin),
        ("Star", "Star or unstar selected item", RailAction::Star),
        ("Delete", "Delete selected item", RailAction::Delete),
    ] {
        let button = gtk4::Button::with_label(label);
        button.add_css_class("flat");
        button.set_tooltip_text(Some(tooltip));
        let callback = callback.clone();
        button.connect_clicked(move |_| callback(action));
        rail.append(&button);
        buttons.borrow_mut().push((action, button));
    }
    if let Some(state) = state {
        let refresh = {
            let buttons = buttons.clone();
            let state = state.clone();
            move || {
                let state = state.borrow();
                for (action, button) in buttons.borrow().iter() {
                    button.set_sensitive(selected_command_available(&state, command_for(*action)));
                }
            }
        };
        refresh();
        let rail_weak = rail.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if rail_weak.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            refresh();
            glib::ControlFlow::Continue
        });
    }
    rail
}
