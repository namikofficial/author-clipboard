//! 7-chip filter bar (All / Text / Images / Files / Pinned / Starred / Sensitive).
//!
//! Each chip is a [`gtk4::ToggleButton`] styled with the `chip`
//! CSS class. The active chip is set from a [`PickerFilter`]
//! and changes emit a callback that the parent page handles.

use gtk4::prelude::*;
use gtk4::{glib, Button, FlowBox, FlowBoxChild, Widget};

use crate::PickerFilter;

/// Callback signature for filter changes.
pub type OnChange = std::rc::Rc<dyn Fn(PickerFilter)>;

/// A 7-chip filter bar.
pub struct FilterBar {
    inner: FlowBox,
    buttons: Vec<(PickerFilter, Button)>,
}

impl FilterBar {
    /// Build a new filter bar. The bar owns a single `on_change`
    /// callback that fires whenever a chip is clicked.
    pub fn new(active: PickerFilter, on_change: OnChange) -> Self {
        let flow = FlowBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .homogeneous(false)
            .selection_mode(gtk4::SelectionMode::None)
            .column_spacing(6)
            .row_spacing(6)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        flow.add_css_class("filter-bar");

        let chips = [
            (PickerFilter::All, "All", "all"),
            (PickerFilter::Text, "Text", "text"),
            (PickerFilter::Images, "Images", "image"),
            (PickerFilter::Files, "Files", "files"),
            (PickerFilter::Pinned, "Pinned", "pin"),
            (PickerFilter::Starred, "Starred", "star"),
            (PickerFilter::Sensitive, "Sensitive", "lock"),
        ];

        let mut buttons = Vec::with_capacity(chips.len());
        for (filter, label, _icon) in chips {
            let btn = Button::with_label(label);
            btn.set_focusable(false);
            btn.add_css_class("chip");
            btn.set_hexpand(false);
            let child = FlowBoxChild::new();
            child.set_child(Some(&btn));
            flow.append(&child);
            if filter == active {
                btn.add_css_class("chip-active");
            }
            buttons.push((filter, btn));
        }

        // Wire the on_change callback. We have to bridge from per-button
        // clicks to a single shared handler.
        let buttons_for_handler: Vec<(PickerFilter, Button)> = buttons
            .iter()
            .map(|(f, b)| (*f, b.clone()))
            .collect();
        let on_change_for_handler = on_change.clone();
        for (filter, btn) in &buttons_for_handler {
            let cb = on_change_for_handler.clone();
            let buttons_for_emit = buttons_for_handler.clone();
            let f = *filter;
            btn.connect_clicked(move |clicked| {
                // Update visual state: clicked is now the only active chip.
                for (other_filter, other_btn) in &buttons_for_emit {
                    if *other_filter == f {
                        other_btn.add_css_class("chip-active");
                    } else {
                        other_btn.remove_css_class("chip-active");
                    }
                }
                cb(f);
            });
        }

        // The SignalHandlerId field is reserved for the GSettings
        // binding path that needs to block re-emission while a
        // change is propagating from the schema.
        Self {
            inner: flow,
            buttons,
        }
    }

    /// Mark a chip as the active one (used by GSettings binding to
    /// sync the bar from external state without firing the callback).
    pub fn set_active(&self, filter: PickerFilter) {
        for (f, btn) in &self.buttons {
            if *f == filter {
                btn.add_css_class("chip-active");
            } else {
                btn.remove_css_class("chip-active");
            }
        }
    }

    /// The current active filter, as reported by the chip CSS class.
    pub fn active(&self) -> PickerFilter {
        for (f, btn) in &self.buttons {
            if btn.has_css_class("chip-active") {
                return *f;
            }
        }
        PickerFilter::All
    }

    /// Borrow the underlying widget for embedding.
    pub fn widget(&self) -> &Widget {
        self.inner.upcast_ref()
    }
}
