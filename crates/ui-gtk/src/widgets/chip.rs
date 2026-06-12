//! Small pill-shaped chip used for source-app / age / type metadata.

use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Label, Widget};

/// Visual style of a [`Chip`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipStyle {
    /// Default accent-tinted chip.
    Default,
    /// Red / danger (sensitive content).
    Danger,
    /// Green / success (pinned).
    Success,
    /// Yellow / warning (starred).
    Warning,
    /// Muted / secondary.
    Muted,
}

/// A small label-shaped pill that fits inline with item rows.
///
/// Built on top of a [`gtk4::Box`] with a `chip` CSS class and an
/// optional modifier class for color.
pub struct Chip {
    inner: GtkBox,
    label: Label,
}

impl Chip {
    /// Build a new chip with the given text and style.
    pub fn new(text: &str, style: ChipStyle) -> Self {
        let label = Label::builder()
            .label(text)
            .hexpand(false)
            .halign(gtk4::Align::Start)
            .build();
        label.add_css_class("chip-label");
        label.set_xalign(0.5);

        let inner = GtkBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .build();
        inner.add_css_class("chip");
        inner.add_css_class(match style {
            ChipStyle::Default => "chip-default",
            ChipStyle::Danger => "chip-danger",
            ChipStyle::Success => "chip-success",
            ChipStyle::Warning => "chip-warning",
            ChipStyle::Muted => "chip-muted",
        });
        inner.append(&label);

        // The box itself is small; we don't want it to eat horizontal
        // space in a row layout.
        inner.set_size_request(-1, 18);

        Self { inner, label }
    }

    /// Update the chip's text.
    pub fn set_text(&self, text: &str) {
        self.label.set_text(text);
    }

    /// Borrow the underlying GTK widget.
    pub fn widget(&self) -> &Widget {
        self.inner.upcast_ref()
    }
}

impl Default for Chip {
    fn default() -> Self {
        Self::new("", ChipStyle::Default)
    }
}
