//! `AdwApplicationWindow` manager. Real implementation in T015.
//!
//! For the bug-fix slice (T016-T018) we ship a minimal manager with
//! a real `AdwApplicationWindow` and `AdwToolbarView`, so launching
//! `author-clipboard` from a terminal shows a proper window
//! (US-003) instead of the previous 520×700 broken pane.
//!
//! T015 brings the 6 pages: Clipboard (real, with IPC), Emoji /
//! Symbols / Kaomoji (real, with `shared::emoji/kaomoji/symbols`
//! data), Snippets (real, with snippet DB), Settings (real, with
//! `AdwPreferencesWindow`).

use crate::ManagerConfig;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::AdwApplicationWindowExt;

/// Run the manager window. Blocks until the window closes.
//
// `ManagerConfig` is taken by value for parity with `run_popup`,
// even though it is only borrowed in a single call below.
#[allow(clippy::needless_pass_by_value)]
pub fn run(config: ManagerConfig) -> anyhow::Result<()> {
    tracing::info!(?config, "ui-gtk manager starting");

    let app = adw::Application::builder()
        .application_id("com.namikofficial.author-clipboard")
        .flags(adw::gio::ApplicationFlags::NON_UNIQUE)
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

// Signature kept for future expansion (e.g. loading errors).
#[allow(clippy::unnecessary_wraps)]
fn build_manager_window(app: &adw::Application) -> anyhow::Result<()> {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Clipboard Manager")
        .default_width(1100)
        .default_height(720)
        .build();

    // ── Headerbar with view switcher ──────────────────────────
    let header = adw::HeaderBar::new();
    let view_switcher = adw::ViewSwitcher::new();
    view_switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
    header.set_title_widget(Some(&view_switcher));

    // ── View stack with 6 pages ───────────────────────────────
    let stack = adw::ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    // Page 1: Clipboard (real, with IPC)
    let shared_config = author_clipboard_shared::config::Config::load();
    let clipboard_page = crate::pages::clipboard::build(&shared_config, |_id, _mime| {
        // In manager mode, we don't close on copy — the user may
        // want to copy multiple items. Toast handling lives in the
        // toast overlay above the stack.
        tracing::info!("manager copy");
    });
    stack.add_titled(&clipboard_page, Some("clipboard"), "Clipboard");

    // Page 2-4: Emoji / Symbols / Kaomoji (real, read-only grid)
    stack.add_titled(&crate::pages::emoji::build(), Some("emoji"), "Emoji");
    stack.add_titled(&crate::pages::symbols::build(), Some("symbols"), "Symbols");
    stack.add_titled(&crate::pages::kaomoji::build(), Some("kaomoji"), "Kaomoji");

    // Page 5: Snippets (real, with snippet DB)
    let snippets_page = crate::pages::snippets::build(&shared_config);
    stack.add_titled(&snippets_page, Some("snippets"), "Snippets");

    // Page 6: Settings (real, AdwPreferencesWindow pattern)
    let settings_page = crate::pages::settings::build(&shared_config);
    stack.add_titled(&settings_page, Some("settings"), "Settings");

    view_switcher.set_stack(Some(&stack));

    // ── Status bar ────────────────────────────────────────────
    let status = gtk4::Label::new(Some("● Daemon"));
    status.set_margin_top(4);
    status.set_margin_bottom(4);
    status.set_halign(gtk4::Align::Start);
    status.set_margin_start(12);
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    status_box.append(&status);
    status_box.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
    status_box.append(&gtk4::Label::new(Some("Tip: / focuses search in popups")));

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    vbox.append(&stack);
    vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    vbox.append(&status_box);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&vbox));

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&toolbar));

    window.set_content(Some(&toast_overlay));

    // ── Esc to close ─────────────────────────────────────────
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
