//! Screen lock detection for clearing sensitive clipboard items.
//!
//! Supports detection via `loginctl` and D-Bus `org.freedesktop.ScreenSaver`.

use std::process::Command;
use tracing::debug;

/// Check if the screen is currently locked.
///
/// Tries `loginctl` first (systemd-based), then falls back to
/// checking D-Bus `org.freedesktop.ScreenSaver`.
pub fn is_screen_locked() -> bool {
    if let Some(locked) = check_loginctl() {
        return locked;
    }
    if let Some(locked) = check_dbus_screensaver() {
        return locked;
    }
    debug!("Could not determine screen lock state");
    false
}

/// Check screen lock via `loginctl show-session`.
fn check_loginctl() -> Option<bool> {
    // Get the current session ID
    let session_output = Command::new("loginctl")
        .args(["show-session", "--property=LockedHint", "--value", "auto"])
        .output()
        .ok()?;

    if !session_output.status.success() {
        // Try without 'auto' — get session list first
        let list_output = Command::new("loginctl")
            .args(["list-sessions", "--no-legend", "--no-pager"])
            .output()
            .ok()?;

        let list_str = String::from_utf8_lossy(&list_output.stdout);
        let session_id = list_str.lines().next()?.split_whitespace().next()?;

        let output = Command::new("loginctl")
            .args([
                "show-session",
                session_id,
                "--property=LockedHint",
                "--value",
            ])
            .output()
            .ok()?;

        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("loginctl LockedHint={value}");
        return Some(value == "yes");
    }

    let value = String::from_utf8_lossy(&session_output.stdout)
        .trim()
        .to_string();
    debug!("loginctl LockedHint={value}");
    Some(value == "yes")
}

/// Check screen lock via D-Bus `ScreenSaver` interface.
fn check_dbus_screensaver() -> Option<bool> {
    let output = Command::new("dbus-send")
        .args([
            "--session",
            "--dest=org.freedesktop.ScreenSaver",
            "--type=method_call",
            "--print-reply",
            "/org/freedesktop/ScreenSaver",
            "org.freedesktop.ScreenSaver.GetActive",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        debug!("D-Bus ScreenSaver not available");
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Response looks like: "   boolean true" or "   boolean false"
    let locked = stdout.contains("boolean true");
    debug!("D-Bus ScreenSaver.GetActive={locked}");
    Some(locked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_screen_locked_returns_bool() {
        // This is a smoke test — actual result depends on environment
        let _result = is_screen_locked();
    }
}
