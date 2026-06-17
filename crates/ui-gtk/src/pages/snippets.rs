//! Snippets page. Reads snippets from the DB and lets the user
//! add / copy / delete snippets. Wired to the same DB the daemon
//! uses.
//!
//! Template support (Phase 15 — `specs/features/026-snippet-templates/`):
//! the `Content` entry has a read-only preview label below it that
//! shows the rendered text via [`author_clipboard_shared::template`].

use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Entry, Label, ListBox, Orientation, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;

/// Build the snippets page widget.
pub fn build(config: &author_clipboard_shared::config::Config) -> Widget {
    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let heading = Label::new(Some("Snippets"));
    heading.set_halign(gtk4::Align::Start);
    heading.set_markup("<span weight=\"bold\" size=\"x-large\">Snippets</span>");
    vbox.append(&heading);

    // Add form.
    let name_entry = Entry::builder()
        .placeholder_text("Name")
        .hexpand(true)
        .build();
    let content_entry = Entry::builder()
        .placeholder_text("Content (use ${date}, ${uuid}, ${cursor}, etc.)")
        .hexpand(true)
        .build();
    let add_btn = Button::with_label("Add");
    add_btn.add_css_class("suggested-action");
    let form = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    form.append(&name_entry);
    form.append(&content_entry);
    form.append(&add_btn);
    vbox.append(&form);

    // Preview row — shows the rendered template below the form.
    let preview_label = Label::new(Some("Preview: "));
    preview_label.set_xalign(0.0);
    preview_label.set_wrap(true);
    preview_label.set_selectable(true);
    preview_label.add_css_class("dim-label");
    preview_label.add_css_class("snippet-preview");
    vbox.append(&preview_label);

    // Live-update the preview as the user types.
    {
        let preview_label = preview_label.clone();
        content_entry.connect_changed(move |entry| {
            let (rendered, _) = author_clipboard_shared::template::render_now(&entry.text());
            preview_label.set_text(&format!("Preview: {rendered}"));
        });
    }

    // List of existing snippets.
    let list = ListBox::new();
    list.add_css_class("snippet-list");
    refresh_list(&list, config);

    let refresh = list.clone();
    let config_for_refresh = config.clone();
    add_btn.connect_clicked(move |_| {
        let name = name_entry.text().to_string();
        let content = content_entry.text().to_string();
        if name.is_empty() || content.is_empty() {
            return;
        }
        if let Ok(db) = author_clipboard_shared::Database::open(&config_for_refresh.db_path()) {
            let _ = db.upsert_snippet(&name, &content);
            name_entry.set_text("");
            content_entry.set_text("");
            refresh_list(&refresh, &config_for_refresh);
        }
    });

    let scrolled = gtk4::ScrolledWindow::builder()
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();
    scrolled.set_child(Some(&list));
    vbox.append(&scrolled);

    vbox.upcast()
}

fn refresh_list(list: &ListBox, config: &author_clipboard_shared::config::Config) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    if let Ok(db) = author_clipboard_shared::Database::open(&config.db_path()) {
        if let Ok(snippets) = db.list_snippets() {
            for s in snippets {
                // Show the rendered preview as the subtitle so users can
                // see what the snippet will paste at a glance.
                let (rendered, _) = author_clipboard_shared::template::render_now(&s.content);
                let subtitle = truncate_for_row(&rendered, 100);
                let row = adw::ActionRow::builder()
                    .title(&s.name)
                    .subtitle(&subtitle)
                    .build();
                row.set_activatable(true);
                list.append(&row);
            }
        }
    }
    if list.first_child().is_none() {
        let empty = Label::new(Some("No snippets yet — add one above!"));
        empty.add_css_class("dim-label");
        empty.set_halign(gtk4::Align::Center);
        empty.set_margin_top(24);
        list.append(&empty);
    }
}

/// Truncate a string at a UTF-8 char boundary, appending an ellipsis
/// when cut. The list row's subtitle line needs to stay one line.
fn truncate_for_row(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    let mut out = String::with_capacity(idx + 1);
    out.push_str(&s[..idx]);
    out.push('\u{2026}');
    out
}
