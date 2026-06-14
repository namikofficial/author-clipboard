//! Layer-shell popup. Real implementation in T014.
//!
//! For the bug-fix slice (T016-T018) we ship a functional popup
//! that uses the real `pages::clipboard` widget (data from IPC),
//! the `controller::focus` Esc handler (US-001), and `/` to focus
//! the search entry (US-002).

use crate::controller::focus::{resolve_escape, EscOutcome, FocusTarget};
use crate::PopupConfig;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use libadwaita as adw;
use libadwaita::prelude::AdwWindowExt;

/// Run the popup. Blocks until the window closes.
pub fn run(config: PopupConfig) -> anyhow::Result<()> {
    tracing::info!(?config, "ui-gtk popup starting");

    let app = adw::Application::builder()
        .application_id("com.namikofficial.author-clipboard.popup")
        .build();

    app.connect_activate(move |app| {
        if let Err(e) = build_popup(app, &config) {
            tracing::error!(?e, "failed to build popup");
            app.quit();
        }
    });

    let args: Vec<String> = vec!["author-clipboard-popup".to_string()];
    let _ = app.run_with_args(&args);
    Ok(())
}

// The function never returns Err today; the `Result` signature is kept
// for future expansion (e.g. resource loading errors).
#[allow(clippy::unnecessary_wraps, clippy::too_many_lines)]
fn build_popup(app: &adw::Application, config: &PopupConfig) -> anyhow::Result<()> {
    let window = adw::Window::builder()
        .application(app)
        .title("Clipboard")
        .default_width(720)
        .default_height(520)
        .resizable(false)
        .build();

    // ── Layer-shell init ─────────────────────────────────────
    if gtk4_layer_shell::is_supported() {
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    } else {
        tracing::warn!("layer-shell not supported; popup will use XDG window");
    }

    // ── Real clipboard page (data via IPC) ────────────────────
    let props = crate::pages::clipboard::ClipboardPageProps {
        initial_query: config.query.clone().unwrap_or_default(),
        initial_filter: config.filter,
        count: config.count,
    };
    let window_for_copy = window.clone();
    let page = crate::pages::clipboard::build(&props, move |req| {
        tracing::info!(id = req.id, mime = %req.mime, "popup copy");
        if let Err(e) = crate::pages::clipboard::copy_via_ipc(req.id, &req.mime) {
            tracing::warn!(?e, "popup copy failed");
        }
        // US-001: close after a successful copy (or failure — we'd rather
        // lose the popup than keep it open if the user pressed Enter).
        window_for_copy.close();
    });

    let status = gtk4::Label::new(Some("↑↓ navigate · / search · Enter copy · Esc close"));
    status.set_margin_top(4);
    status.set_margin_bottom(4);
    status.set_halign(gtk4::Align::Start);
    status.set_margin_start(12);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.append(&page);
    content.append(&status);

    window.set_content(Some(&content));

    // ── US-001: global Esc handler in Capture phase ──────────
    let focus_state: std::rc::Rc<std::cell::RefCell<FocusTarget>> =
        std::rc::Rc::new(std::cell::RefCell::new(FocusTarget::List));
    let content_for_esc: gtk4::Widget = content.clone().upcast();
    let focus_state_for_esc = focus_state.clone();
    let esc_controller = gtk4::EventControllerKey::new();
    esc_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    esc_controller.connect_key_pressed(move |_, key, _, _| {
        if key != gdk::Key::Escape {
            return glib::Propagation::Proceed;
        }
        let search = find_search_entry(&content_for_esc);
        let list = find_list_box(&content_for_esc);
        let query_empty = search.as_ref().is_none_or(|s| s.text().is_empty());
        let focus = *focus_state_for_esc.borrow();
        let outcome = resolve_escape(focus, query_empty);
        match outcome {
            EscOutcome::ClearSearch => {
                if let Some(s) = &search {
                    s.set_text("");
                }
                if let Some(l) = &list {
                    l.grab_focus();
                }
            }
            EscOutcome::BlurSearch => {
                if let Some(l) = &list {
                    l.grab_focus();
                }
            }
            EscOutcome::Close => {
                if let Some(w) = find_window(&content_for_esc) {
                    w.close();
                }
            }
            EscOutcome::Proceed => {}
        }
        glib::Propagation::Stop
    });
    window.add_controller(esc_controller);

    // ── `/` focuses the search entry ──────────────────────────
    let content_for_slash: gtk4::Widget = content.clone().upcast();
    let slash = gtk4::EventControllerKey::new();
    slash.set_propagation_phase(gtk4::PropagationPhase::Capture);
    slash.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::slash {
            if let Some(s) = find_search_entry(&content_for_slash) {
                if !s.has_focus() {
                    s.grab_focus();
                    return glib::Propagation::Stop;
                }
            }
        }
        glib::Propagation::Proceed
    });
    window.add_controller(slash);

    // ── Track focus changes (for the focus_state tracker) ─────
    let focus_state_for_search = focus_state.clone();
    let content_for_search_tracker: gtk4::Widget = content.clone().upcast();
    if let Some(search) = find_search_entry(&content_for_search_tracker) {
        search.connect_has_focus_notify(move |entry| {
            if entry.has_focus() {
                *focus_state_for_search.borrow_mut() = FocusTarget::Search;
            } else {
                *focus_state_for_search.borrow_mut() = FocusTarget::List;
            }
        });
    }
    let focus_state_for_list = focus_state.clone();
    let content_for_list_tracker: gtk4::Widget = content.clone().upcast();
    if let Some(list) = find_list_box(&content_for_list_tracker) {
        list.connect_row_selected(move |_, _| {
            *focus_state_for_list.borrow_mut() = FocusTarget::List;
        });
    }

    // ── US-002: open with the list focused ───────────────────
    let content_for_focus: gtk4::Widget = content.clone().upcast();
    window.connect_map(move |_| {
        if let Some(list) = find_list_box(&content_for_focus) {
            list.grab_focus();
        }
    });

    window.present();
    Ok(())
}

fn find_search_entry(widget: &gtk4::Widget) -> Option<gtk4::SearchEntry> {
    if let Ok(entry) = widget.clone().downcast::<gtk4::SearchEntry>() {
        return Some(entry);
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(found) = find_search_entry(&c) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

fn find_list_box(widget: &gtk4::Widget) -> Option<gtk4::ListBox> {
    if let Ok(list) = widget.clone().downcast::<gtk4::ListBox>() {
        return Some(list);
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(found) = find_list_box(&c) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

fn find_window(widget: &gtk4::Widget) -> Option<adw::Window> {
    if let Ok(w) = widget.clone().downcast::<adw::Window>() {
        return Some(w);
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(found) = find_window(&c) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}
