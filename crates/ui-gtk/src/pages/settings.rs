//! Settings page. Uses `AdwPreferencesGroup` and friends to render
//! the 5 groups (Status, Privacy, History, Data, About) from the
//! applet's old `view_settings`.
//!
//! All changes write through to `Config::save()`. The settings
//! take effect immediately for the UI; the daemon reads them at
//! next start.

use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Label, Orientation, SpinButton, Switch, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the settings page widget.
pub fn build(config: &author_clipboard_shared::config::Config) -> Widget {
    let scrolled = gtk4::ScrolledWindow::builder()
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let prefs = adw::PreferencesPage::new();

    // ── Status ────────────────────────────────────────────────
    let status_group = adw::PreferencesGroup::builder()
        .title("Status")
        .description("Daemon connection")
        .build();
    let daemon_row = adw::ActionRow::builder()
        .title("Daemon")
        .subtitle("author-clipboard-daemon must be running for capture to work")
        .build();
    let daemon_btn = gtk4::Button::with_label(if check_daemon() {
        "● Running"
    } else {
        "○ Not running"
    });
    daemon_btn.set_valign(gtk4::Align::Center);
    if check_daemon() {
        daemon_btn.add_css_class("success");
    } else {
        daemon_btn.add_css_class("warning");
    }
    daemon_row.add_suffix(&daemon_btn);
    daemon_row.set_activatable_widget(Some(&daemon_btn));
    status_group.add(&daemon_row);
    prefs.add(&status_group);

    // ── Privacy ───────────────────────────────────────────────
    let privacy_group = adw::PreferencesGroup::builder()
        .title("Privacy")
        .description("Sensitive content handling")
        .build();
    let incognito_row = adw::ActionRow::builder()
        .title("Incognito mode")
        .subtitle("Pause clipboard capture")
        .build();
    let incognito_switch = Switch::builder()
        .active(config.is_incognito())
        .valign(gtk4::Align::Center)
        .build();
    let config_for_inc = config.clone();
    incognito_switch.connect_state_set(move |_, state| {
        let _ = config_for_inc.set_incognito(state);
        glib::Propagation::Proceed
    });
    incognito_row.add_suffix(&incognito_switch);
    incognito_row.set_activatable_widget(Some(&incognito_switch));
    privacy_group.add(&incognito_row);

    let clear_lock_row = adw::ActionRow::builder()
        .title("Clear history on screen lock")
        .subtitle("Remove unpinned items when the screen locks")
        .build();
    let clear_lock_switch = Switch::builder()
        .active(config.clear_on_lock)
        .valign(gtk4::Align::Center)
        .build();
    let config_for_lock = Rc::new(RefCell::new(config.clone()));
    clear_lock_switch.connect_state_set(move |_, state| {
        config_for_lock.borrow_mut().clear_on_lock = state;
        let _ = config_for_lock.borrow().save();
        glib::Propagation::Proceed
    });
    clear_lock_row.add_suffix(&clear_lock_switch);
    clear_lock_row.set_activatable_widget(Some(&clear_lock_switch));
    privacy_group.add(&clear_lock_row);

    let encrypt_row = adw::ActionRow::builder()
        .title("Encrypt sensitive items at rest")
        .subtitle("AES-256-GCM with per-item nonce")
        .build();
    let encrypt_switch = Switch::builder()
        .active(config.encrypt_sensitive)
        .valign(gtk4::Align::Center)
        .build();
    let config_for_enc = Rc::new(RefCell::new(config.clone()));
    encrypt_switch.connect_state_set(move |_, state| {
        config_for_enc.borrow_mut().encrypt_sensitive = state;
        let _ = config_for_enc.borrow().save();
        glib::Propagation::Proceed
    });
    encrypt_row.add_suffix(&encrypt_switch);
    encrypt_row.set_activatable_widget(Some(&encrypt_switch));
    privacy_group.add(&encrypt_row);
    prefs.add(&privacy_group);

    // ── History ───────────────────────────────────────────────
    let history_group = adw::PreferencesGroup::builder()
        .title("History")
        .description("Storage limits")
        .build();

    // Max items
    let max_items_row = adw::ActionRow::builder()
        .title("Max items")
        .subtitle(format!("Currently: {}", config.max_items))
        .build();
    let max_items_adj = gtk4::Adjustment::builder()
        .value(config.max_items as f64)
        .lower(10.0)
        .upper(10_000.0)
        .step_increment(10.0)
        .build();
    let max_items_spin = SpinButton::builder()
        .adjustment(&max_items_adj)
        .climb_rate(50.0)
        .digits(0)
        .valign(gtk4::Align::Center)
        .build();
    let config_for_max = Rc::new(RefCell::new(config.clone()));
    let max_items_row_for_handler = max_items_row.clone();
    max_items_spin.connect_value_changed(move |spin| {
        let val = spin.value() as usize;
        config_for_max.borrow_mut().max_items = val;
        let _ = config_for_max.borrow().save();
        max_items_row_for_handler.set_subtitle(&format!("Currently: {val}"));
    });
    max_items_row.add_suffix(&max_items_spin);
    max_items_row.set_activatable_widget(Some(&max_items_spin));
    history_group.add(&max_items_row);

    // Keep history (days)
    let days = if config.ttl_seconds == 0 {
        0u64
    } else {
        config.ttl_seconds / 86400
    };
    let ttl_row = adw::ActionRow::builder()
        .title("Keep history")
        .subtitle(if days == 0 {
            "Currently: forever".to_string()
        } else {
            format!("Currently: {days} days")
        })
        .build();
    let ttl_adj = gtk4::Adjustment::builder()
        .value(days as f64)
        .lower(0.0)
        .upper(365.0)
        .step_increment(1.0)
        .build();
    let ttl_spin = SpinButton::builder()
        .adjustment(&ttl_adj)
        .climb_rate(7.0)
        .digits(0)
        .valign(gtk4::Align::Center)
        .build();
    let config_for_ttl = Rc::new(RefCell::new(config.clone()));
    let ttl_row_for_handler = ttl_row.clone();
    ttl_spin.connect_value_changed(move |spin| {
        let val = spin.value() as u64;
        config_for_ttl.borrow_mut().ttl_seconds = val * 86400;
        let _ = config_for_ttl.borrow().save();
        if val == 0 {
            ttl_row_for_handler.set_subtitle("Currently: forever");
        } else {
            ttl_row_for_handler.set_subtitle(&format!("Currently: {val} days"));
        }
    });
    ttl_row.add_suffix(&ttl_spin);
    ttl_row.set_activatable_widget(Some(&ttl_spin));
    history_group.add(&ttl_row);

    prefs.add(&history_group);

    // ── Data ──────────────────────────────────────────────────
    let data_group = adw::PreferencesGroup::builder()
        .title("Data")
        .description("Clear, export, import")
        .build();
    let clear_row = adw::ActionRow::builder()
        .title("Clear all unpinned")
        .subtitle("Remove all unpinned items from history")
        .build();
    let clear_btn = Button::with_label("Clear");
    clear_btn.add_css_class("destructive-action");
    clear_btn.set_valign(gtk4::Align::Center);
    clear_btn.connect_clicked(|_| {
        // Best-effort: send IPC ClearUnpinned.
        if let Ok(resp) = author_clipboard_shared::ipc::IpcClient::new()
            .send_command(&author_clipboard_shared::ipc::IpcCommand::ClearUnpinned)
        {
            if resp.ok {
                tracing::info!("cleared unpinned");
            }
        }
    });
    clear_row.add_suffix(&clear_btn);
    clear_row.set_activatable_widget(Some(&clear_btn));
    data_group.add(&clear_row);
    prefs.add(&data_group);

    // ── About ─────────────────────────────────────────────────
    let about_group = adw::PreferencesGroup::builder()
        .title("About")
        .build();
    let about_row = adw::ActionRow::builder()
        .title("Author Clipboard")
        .subtitle(format!("v{}  ·  GPL-3.0  ·  COSMIC + Hyprland", env!("CARGO_PKG_VERSION")))
        .build();
    about_group.add(&about_row);
    prefs.add(&about_group);

    scrolled.set_child(Some(&prefs));
    scrolled.upcast()
}

fn check_daemon() -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", "author-clipboard-daemon"])
        .output()
        .is_ok_and(|o| o.status.success())
}
