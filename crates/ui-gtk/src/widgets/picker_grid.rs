//! Searchable expression grid shared by emoji, symbol, and kaomoji pages.

use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::{clipboard, config::Config, Database};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, FlowBox, FlowBoxChild, Label, Orientation, SearchEntry, Widget};

#[derive(Debug, Clone, PartialEq, Eq)]
/// One searchable expression displayed by a picker.
pub struct ExpressionItem {
    /// Text copied to the clipboard.
    pub value: String,
    /// Human-readable search term and tooltip.
    pub description: String,
    /// Category used by the category chips.
    pub category: String,
}

#[derive(Debug, Clone, Copy)]
/// Visual and persistence options for one picker page.
pub struct PickerPresentation {
    /// Page heading.
    pub title: &'static str,
    /// Key used in the recently-used database table.
    pub kind: &'static str,
    /// Whether cells should expand to fit long expressions.
    pub wide_cells: bool,
}

#[derive(Debug, Default)]
struct PickerState {
    query: String,
    category: Option<String>,
}

#[must_use]
/// Return whether an item passes the active query and category.
pub fn matches(item: &ExpressionItem, query: &str, category: Option<&str>) -> bool {
    if category.is_some_and(|selected| item.category != selected) {
        return false;
    }
    let query = query.trim().to_lowercase();
    query.is_empty()
        || item.value.to_lowercase().contains(&query)
        || item.description.to_lowercase().contains(&query)
        || item.category.to_lowercase().contains(&query)
}

/// Build a complete picker with search, category chips, recents, and activation.
#[allow(clippy::too_many_lines)]
pub fn build(
    presentation: PickerPresentation,
    categories: &[(String, String)],
    items: Vec<ExpressionItem>,
) -> Widget {
    let config = Config::load();
    let recent = Database::open(&config.db_path())
        .and_then(|db| db.get_recently_used(presentation.kind, 18))
        .unwrap_or_default();
    let items = Rc::new(items);
    let recent = Rc::new(RefCell::new(recent));
    let state = Rc::new(RefCell::new(PickerState::default()));

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let heading = Label::new(None);
    heading.set_halign(gtk4::Align::Start);
    heading.set_markup(&format!(
        "<span weight=\"bold\" size=\"x-large\">{}</span>",
        glib::markup_escape_text(presentation.title)
    ));
    root.append(&heading);

    let search = SearchEntry::builder()
        .placeholder_text(format!("Search {}", presentation.title.to_lowercase()))
        .hexpand(true)
        .build();
    root.append(&search);

    let chip_strip = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    let all = Button::with_label("All");
    all.set_widget_name("all");
    all.add_css_class("chip");
    all.add_css_class("suggested-action");
    chip_strip.append(&all);
    let button = Button::with_label("↻ Recent");
    button.add_css_class("chip");
    button.set_widget_name("recent");
    chip_strip.append(&button);
    for (name, icon) in categories {
        let button = Button::with_label(&format!("{icon} {name}"));
        button.add_css_class("chip");
        button.set_widget_name(name);
        chip_strip.append(&button);
    }
    let chips = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Never)
        .child(&chip_strip)
        .build();
    root.append(&chips);

    let flow = FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .activate_on_single_click(true)
        .homogeneous(!presentation.wide_cells)
        .min_children_per_line(if presentation.wide_cells { 1 } else { 4 })
        .max_children_per_line(if presentation.wide_cells { 4 } else { 12 })
        .column_spacing(6)
        .row_spacing(6)
        .build();
    flow.add_css_class("picker-grid");
    let empty = Label::new(Some("No expressions match this search."));
    empty.add_css_class("dim-label");
    empty.set_visible(false);
    let status = Label::new(None);
    status.set_halign(gtk4::Align::Start);
    status.add_css_class("dim-label");

    let render: Rc<dyn Fn()> = {
        let flow = flow.clone();
        let empty = empty.clone();
        let status = status.clone();
        let state = Rc::clone(&state);
        let items = Rc::clone(&items);
        let recent = Rc::clone(&recent);
        Rc::new(move || {
            while let Some(child) = flow.first_child() {
                flow.remove(&child);
            }
            let state = state.borrow();
            let visible: Vec<ExpressionItem> = if state.category.as_deref() == Some("recent") {
                recent
                    .borrow()
                    .iter()
                    .filter_map(|value| items.iter().find(|item| &item.value == value).cloned())
                    .filter(|item| matches(item, &state.query, None))
                    .collect()
            } else {
                items
                    .iter()
                    .filter(|item| matches(item, &state.query, state.category.as_deref()))
                    .cloned()
                    .collect()
            };
            empty.set_visible(visible.is_empty());
            status.set_text(&format!("{} results", visible.len()));
            for item in visible {
                let child = FlowBoxChild::new();
                let button = Button::with_label(&item.value);
                button.add_css_class(if presentation.wide_cells {
                    "kaomoji-cell"
                } else if presentation.kind == "emoji" {
                    "emoji-cell"
                } else {
                    "symbol-cell"
                });
                button.set_tooltip_text(Some(&format!("{} · {}", item.description, item.category)));
                if presentation.wide_cells {
                    button.set_hexpand(true);
                    button.set_halign(gtk4::Align::Fill);
                } else {
                    button.set_size_request(44, 44);
                }
                let value = item.value.clone();
                let status = status.clone();
                let recent = Rc::clone(&recent);
                button.connect_clicked(move |_| match clipboard::set_clipboard_text(&value) {
                    Ok(_) => {
                        let config = Config::load();
                        if let Ok(db) = Database::open(&config.db_path()) {
                            if let Err(error) = db.record_usage(presentation.kind, &value) {
                                tracing::warn!("failed to persist expression usage: {error}");
                            }
                        }
                        let mut recent = recent.borrow_mut();
                        recent.retain(|entry| entry != &value);
                        recent.insert(0, value.clone());
                        recent.truncate(18);
                        status.set_text(&format!("Copied {value}"));
                    }
                    Err(error) => status.set_text(&format!("Could not copy: {error}")),
                });
                child.set_child(Some(&button));
                flow.append(&child);
            }
        })
    };

    {
        let state = Rc::clone(&state);
        let render = Rc::clone(&render);
        search.connect_search_changed(move |entry| {
            state.borrow_mut().query = entry.text().to_string();
            render();
        });
    }
    {
        let state = Rc::clone(&state);
        let render = Rc::clone(&render);
        all.connect_clicked(move |_| {
            state.borrow_mut().category = None;
            render();
        });
    }
    let mut child = chip_strip.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        let Ok(button) = widget.downcast::<Button>() else {
            continue;
        };
        let category = button.widget_name().to_string();
        if category == "all" {
            continue;
        }
        let state = Rc::clone(&state);
        let render = Rc::clone(&render);
        button.connect_clicked(move |_| {
            state.borrow_mut().category = Some(category.clone());
            render();
        });
    }

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .child(&flow)
        .build();
    root.append(&scrolled);
    root.append(&empty);
    root.append(&status);
    render();
    root.upcast()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> ExpressionItem {
        ExpressionItem {
            value: "∞".into(),
            description: "Infinity".into(),
            category: "Math".into(),
        }
    }

    #[test]
    fn matches_value_description_and_category_case_insensitively() {
        assert!(matches(&item(), "∞", None));
        assert!(matches(&item(), "inFIN", None));
        assert!(matches(&item(), "math", None));
    }

    #[test]
    fn combines_category_and_query() {
        assert!(matches(&item(), "infinity", Some("Math")));
        assert!(!matches(&item(), "infinity", Some("Currency")));
        assert!(!matches(&item(), "heart", Some("Math")));
    }
}
