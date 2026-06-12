//! AdwApplicationWindow manager. Real implementation in T015.
//!
//! For the bug-fix slice (T016-T018) we ship a minimal manager with
//! a real `AdwApplicationWindow` and `AdwToolbarView`, so launching
//! `author-clipboard` from a terminal shows a proper window
//! (US-003) instead of the previous 520×700 broken pane.

use crate::ManagerConfig;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::AdwApplicationWindowExt;

/// Run the manager window. Blocks until the window closes.
pub fn run(config: ManagerConfig) -> anyhow::Result<()> {
    tracing::info!(?config, "ui-gtk manager starting");

    let app = adw::Application::builder()
        .application_id("com.namikofficial.author-clipboard")
        .build();

    app.connect_activate(move |app| {
        if let Err(e) = build_manager_window(app) {
            tracing::error!(?e, "failed to build manager");
            app.quit();
        }
    });

    let args: Vec<String> = vec!["author-clipboard".to_string()];
    let _ = app.run_with_args(&args);
    Ok(())
}

fn build_manager_window(app: &adw::Application) -> anyhow::Result<()> {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Clipboard Manager")
        .default_width(1100)
        .default_height(720)
        .build();

    // ── Headerbar ──────────────────────────────────────────────────
    let header = adw::HeaderBar::new();
    let view_switcher = adw::ViewSwitcher::new();
    view_switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
    header.set_title_widget(Some(&view_switcher));

    // ── View stack with 6 pages (placeholders) ─────────────────────
    let stack = adw::ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    for (id, title) in [
        ("clipboard", "Clipboard"),
        ("emoji", "Emoji"),
        ("symbols", "Symbols"),
        ("kaomoji", "Kaomoji"),
        ("snippets", "Snippets"),
        ("settings", "Settings"),
    ] {
        let page = build_placeholder_page(title, id);
        stack.add_titled(&page, Some(id), title);
    }
    view_switcher.set_stack(Some(&stack));

    // ── Toast overlay + status bar ─────────────────────────────────
    let status = gtk4::Label::new(Some("8 items · 2 pinned · ● Daemon · 🔒 Incognito"));
    status.set_margin_top(4);
    status.set_margin_bottom(4);
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    status_box.append(&status);
    status_box.set_margin_start(12);
    status_box.set_margin_end(12);

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    vbox.append(&stack);
    vbox.append(&separator());
    vbox.append(&status_box);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&vbox));

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&toolbar));

    window.set_content(Some(&toast_overlay));

    // ── Esc to close (US-001 in manager mode) ──────────────────────
    let window_for_esc = window.clone();
    let esc_controller = gtk4::EventControllerKey::new();
    esc_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    esc_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_for_esc.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(esc_controller);

    window.present();
    Ok(())
}

fn build_placeholder_page(title: &str, id: &str) -> gtk4::Box {
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    vbox.set_margin_top(24);
    vbox.set_margin_bottom(24);
    vbox.set_margin_start(24);
    vbox.set_margin_end(24);

    let heading = gtk4::Label::new(Some(title));
    heading.set_halign(gtk4::Align::Start);
    let attrs = gtk4::pango::AttrList::new();
    attrs.insert(gtk4::pango::AttrInt::new_weight(gtk4::pango::Weight::Bold));
    heading.set_attributes(Some(&attrs));
    heading.set_markup(&format!("<span weight=\"bold\" size=\"x-large\">{title}</span>"));

    let body = gtk4::Label::new(Some(&format!(
        "Page '{id}' — full implementation lands in T013-T015.\n\
         Filter, search, and item list widgets are being ported in\n\
         subsequent tasks. This skeleton proves the manager window\n\
         is a real AdwApplicationWindow (US-003 fix)."
    )));
    body.set_halign(gtk4::Align::Start);
    body.set_wrap(true);
    body.set_xalign(0.0);
    body.set_yalign(0.0);

    vbox.append(&heading);
    vbox.append(&body);
    vbox
}

fn separator() -> gtk4::Separator {
    gtk4::Separator::new(gtk4::Orientation::Horizontal)
}
