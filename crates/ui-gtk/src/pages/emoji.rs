//! Emoji picker page. Reads from `shared::emoji::CATEGORIES` and
//! renders a grid of emoji with category chips.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, FlowBox, FlowBoxChild, Label, Orientation, Widget};

/// Build the emoji page widget.
pub fn build() -> Widget {
    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let heading = Label::new(Some("Emoji"));
    heading.set_halign(gtk4::Align::Start);
    heading.set_markup("<span weight=\"bold\" size=\"x-large\">Emoji</span>");
    vbox.append(&heading);

    let categories = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    for cat in author_clipboard_shared::emoji::CATEGORIES {
        let btn = Button::with_label(&format!("{} {}", cat.icon, cat.name));
        btn.add_css_class("chip");
        categories.append(&btn);
    }
    vbox.append(&categories);

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
    for cat in author_clipboard_shared::emoji::CATEGORIES {
        for &emoji in cat.emojis {
            let child = FlowBoxChild::new();
            let btn = Button::with_label(emoji);
            btn.set_focusable(true);
            btn.set_size_request(40, 40);
            btn.add_css_class("emoji-cell");
            child.set_child(Some(&btn));
            grid.append(&child);
        }
    }
    scrolled.set_child(Some(&grid));
    vbox.append(&scrolled);

    vbox.upcast()
}
