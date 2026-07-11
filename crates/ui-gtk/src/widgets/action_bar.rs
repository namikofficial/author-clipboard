//! Selected-result command rail for the popup.

use gtk4::prelude::*;

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
    /// Explicitly reveal a protected selection for the configured timeout.
    Reveal,
    /// Toggle retention pinning.
    Pin,
    /// Toggle priority starring.
    Star,
    /// Delete the selected history item.
    Delete,
}

/// Build a compact, keyboard-labelled command rail.
pub fn build(on_action: impl Fn(RailAction) + 'static) -> gtk4::Box {
    let rail = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    rail.add_css_class("popup-action-rail");
    rail.set_accessible_role(gtk4::AccessibleRole::Toolbar);
    let callback = std::rc::Rc::new(on_action);
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
    }
    rail
}
