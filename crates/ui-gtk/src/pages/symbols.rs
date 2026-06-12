//! Symbols picker page. Reads from `shared::symbols::CATEGORIES`.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, FlowBox, FlowBoxChild, Label, Orientation, Widget};

/// Build the symbols page widget.
pub fn build() -> Widget {
    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let heading = Label::new(Some("Symbols"));
    heading.set_halign(gtk4::Align::Start);
    heading.set_markup("<span weight=\"bold\" size=\"x-large\">Symbols</span>");
    vbox.append(&heading);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();
    let grid = FlowBox::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .column_spacing(4)
        .row_spacing(4)
        .build();
    grid.add_css_class("picker-grid");
    for cat in author_clipboard_shared::symbols::CATEGORIES {
        for &(sym, _desc) in cat.symbols {
            let child = FlowBoxChild::new();
            let btn = Button::with_label(sym);
            btn.set_size_request(40, 40);
            btn.set_tooltip_text(Some(sym));
            btn.add_css_class("symbol-cell");
            child.set_child(Some(&btn));
            grid.append(&child);
        }
    }
    scrolled.set_child(Some(&grid));
    vbox.append(&scrolled);

    vbox.upcast()
}
