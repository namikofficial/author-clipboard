//! Asynchronous snippets page backed by the daemon service.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Label, ListBox, Orientation, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;

/// Build the snippets page without performing I/O on GTK callbacks.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn build(
    _config: &author_clipboard_shared::config::Config,
    service: std::sync::Arc<dyn crate::service::ClipboardService>,
) -> Widget {
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
    let name = Entry::builder()
        .placeholder_text("Name")
        .hexpand(true)
        .build();
    let content = Entry::builder()
        .placeholder_text("Content (use ${date}, ${uuid}, ${cursor}, etc.)")
        .hexpand(true)
        .build();
    let add = Button::with_label("Add");
    add.add_css_class("suggested-action");
    let form = GtkBox::new(Orientation::Horizontal, 6);
    form.append(&name);
    form.append(&content);
    form.append(&add);
    vbox.append(&form);
    let preview = Label::new(Some("Preview: "));
    preview.set_xalign(0.0);
    preview.set_wrap(true);
    preview.add_css_class("dim-label");
    vbox.append(&preview);
    let preview_for_input = preview.clone();
    content.connect_changed(move |entry| {
        let (rendered, _) = author_clipboard_shared::template::render_now(&entry.text());
        preview_for_input.set_text(&format!("Preview: {rendered}"));
    });
    let list = ListBox::new();
    list.add_css_class("snippet-list");
    let scrolled = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .child(&list)
        .build();
    vbox.append(&scrolled);

    let refresh: std::rc::Rc<dyn Fn()> = {
        let service = service.clone();
        let list = list.clone();
        std::rc::Rc::new(move || {
            let service = service.clone();
            let list = list.clone();
            glib::MainContext::default().spawn_local(async move {
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }
                match service
                    .command(author_clipboard_shared::ipc::IpcCommand::ListSnippets)
                    .await
                {
                    Ok(data) => {
                        for value in data
                            .get("snippets")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                        {
                            let name = value
                                .get("name")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("Unnamed");
                            let text = value
                                .get("content")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("");
                            let (rendered, _) = author_clipboard_shared::template::render_now(text);
                            let row = adw::ActionRow::builder()
                                .title(name)
                                .subtitle(truncate_for_row(&rendered, 100))
                                .build();
                            list.append(&row);
                        }
                        if list.first_child().is_none() {
                            list.append(&Label::new(Some("No snippets yet — add one above!")));
                        }
                    }
                    Err(error) => list.append(&Label::new(Some(&format!(
                        "Could not load snippets: {error}"
                    )))),
                }
            });
        })
    };
    let service_for_add = service.clone();
    let refresh_for_add = refresh.clone();
    add.connect_clicked(move |_| {
        let name_value = name.text().to_string();
        let content_value = content.text().to_string();
        if name_value.is_empty() || content_value.is_empty() {
            return;
        }
        let service = service_for_add.clone();
        let refresh = refresh_for_add.clone();
        let name = name.clone();
        let content = content.clone();
        glib::MainContext::default().spawn_local(async move {
            match service
                .command(author_clipboard_shared::ipc::IpcCommand::UpsertSnippet {
                    name: name_value,
                    content: content_value,
                })
                .await
            {
                Ok(_) => {
                    name.set_text("");
                    content.set_text("");
                    refresh();
                }
                Err(error) => tracing::warn!(%error, "snippet update failed"),
            }
        });
    });
    refresh();
    vbox.upcast()
}

fn truncate_for_row(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    format!("{}…", &s[..idx])
}
