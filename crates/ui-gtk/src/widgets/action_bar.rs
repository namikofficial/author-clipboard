//! Selected-result command rail for the popup.

use gtk4::prelude::*;

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

/// A constructed action rail with a refresh handle for reactive updates.
pub struct ActionRail {
    /// The GTK box widget.
    pub widget: gtk4::Box,
    /// Call to re-evaluate button sensitivity from current state.
    /// Wrapped in `Rc` so multiple GTK signal handlers can share it.
    pub refresh: std::rc::Rc<dyn Fn()>,
}

/// Build a compact, keyboard-labelled command rail without state tracking.
pub fn build(on_action: impl Fn(RailAction) + 'static) -> gtk4::Box {
    build_with_state_inner(std::rc::Rc::new(std::cell::RefCell::new(AppState::default())), on_action, false).widget
}

/// Build a rail whose controls track the authoritative selected item.
///
/// Returns an `ActionRail` whose `refresh` method must be called when state
/// changes that affect button sensitivity (selection, pin/star toggles,
/// item content changes). Replaces the old 100ms polling approach.
pub fn build_with_state(
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    on_action: impl Fn(RailAction) + 'static,
) -> ActionRail {
    build_with_state_inner(state, on_action, true)
}

fn build_with_state_inner(
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    on_action: impl Fn(RailAction) + 'static,
    track_state: bool,
) -> ActionRail {
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
    let refresh: std::rc::Rc<dyn Fn()> = if track_state {
        let buttons = buttons.clone();
        let state = state.clone();
        std::rc::Rc::new(move || {
            let s = state.borrow();
            for (action, button) in buttons.borrow().iter() {
                button.set_sensitive(selected_command_available(&s, command_for(*action)));
            }
        })
    } else {
        std::rc::Rc::new(|| {})
    };
    refresh();
    ActionRail { widget: rail, refresh }
}
