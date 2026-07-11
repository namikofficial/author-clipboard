//! Native collection manager backed by the shared SQLite database.

use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::config::Config;
use author_clipboard_shared::types::{ClipboardItem, Collection};
use author_clipboard_shared::Database;
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
    (!name.is_empty()).then(|| name.to_string())
}

fn load_models(db: &Database) -> Result<Vec<CollectionRowModel>, String> {
    db.list_collections()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|collection| {
            let item_count = db
                .get_collection_items(&collection.id)
                .map_err(|e| e.to_string())?
                .len();
            Ok(CollectionRowModel {
                id: collection.id,
                name: collection.name,
                item_count,
            })
        })
        .collect()
}

struct PageWidgets {
    collections: ListBox,
    items: ListBox,
    title: Label,
    status: Label,
    rename: Entry,
}

/// Build the Collections manager page.
#[allow(deprecated)]
pub fn build(config: &Config) -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 12);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(18);
    page.set_margin_end(18);

    let heading = Label::new(Some("Collections"));
    heading.add_css_class("title-1");
    heading.set_halign(gtk4::Align::Start);
    page.append(&heading);

    let create_bar = GtkBox::new(Orientation::Horizontal, 8);
    let create_entry = Entry::builder()
        .placeholder_text("New collection name")
        .hexpand(true)
        .build();
    let create_button = Button::with_label("Create");
    create_button.add_css_class("suggested-action");
    create_bar.append(&create_entry);
    create_bar.append(&create_button);
    page.append(&create_bar);

    let body = gtk4::Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(300)
        .wide_handle(true)
        .vexpand(true)
        .build();
    let collections = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .build();
    collections.add_css_class("boxed-list");
    let collection_scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&collections)
        .build();

    let detail = GtkBox::new(Orientation::Vertical, 8);
    detail.set_margin_start(12);
    let title = Label::new(Some("Select a collection"));
    title.add_css_class("title-2");
    title.set_halign(gtk4::Align::Start);
    detail.append(&title);
    let edit_bar = GtkBox::new(Orientation::Horizontal, 8);
    let rename = Entry::builder()
        .placeholder_text("Collection name")
        .hexpand(true)
        .build();
    let rename_button = Button::with_label("Rename");
    let delete_button = Button::with_label("Delete");
    delete_button.add_css_class("destructive-action");
    edit_bar.append(&rename);
    edit_bar.append(&rename_button);
    edit_bar.append(&delete_button);
    detail.append(&edit_bar);
    let items = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .build();
    items.add_css_class("boxed-list");
    let item_scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .child(&items)
        .build();
    detail.append(&item_scroll);
    let status = Label::new(None);
    status.set_halign(gtk4::Align::Start);
    status.add_css_class("dim-label");
    detail.append(&status);
    body.set_start_child(Some(&collection_scroll));
    body.set_end_child(Some(&detail));
    page.append(&body);

    let widgets = Rc::new(PageWidgets {
        collections,
        items,
        title,
        status,
        rename,
    });
    let config = Rc::new(config.clone());
    let models = Rc::new(RefCell::new(Vec::<CollectionRowModel>::new()));
    let selected = Rc::new(RefCell::new(None::<String>));

    let refresh = make_refresh(
        config.clone(),
        widgets.clone(),
        models.clone(),
        selected.clone(),
    );
    refresh();

    let refresh_for_select = refresh.clone();
    let models_for_select = models.clone();
    let selected_for_select = selected.clone();
    let config_for_select = config.clone();
    let widgets_for_select = widgets.clone();
    widgets.collections.connect_row_selected(move |_list, row| {
        let Some(index) = row.and_then(|row| usize::try_from(row.index()).ok()) else {
            return;
        };
        let Some(model) = models_for_select.borrow().get(index).cloned() else {
            return;
        };
        *selected_for_select.borrow_mut() = Some(model.id.clone());
        widgets_for_select.title.set_label(&model.name);
        widgets_for_select.rename.set_text(&model.name);
        render_items(
            &config_for_select,
            &widgets_for_select,
            &model.id,
            refresh_for_select.clone(),
        );
    });

    let config_for_create = config.clone();
    let widgets_for_create = widgets.clone();
    let refresh_for_create = refresh.clone();
    let create_entry_for_click = create_entry.clone();
    create_button.connect_clicked(move |_| {
        let Some(name) = normalize_name(&create_entry_for_click.text()) else {
            widgets_for_create
                .status
                .set_label("Enter a collection name.");
            return;
        };
        match Database::open(&config_for_create.db_path())
            .and_then(|db| db.create_collection(&name))
        {
            Ok(_) => {
                create_entry_for_click.set_text("");
                widgets_for_create.status.set_label("Collection created.");
                refresh_for_create();
            }
            Err(e) => widgets_for_create
                .status
                .set_label(&format!("Could not create collection: {e}")),
        }
    });

    let config_for_rename = config.clone();
    let widgets_for_rename = widgets.clone();
    let selected_for_rename = selected.clone();
    let refresh_for_rename = refresh.clone();
    rename_button.connect_clicked(move |_| {
        let Some(id) = selected_for_rename.borrow().clone() else {
            return;
        };
        let Some(name) = normalize_name(&widgets_for_rename.rename.text()) else {
            widgets_for_rename
                .status
                .set_label("Enter a collection name.");
            return;
        };
        match Database::open(&config_for_rename.db_path())
            .and_then(|db| db.rename_collection(&id, &name))
        {
            Ok(()) => {
                widgets_for_rename.status.set_label("Collection renamed.");
                refresh_for_rename();
            }
            Err(e) => widgets_for_rename
                .status
                .set_label(&format!("Could not rename collection: {e}")),
        }
    });

    let config_for_delete = config.clone();
    let widgets_for_delete = widgets.clone();
    let selected_for_delete = selected.clone();
    let refresh_for_delete = refresh;
    delete_button.connect_clicked(move |button| {
        let Some(id) = selected_for_delete.borrow().clone() else {
            return;
        };
        let Some(window) = button
            .root()
            .and_then(|root| root.downcast::<gtk4::Window>().ok())
        else {
            return;
        };
        let dialog = gtk4::MessageDialog::builder()
            .transient_for(&window)
            .modal(true)
            .text("Delete this collection?")
            .secondary_text("Clipboard items in it will remain in history.")
            .build();
        dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
        dialog.add_button("Delete", gtk4::ResponseType::Accept);
        let config = config_for_delete.clone();
        let widgets = widgets_for_delete.clone();
        let selected = selected_for_delete.clone();
        let refresh = refresh_for_delete.clone();
        dialog.connect_response(move |dialog, response| {
            if response == gtk4::ResponseType::Accept {
                match Database::open(&config.db_path()).and_then(|db| db.delete_collection(&id)) {
                    Ok(()) => {
                        *selected.borrow_mut() = None;
                        widgets
                            .status
                            .set_label("Collection deleted; clipboard items were kept.");
                        refresh();
                    }
                    Err(e) => widgets
                        .status
                        .set_label(&format!("Could not delete collection: {e}")),
                }
            }
            dialog.close();
        });
        dialog.present();
    });

    page
}

type Refresh = Rc<dyn Fn()>;

fn make_refresh(
    config: Rc<Config>,
    widgets: Rc<PageWidgets>,
    models: Rc<RefCell<Vec<CollectionRowModel>>>,
    selected: Rc<RefCell<Option<String>>>,
) -> Refresh {
    Rc::new(move || {
        while let Some(child) = widgets.collections.first_child() {
            widgets.collections.remove(&child);
        }
        let loaded = Database::open(&config.db_path())
            .map_err(|e| e.to_string())
            .and_then(|db| load_models(&db));
        match loaded {
            Ok(rows) => {
                for row in &rows {
                    let line = GtkBox::new(Orientation::Horizontal, 8);
                    let name = Label::new(Some(&row.name));
                    name.set_halign(gtk4::Align::Start);
                    name.set_hexpand(true);
                    let badge = Label::new(Some(&row.item_count.to_string()));
                    badge.add_css_class("badge");
                    line.append(&name);
                    line.append(&badge);
                    widgets.collections.append(&line);
                }
                if rows.is_empty() {
                    widgets.status.set_label("No collections yet.");
                }
                *models.borrow_mut() = rows;
            }
            Err(e) => widgets
                .status
                .set_label(&format!("Could not load collections: {e}")),
        }
        if selected.borrow().is_none() {
            widgets.title.set_label("Select a collection");
            widgets.rename.set_text("");
            while let Some(child) = widgets.items.first_child() {
                widgets.items.remove(&child);
            }
        }
    })
}

fn render_items(config: &Config, widgets: &Rc<PageWidgets>, collection_id: &str, refresh: Refresh) {
    while let Some(child) = widgets.items.first_child() {
        widgets.items.remove(&child);
    }
    let result =
        Database::open(&config.db_path()).and_then(|db| db.get_collection_items(collection_id));
    match result {
        Ok(items) if items.is_empty() => widgets.status.set_label("This collection is empty."),
        Ok(items) => {
            widgets.status.set_label(&format!("{} items", items.len()));
            for item in items {
                widgets
                    .items
                    .append(&item_line(config, collection_id, &item, refresh.clone()));
            }
        }
        Err(e) => widgets
            .status
            .set_label(&format!("Could not load collection items: {e}")),
    }
}

fn item_line(
    config: &Config,
    collection_id: &str,
    item: &ClipboardItem,
    refresh: Refresh,
) -> GtkBox {
    let line = GtkBox::new(Orientation::Horizontal, 8);
    line.set_margin_top(8);
    line.set_margin_bottom(8);
    line.set_margin_start(8);
    line.set_margin_end(8);
    let preview = if item.sensitive || item.encrypted {
        item.redacted_preview
            .as_deref()
            .unwrap_or("Sensitive clipboard item")
    } else {
        item.plain_text.as_deref().unwrap_or(&item.content)
    };
    let preview = preview.lines().next().unwrap_or_default();
    let label = Label::new(Some(preview));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    let remove = Button::with_label("Remove");
    remove.add_css_class("flat");
    let config = config.clone();
    let collection_id = collection_id.to_string();
    let item_id = item.id;
    remove.connect_clicked(move |_| {
        if let Ok(db) = Database::open(&config.db_path()) {
            if db.remove_from_collection(&collection_id, item_id).is_ok() {
                refresh();
            }
        }
    });
    line.append(&label);
    line.append(&remove);
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_names_are_trimmed_and_blank_names_rejected() {
        assert_eq!(normalize_name("  deploy  ").as_deref(), Some("deploy"));
        assert_eq!(normalize_name(" \n\t "), None);
    }

    #[test]
    fn models_include_stable_id_and_item_count() {
        let db = Database::open_in_memory().unwrap();
        let id = db.create_collection("Work").unwrap();
        let item_id = db
            .insert_item(&ClipboardItem::new_text("cargo test".to_string()))
            .unwrap();
        db.add_to_collection(&id, item_id).unwrap();
        assert_eq!(
            load_models(&db).unwrap(),
            vec![CollectionRowModel {
                id,
                name: "Work".into(),
                item_count: 1
            }]
        );
    }
}
