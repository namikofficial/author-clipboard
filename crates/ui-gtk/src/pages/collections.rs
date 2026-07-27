//! Asynchronous collection manager backed by the daemon service.

use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::config::Config;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Label, ListBox, Orientation, SelectionMode};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectionRowModel {
    id: String,
    name: String,
    item_count: usize,
}

fn normalize_name(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// Build the asynchronous Collections page.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
pub fn build(
    _config: &Config,
    service: std::sync::Arc<dyn crate::service::ClipboardService>,
) -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 12);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(18);
    page.set_margin_end(18);

    let heading = Label::new(Some("Collections"));
    heading.add_css_class("title-1");
    heading.set_halign(gtk4::Align::Start);
    page.append(&heading);

    let create_entry = Entry::builder()
        .placeholder_text("New collection name")
        .hexpand(true)
        .build();
    let create_button = Button::with_label("Create");
    create_button.add_css_class("suggested-action");
    let create_bar = GtkBox::new(Orientation::Horizontal, 8);
    create_bar.append(&create_entry);
    create_bar.append(&create_button);
    page.append(&create_bar);

    let collections = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .build();
    collections.add_css_class("boxed-list");
    let items = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .build();
    items.add_css_class("boxed-list");
    let title = Label::new(Some("Select a collection"));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Start);
    let rename = Entry::builder()
        .placeholder_text("Collection name")
        .hexpand(true)
        .build();
    let rename_button = Button::with_label("Rename");
    let delete_button = Button::with_label("Delete");
    delete_button.add_css_class("destructive-action");
    let status = Label::new(Some("Loading collections…"));
    status.set_halign(gtk4::Align::Start);
    status.add_css_class("dim-label");

    let edit = GtkBox::new(Orientation::Horizontal, 8);
    edit.append(&rename);
    edit.append(&rename_button);
    edit.append(&delete_button);
    let detail = GtkBox::new(Orientation::Vertical, 8);
    detail.set_margin_start(12);
    detail.append(&title);
    detail.append(&edit);
    detail.append(&items);
    detail.append(&status);
    let body = gtk4::Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(300)
        .vexpand(true)
        .build();
    body.set_start_child(Some(&collections));
    body.set_end_child(Some(&detail));
    page.append(&body);

    let models = Rc::new(RefCell::new(Vec::<CollectionRowModel>::new()));
    let selected = Rc::new(RefCell::new(None::<String>));

    let refresh: Rc<dyn Fn()> = {
        let service = service.clone();
        let collections = collections.clone();
        let models = models.clone();
        let status = status.clone();
        Rc::new(move || {
            let service = service.clone();
            let collections = collections.clone();
            let models = models.clone();
            let status = status.clone();
            glib::MainContext::default().spawn_local(async move {
                match service
                    .command(author_clipboard_shared::ipc::IpcCommand::ListCollections)
                    .await
                {
                    Ok(data) => {
                        while let Some(child) = collections.first_child() {
                            collections.remove(&child);
                        }
                        let rows: Vec<_> = data
                            .get("collections")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|v| {
                                Some(CollectionRowModel {
                                    id: v.get("id")?.as_str()?.to_owned(),
                                    name: v.get("name")?.as_str()?.to_owned(),
                                    item_count: usize::try_from(
                                        v.get("item_count")
                                            .and_then(serde_json::Value::as_u64)
                                            .unwrap_or(0),
                                    )
                                    .ok()?,
                                })
                            })
                            .collect();
                        for row in &rows {
                            let line = GtkBox::new(Orientation::Horizontal, 8);
                            let name = Label::new(Some(&row.name));
                            name.set_halign(gtk4::Align::Start);
                            name.set_hexpand(true);
                            line.append(&name);
                            line.append(&Label::new(Some(&row.item_count.to_string())));
                            collections.append(&line);
                        }
                        status.set_label(if rows.is_empty() {
                            "No collections yet."
                        } else {
                            "Collections loaded."
                        });
                        *models.borrow_mut() = rows;
                    }
                    Err(error) => status.set_label(&format!("Could not load collections: {error}")),
                }
            });
        })
    };

    let refresh_for_select = refresh.clone();
    let service_for_select = service.clone();
    let models_for_select = models.clone();
    let selected_for_select = selected.clone();
    let title_for_select = title.clone();
    let rename_for_select = rename.clone();
    let items_for_select = items.clone();
    let status_for_select = status.clone();
    collections.connect_row_selected(move |_list, row| {
        let Some(index) = row.and_then(|r| usize::try_from(r.index()).ok()) else {
            return;
        };
        let Some(model) = models_for_select.borrow().get(index).cloned() else {
            return;
        };
        *selected_for_select.borrow_mut() = Some(model.id.clone());
        title_for_select.set_label(&model.name);
        rename_for_select.set_text(&model.name);
        let service = service_for_select.clone();
        let items = items_for_select.clone();
        let status = status_for_select.clone();
        let _refresh = refresh_for_select.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Some(child) = items.first_child() {
                items.remove(&child);
            }
            match service
                .command(
                    author_clipboard_shared::ipc::IpcCommand::GetCollectionItems { id: model.id },
                )
                .await
            {
                Ok(data) => {
                    let values = data
                        .get("items")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    status.set_label(&format!("{} items", values.len()));
                    for value in values {
                        let preview = value
                            .get("preview")
                            .or_else(|| value.get("plain_text"))
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("Clipboard item");
                        items.append(&Label::new(Some(preview)));
                    }
                }
                Err(error) => status.set_label(&format!("Could not load items: {error}")),
            }
        });
    });

    let service_for_create = service.clone();
    let refresh_for_create = refresh.clone();
    let status_for_create = status.clone();
    create_button.connect_clicked(move |_| {
        let Some(name) = normalize_name(&create_entry.text()) else {
            status_for_create.set_label("Enter a collection name.");
            return;
        };
        let service = service_for_create.clone();
        let refresh = refresh_for_create.clone();
        let entry = create_entry.clone();
        let status = status_for_create.clone();
        glib::MainContext::default().spawn_local(async move {
            match service
                .command(author_clipboard_shared::ipc::IpcCommand::CreateCollection { name })
                .await
            {
                Ok(_) => {
                    entry.set_text("");
                    status.set_label("Collection created.");
                    refresh();
                }
                Err(error) => status.set_label(&format!("Could not create collection: {error}")),
            }
        });
    });

    let service_for_rename = service.clone();
    let selected_for_rename = selected.clone();
    let status_for_rename = status.clone();
    let refresh_for_rename = refresh.clone();
    rename_button.connect_clicked(move |_| {
        let Some(id) = selected_for_rename.borrow().clone() else {
            return;
        };
        let Some(name) = normalize_name(&rename.text()) else {
            status_for_rename.set_label("Enter a collection name.");
            return;
        };
        let service = service_for_rename.clone();
        let status = status_for_rename.clone();
        let refresh = refresh_for_rename.clone();
        glib::MainContext::default().spawn_local(async move {
            match service
                .command(author_clipboard_shared::ipc::IpcCommand::RenameCollection {
                    id,
                    new_name: name,
                })
                .await
            {
                Ok(_) => {
                    status.set_label("Collection renamed.");
                    refresh();
                }
                Err(error) => status.set_label(&format!("Could not rename collection: {error}")),
            }
        });
    });

    let service_for_delete = service.clone();
    let selected_for_delete = selected.clone();
    let status_for_delete = status.clone();
    let refresh_for_delete = refresh.clone();
    delete_button.connect_clicked(move |_| {
        let Some(id) = selected_for_delete.borrow().clone() else {
            return;
        };
        let service = service_for_delete.clone();
        let status = status_for_delete.clone();
        let refresh = refresh_for_delete.clone();
        glib::MainContext::default().spawn_local(async move {
            match service
                .command(author_clipboard_shared::ipc::IpcCommand::DeleteCollection { id })
                .await
            {
                Ok(_) => {
                    status.set_label("Collection deleted.");
                    refresh();
                }
                Err(error) => status.set_label(&format!("Could not delete collection: {error}")),
            }
        });
    });

    refresh();
    page
}

#[cfg(test)]
mod tests {
    use super::normalize_name;

    #[test]
    fn rejects_blank_collection_names() {
        assert_eq!(normalize_name("  "), None);
        assert_eq!(normalize_name(" Work "), Some("Work".into()));
    }
}
