//! author-clipboard-hypr-picker: First-party Hyprland/wlroots clipboard picker
//!
//! A lightweight, keyboard-first clipboard picker that opens as a Wayland
//! layer-shell overlay. Designed for Hyprland keybinds but works on any
//! wlroots compositor with layer-shell support.
//!
//! Usage:
//!   author-clipboard-hypr-picker [--source history|snippets|emoji|symbols|kaomoji|all]
//!                                 [--count 50] [--include-sensitive]

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use author_clipboard_shared::config::Config;
use author_clipboard_shared::db::Database;
use author_clipboard_shared::picker::{
    self, PickerAction, PickerEntry, PickerOptions, PickerSource,
};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const APP_ID: &str = "com.namikofficial.author-clipboard-hypr-picker";

struct PickerState {
    entries: Vec<PickerEntry>,
    filtered: Vec<PickerEntry>,
    selected_index: usize,
    source: PickerSource,
    include_sensitive: bool,
    action: PickerAction,
}

#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<()> {
    let app = gtk::Application::builder().application_id(APP_ID).build();

    let args: Vec<String> = std::env::args().collect();
    let options = parse_args(&args);

    app.connect_activate(move |app| build_ui(app, options.clone()));
    app.run();

    Ok(())
}

fn parse_args(args: &[String]) -> PickerOptions {
    let mut source = PickerSource::History;
    let mut limit = 50;
    let mut include_sensitive = false;
    let mut action = PickerAction::Copy;
    let mut query = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source" | "-s" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    source = val.parse().unwrap_or(PickerSource::History);
                }
            }
            "--count" | "-c" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    limit = val.parse().unwrap_or(50);
                }
            }
            "--include-sensitive" => {
                include_sensitive = true;
            }
            "--action" | "-a" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    action = val.parse().unwrap_or(PickerAction::Copy);
                }
            }
            "--query" | "-q" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    query = Some(val.clone());
                }
            }
            _ => {}
        }
        i += 1;
    }

    PickerOptions {
        source,
        limit,
        query,
        include_sensitive,
        action,
    }
}

#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
fn build_ui(app: &gtk::Application, options: PickerOptions) {
    let config = Config::load();
    let db = Database::open(&config.db_path()).expect("Failed to open database");

    let entries = picker::load_entries(&db, &config, &options).unwrap_or_default();

    let state = Rc::new(RefCell::new(PickerState {
        entries: entries.clone(),
        filtered: entries,
        selected_index: 0,
        source: options.source,
        include_sensitive: options.include_sensitive,
        action: options.action,
    }));

    let width: i32 = config.picker.width.try_into().unwrap_or(720);
    let height: i32 = config.picker.height.try_into().unwrap_or(520);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Author Clipboard")
        .default_width(width)
        .default_height(height)
        .decorated(false)
        .resizable(false)
        .build();

    // Layer-shell setup
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search clipboard\u{2026}"));
    main_box.append(&search_entry);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.set_min_content_height(300);

    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);

    populate_list_box(&list_box, &state.borrow());

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    let status_label = gtk::Label::new(None);
    update_status(&status_label, &state.borrow());
    main_box.append(&status_label);

    window.set_child(Some(&main_box));

    // ── Keyboard controller ────────────────────────────────────

    let key_controller = gtk::EventControllerKey::new();
    let window_ref = window.clone();
    let state_clone = Rc::clone(&state);
    let list_box_ref = list_box.clone();
    let status_ref = status_label.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        let mut s = state_clone.borrow_mut();
        let len = s.filtered.len();
        if len == 0 {
            return glib::Propagation::Proceed;
        }

        let key_name = key
            .name()
            .map(|n| n.as_str().to_owned())
            .unwrap_or_default();

        match key_name.as_str() {
            "Escape" => {
                window_ref.close();
                glib::Propagation::Stop
            }
            "Down" => {
                s.selected_index = (s.selected_index + 1).min(len - 1);
                select_row_at(&list_box_ref, &s, &status_ref, &state_clone);
                glib::Propagation::Stop
            }
            "Up" => {
                s.selected_index = s.selected_index.saturating_sub(1);
                select_row_at(&list_box_ref, &s, &status_ref, &state_clone);
                glib::Propagation::Stop
            }
            "Return" => {
                if let Some(entry) = s.filtered.get(s.selected_index) {
                    let entry = entry.clone();
                    let action = s.action;
                    drop(s);
                    let config = Config::load();
                    let _ = picker::restore_entry(&entry, &config, action);
                    window_ref.close();
                }
                glib::Propagation::Stop
            }
            "Delete" => {
                handle_delete(&mut s, &list_box_ref, &state_clone, &status_ref);
                glib::Propagation::Stop
            }
            _ => {
                if modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    if let Some(num) = key_name.chars().next().and_then(|c| c.to_digit(10)) {
                        let idx = (num as usize).saturating_sub(1);
                        if idx < len {
                            if let Some(entry) = s.filtered.get(idx) {
                                let entry = entry.clone();
                                let action = s.action;
                                drop(s);
                                let config = Config::load();
                                let _ = picker::restore_entry(&entry, &config, action);
                                window_ref.close();
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                }
                glib::Propagation::Proceed
            }
        }
    });

    window.add_controller(key_controller.clone().upcast::<gtk::EventController>());

    // ── Search filtering ───────────────────────────────────────

    let state_for_search = Rc::clone(&state);
    let list_for_search = list_box.clone();
    let status_for_search = status_label.clone();

    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        let mut s = state_for_search.borrow_mut();
        if query.is_empty() {
            let cloned = s.entries.clone();
            s.filtered = cloned;
        } else {
            s.filtered = picker::filter_entries(&s.entries, &query);
        }
        s.selected_index = 0;
        drop(s);

        populate_list_box(&list_for_search, &state_for_search.borrow());
        update_status(&status_for_search, &state_for_search.borrow());

        if let Some(row) = list_for_search.row_at_index(0) {
            list_for_search.select_row(Some(&row));
        }
    });

    search_entry.grab_focus();

    if let Some(row) = list_box.row_at_index(0) {
        list_box.select_row(Some(&row));
    }

    window.present();
}

fn select_row_at(
    list_box: &gtk::ListBox,
    state: &PickerState,
    status: &gtk::Label,
    state_ref: &Rc<RefCell<PickerState>>,
) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let idx = state.selected_index as i32;
    if let Some(row) = list_box.row_at_index(idx) {
        list_box.select_row(Some(&row));
        row.grab_focus();
    }
    update_status(status, &state_ref.borrow());
}

fn handle_delete(
    state: &mut PickerState,
    list_box: &gtk::ListBox,
    state_ref: &Rc<RefCell<PickerState>>,
    status: &gtk::Label,
) {
    if let Some(entry) = state.filtered.get(state.selected_index) {
        if let Some(id) = entry.id {
            let config = Config::load();
            if let Ok(db) = Database::open(&config.db_path()) {
                let _ = db.delete_item(id);
                let opts = PickerOptions {
                    source: state.source,
                    limit: state.entries.len(),
                    query: None,
                    include_sensitive: state.include_sensitive,
                    action: state.action,
                };
                if let Ok(new_entries) = picker::load_entries(&db, &config, &opts) {
                    state.entries = new_entries;
                    let cloned = state.entries.clone();
                    state.filtered = cloned;
                    state.selected_index = state
                        .selected_index
                        .min(state.filtered.len().saturating_sub(1));
                }
                populate_list_box(list_box, &state_ref.borrow());
                update_status(status, &state_ref.borrow());
            }
        }
    }
}

fn populate_list_box(list_box: &gtk::ListBox, state: &PickerState) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    for entry in &state.filtered {
        let label_text = if entry.sensitive {
            "\u{1f512}  Sensitive item".to_string()
        } else {
            picker::format_external_label(entry, false)
        };
        let label = gtk::Label::builder()
            .label(&label_text)
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();
        list_box.append(&label);
    }
}

fn update_status(label: &gtk::Label, state: &PickerState) {
    let total = state.filtered.len();
    let selected = state.selected_index + 1;
    if total == 0 {
        label.set_text("No items found");
    } else {
        label.set_text(&format!(
            "{selected}/{total}  \u{00b7}  \u{2191}\u{2193} navigate  \u{00b7}  Enter copy  \u{00b7}  Ctrl+1-9 quick  \u{00b7}  Esc close"
        ));
    }
}
