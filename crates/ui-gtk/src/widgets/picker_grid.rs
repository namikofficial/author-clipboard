//! Grid widget for emoji / symbol / kaomoji pickers.
//!
//! A `gtk::FlowBox`-based grid with category chips and search.
//! Used by `pages/emoji.rs`, `pages/symbols.rs`, `pages/kaomoji.rs`.

use gtk4::prelude::*;
use gtk4::{FlowBox, FlowBoxChild, Label, ScrolledWindow, SelectionMode, Widget};
use std::rc::Rc;

/// Callback when a grid item is activated.
pub type OnActivate = Rc<dyn Fn(&str)>;

/// Build a picker grid from a list of items.
///
/// Each item is rendered as a `FlowBoxChild` with a text label.
/// `on_activate` fires when the user clicks or presses Enter on an item.
pub fn build(items: &[String], on_activate: &OnActivate) -> Widget {
    let flow = FlowBox::builder()
        .selection_mode(SelectionMode::Single)
        .min_children_per_line(4)
        .max_children_per_line(12)
        .activate_on_single_click(true)
        .homogeneous(true)
        .column_spacing(4)
        .row_spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    for item in items {
        let item_clone = item.clone();
        let on_activate = Rc::clone(on_activate);
        let child = FlowBoxChild::new();
        let label = Label::new(Some(item));
        label.set_css_classes(&["picker-grid-item"]);
        label.set_margin_top(4);
        label.set_margin_bottom(4);
        label.set_margin_start(4);
        label.set_margin_end(4);
        child.set_child(Some(&label));
        child.connect_activate(move |_| {
            on_activate(&item_clone);
        });
        flow.append(&child);
    }

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .child(&flow)
        .build();

    scrolled.upcast()
}
