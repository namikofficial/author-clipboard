//! Compositor and display server detection utilities.

/// Identifies the current display server / compositor environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayServer {
    /// Wayland with wlr-data-control support (full functionality)
    WaylandWithDataControl,
    /// Wayland without wlr-data-control (limited/no clipboard history)
    WaylandNoDataControl,
    /// X11/Xorg display server (not supported)
    X11,
    /// Unknown display server
    Unknown,
}

/// Detect the current display server environment.
///
/// Returns an enum indicating what level of support is available.
/// Does NOT connect to Wayland — just checks environment variables.
pub fn detect_display_server() -> DisplayServer {
    let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
    let display = std::env::var("DISPLAY").is_ok();
    let cosmic_data_control = std::env::var("COSMIC_DATA_CONTROL_ENABLED")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    match (wayland_display, display, cosmic_data_control) {
        (true, _, true) => DisplayServer::WaylandWithDataControl,
        (true, _, false) => DisplayServer::WaylandNoDataControl,
        (false, true, _) => DisplayServer::X11,
        _ => DisplayServer::Unknown,
    }
}

/// Return a user-facing error message describing what's wrong and how to fix it.
pub fn get_compositor_help(server: &DisplayServer) -> Option<&'static str> {
    match server {
        DisplayServer::WaylandWithDataControl => None,
        DisplayServer::WaylandNoDataControl => Some(
            "author-clipboard requires COSMIC_DATA_CONTROL_ENABLED=1 to be set.\n\
             \n\
             On COSMIC desktop:\n\
             - This is usually set automatically. Check your session settings.\n\
             - You can set it in /etc/environment or ~/.profile:\n\
               export COSMIC_DATA_CONTROL_ENABLED=1\n\
             \n\
             On other Wayland compositors (Sway, Hyprland, etc.):\n\
             - The wlr-data-control protocol must be supported by your compositor.\n\
             - Check: compositor_name --list-protocols | grep data-control\n\
             - COSMIC_DATA_CONTROL_ENABLED is not needed on other wlroots compositors.",
        ),
        DisplayServer::X11 => Some(
            "author-clipboard requires a Wayland compositor with wlr-data-control support.\n\
             X11/Xorg is not supported.\n\
             \n\
             To use author-clipboard:\n\
             - Switch to a Wayland session (COSMIC, Sway, Hyprland, KDE Wayland, etc.)\n\
             - On COSMIC: log out and select 'COSMIC' session (not 'COSMIC (X11)')\n\
             - Ensure COSMIC_DATA_CONTROL_ENABLED=1 is set for COSMIC desktop",
        ),
        DisplayServer::Unknown => Some(
            "Could not detect display server. Ensure WAYLAND_DISPLAY is set.\n\
             author-clipboard requires Wayland with wlr-data-control protocol support.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x11_help_is_actionable() {
        let help = get_compositor_help(&DisplayServer::X11);
        assert!(help.is_some());
        let msg = help.unwrap();
        assert!(msg.contains("X11"));
        assert!(msg.contains("Wayland"));
    }

    #[test]
    fn test_wayland_with_control_no_help() {
        assert!(get_compositor_help(&DisplayServer::WaylandWithDataControl).is_none());
    }

    #[test]
    fn test_no_data_control_help_mentions_env_var() {
        let help = get_compositor_help(&DisplayServer::WaylandNoDataControl);
        assert!(help.unwrap().contains("COSMIC_DATA_CONTROL_ENABLED"));
    }
}
