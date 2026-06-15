//! Kaomoji picker page. Reads from `shared::kaomoji::CATEGORIES`.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, FlowBox, FlowBoxChild, Label, Orientation, Widget};

/// Build the kaomoji page widget.
pub fn build() -> Widget {
    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let heading = Label::new(Some("Kaomoji"));
    heading.set_halign(gtk4::Align::Start);
    heading.set_markup("<span weight=\"bold\" size=\"x-large\">Kaomoji</span>");
    vbox.append(&heading);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();
    let grid = FlowBox::builder()
        .orientation(Orientation::Vertical)
        .homogeneous(false)
        .column_spacing(8)
        .row_spacing(8)
        .build();
    grid.add_css_class("picker-grid");
    for cat in author_clipboard_shared::kaomoji::CATEGORIES {
        for &k in cat.items {
            let child = FlowBoxChild::new();
            let btn = Button::with_label(k);
            btn.set_halign(gtk4::Align::Start);
            btn.set_size_request(200, 32);
            btn.add_css_class("kaomoji-cell");
            child.set_child(Some(&btn));
            grid.append(&child);
        }
    }
    scrolled.set_child(Some(&grid));
    vbox.append(&scrolled);

    vbox.upcast()
}
