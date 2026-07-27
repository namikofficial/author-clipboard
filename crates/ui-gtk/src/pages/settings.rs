//! Settings page. Uses `AdwPreferencesGroup` and friends to render
//! the 5 groups (Status, Privacy, History, Data, About).
//!
//! * Config is shared via `Rc<RefCell<Config>>` – all rows mutate
//!   through `ConfigPatch`, then save atomically, then notify the
//!   daemon via IPC.
//! * Daemon health is checked via IPC `Ping` (replacing `pgrep -f`)
//!   with a 5-second periodic refresh.
//! * Save failures surface through the `toast` callback.
//! * Subtitle updates happen only after a successful save.
//!
//! Routing all settings mutations through `ConfigPatch::apply_patch`
//! gives us centralised validation and a single code path for testing.

use gtk4::prelude::*;
use gtk4::{glib, Button, SpinButton, Switch, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::config::{Config, ConfigPatch};
use author_clipboard_shared::ipc::IpcCommand;

/// Toast callback – invoked on the GLib main context with a message.
pub type ToastFn = Rc<dyn Fn(&str) + 'static>;


/// Shared config handle.
type SharedConfig = Rc<RefCell<Config>>;

/// Build the settings page widget.
///
/// `config` is the shared mutable configuration. Every row reads
/// the current value at build time and writes back through
/// `ConfigPatch`. The `toast` callback is called on save failure.
#[allow(clippy::too_many_lines)]
pub fn build(
    config: SharedConfig,
    service: std::sync::Arc<dyn crate::service::ClipboardService>,
    toast: ToastFn,
) -> Widget {
    let scrolled = gtk4::ScrolledWindow::builder()
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let prefs = adw::PreferencesPage::new();

    // Toast is already Rc<dyn Fn> — cloneable by cloning Rc.

    // ── Status ────────────────────────────────────────────────
    let status_group = adw::PreferencesGroup::builder()
        .title("Status")
        .description("Daemon connection")
        .build();
    let daemon_row = adw::ActionRow::builder()
        .title("Daemon")
        .subtitle("author-clipboard-daemon must be running for capture to work")
        .build();
    let daemon_btn = gtk4::Button::with_label("○ Checking…");
    daemon_btn.set_valign(gtk4::Align::Center);
    daemon_btn.add_css_class("warning");
    daemon_row.add_suffix(&daemon_btn);
    daemon_row.set_activatable_widget(Some(&daemon_btn));
    status_group.add(&daemon_row);
    prefs.add(&status_group);

    // ── Daemon health check (IPC Ping, periodic) ──────────────
    {
        let svc = service.clone();
        let btn = daemon_btn.clone();
        let check = move || {
            let svc = svc.clone();
            let btn = btn.clone();
            glib::MainContext::default().spawn_local(async move {
                match svc.command(IpcCommand::Ping).await {
                    Ok(_) => {
                        btn.set_label("● Running");
                        btn.remove_css_class("warning");
                        btn.add_css_class("success");
                    }
                    Err(_) => {
                        btn.set_label("○ Not running");
                        btn.remove_css_class("success");
                        btn.add_css_class("warning");
                    }
                }
            });
        };
        // Initial check
        check();
        // Periodic check every 5 seconds
        let check_for_timer = check.clone();
        glib::timeout_add_local(std::time::Duration::from_secs(5), move || {
            check_for_timer();
            glib::ControlFlow::Continue
        });
        // Manual refresh on button click
        daemon_btn.connect_clicked(move |_| check());
    }

    // ── Privacy ───────────────────────────────────────────────
    let privacy_group = adw::PreferencesGroup::builder()
        .title("Privacy")
        .description("Sensitive content handling")
        .build();

    // Incognito mode
    let incognito_row = adw::ActionRow::builder()
        .title("Incognito mode")
        .subtitle("Pause clipboard capture")
        .build();
    let incognito_switch = Switch::builder()
        .active(config.borrow().is_incognito())
        .valign(gtk4::Align::Center)
        .build();
    {
        let cfg = config.clone();
        let toast = toast.clone();
        let row = incognito_row.clone();
        let svc = service.clone();
        incognito_switch.connect_state_set(move |_, state| {
            let _ = cfg.borrow_mut().set_incognito(state);
            let subtitle = if cfg.borrow().is_incognito() {
                "Capture paused"
            } else {
                "Capture active"
            };
            match cfg.borrow().save() {
                Ok(()) => {
                    row.set_subtitle(subtitle);
                    notify_daemon_config_changed(&svc);
                }
                Err(e) => toast(&format!("Failed to save setting: {e}")),
            }
            glib::Propagation::Proceed
        });
    }
    incognito_row.add_suffix(&incognito_switch);
    incognito_row.set_activatable_widget(Some(&incognito_switch));
    privacy_group.add(&incognito_row);

    // Clear on lock
    let clear_lock_row = adw::ActionRow::builder()
        .title("Clear history on screen lock")
        .subtitle(if config.borrow().clear_on_lock {
            "Will remove unpinned items when screen locks"
        } else {
            "Will NOT remove items when screen locks"
        })
        .build();
    let clear_lock_switch = Switch::builder()
        .active(config.borrow().clear_on_lock)
        .valign(gtk4::Align::Center)
        .build();
    {
        let cfg = config.clone();
        let row = clear_lock_row.clone();
        let srv = service.clone();
        let toast = toast.clone();
        clear_lock_switch.connect_state_set(move |_, state| {
            let subtitle = if state {
                "Will remove unpinned items when screen locks"
            } else {
                "Will NOT remove items when screen locks"
            };
            apply_patch_save_notify(
                &cfg,
                &ConfigPatch::ClearOnLock(state),
                &row,
                &subtitle,
                &*toast,
                &srv,
            );
            glib::Propagation::Proceed
        });
    }
    clear_lock_row.add_suffix(&clear_lock_switch);
    clear_lock_row.set_activatable_widget(Some(&clear_lock_switch));
    privacy_group.add(&clear_lock_row);

    // Encrypt sensitive
    let encrypt_row = adw::ActionRow::builder()
        .title("Encrypt sensitive items at rest")
        .subtitle(if config.borrow().encrypt_sensitive {
            "AES-256-GCM with per-item nonce"
        } else {
            "Sensitive items stored as plaintext"
        })
        .build();
    let encrypt_switch = Switch::builder()
        .active(config.borrow().encrypt_sensitive)
        .valign(gtk4::Align::Center)
        .build();
    {
        let cfg = config.clone();
        let row = encrypt_row.clone();
        let srv = service.clone();
        let toast = toast.clone();
        encrypt_switch.connect_state_set(move |_, state| {
            let subtitle = if state {
                "AES-256-GCM with per-item nonce"
            } else {
                "Sensitive items stored as plaintext"
            };
            apply_patch_save_notify(
                &cfg,
                &ConfigPatch::EncryptSensitive(state),
                &row,
                &subtitle,
                &*toast,
                &srv,
            );
            glib::Propagation::Proceed
        });
    }
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
    let current_max = config.borrow().max_items;
    let max_items_row = adw::ActionRow::builder()
        .title("Max items")
        .subtitle(format!("Currently: {current_max}"))
        .build();
    let max_items_adj = gtk4::Adjustment::builder()
        .value(f64::from(u32::try_from(current_max).unwrap_or(u32::MAX)))
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
    {
        let cfg = config.clone();
        let row = max_items_row.clone();
        let srv = service.clone();
        let toast = toast.clone();
        max_items_spin.connect_value_changed(move |spin| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let val = spin.value().round().max(0.0) as usize;
            let subtitle = format!("Currently: {val}");
            apply_patch_save_notify(
                &cfg,
                &ConfigPatch::MaxItems(val),
                &row,
                &subtitle,
                &*toast,
                &srv,
            );
        });
    }
    max_items_row.add_suffix(&max_items_spin);
    max_items_row.set_activatable_widget(Some(&max_items_spin));
    history_group.add(&max_items_row);

    // Keep history (days)
    let days = {
        let c = config.borrow();
        if c.ttl_seconds == 0 {
            0u64
        } else {
            c.ttl_seconds / 86400
        }
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
        .value(f64::from(u32::try_from(days).unwrap_or(u32::MAX)))
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
    {
        let cfg = config.clone();
        let row = ttl_row.clone();
        let srv = service.clone();
        let toast = toast.clone();
        ttl_spin.connect_value_changed(move |spin| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let val = spin.value().round().max(0.0) as u64;
            let subtitle = if val == 0 {
                "Currently: forever".to_string()
            } else {
                format!("Currently: {val} days")
            };
            apply_patch_save_notify(
                &cfg,
                &ConfigPatch::TtlDays(val),
                &row,
                &subtitle,
                &*toast,
                &srv,
            );
        });
    }
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
    {
        let row = clear_row.clone();
        let srv = service.clone();
        let toast = toast.clone();
        clear_btn.connect_clicked(move |_| {
            let srv = srv.clone();
            let row = row.clone();
            let toast = toast.clone();
            glib::MainContext::default().spawn_local(async move {
                match srv.command(IpcCommand::ClearUnpinned).await {
                    Ok(_) => row.set_subtitle("Unpinned history cleared."),
                    Err(error) => toast(&format!("Could not clear history: {error}")),
                }
            });
        });
    }
    clear_row.add_suffix(&clear_btn);
    clear_row.set_activatable_widget(Some(&clear_btn));
    data_group.add(&clear_row);
    prefs.add(&data_group);

    // ── About ─────────────────────────────────────────────────
    let about_group = adw::PreferencesGroup::builder().title("About").build();
    let about_row = adw::ActionRow::builder()
        .title("Author Clipboard")
        .subtitle(format!(
            "v{}  ·  GPL-3.0  ·  COSMIC + Hyprland",
            env!("CARGO_PKG_VERSION")
        ))
        .build();
    about_group.add(&about_row);
    prefs.add(&about_group);

    scrolled.set_child(Some(&prefs));
    scrolled.upcast()
}

/// Apply a patch, save config, update subtitle, and notify the daemon.
fn apply_patch_save_notify(
    config: &SharedConfig,
    patch: &ConfigPatch,
    row: &adw::ActionRow,
    subtitle: &str,
    toast: &dyn Fn(&str),
    service: &std::sync::Arc<dyn crate::service::ClipboardService>,
) {
    let mut c = config.borrow_mut();
    match c.apply_patch(patch) {
        Ok(true) => match c.save() {
            Ok(()) => {
                row.set_subtitle(subtitle);
                notify_daemon_config_changed(service);
            }
            Err(e) => toast(&format!("Failed to save: {e}")),
        },
        Ok(false) => { /* no change */ }
        Err(msg) => toast(&format!("Invalid value: {msg}")),
    }
}

/// Tell the daemon to reload its configuration.
fn notify_daemon_config_changed(service: &std::sync::Arc<dyn crate::service::ClipboardService>) {
    let srv = std::sync::Arc::clone(service);
    glib::MainContext::default().spawn_local(async move {
        let _ = srv
            .command(IpcCommand::UpdateConfig {
                config: serde_json::Value::Null,
            })
            .await;
    });
}
