//! Compositor and display server detection utilities.

/// Identifies the current display server / compositor environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayServer {
    /// Generic Wayland session. Protocol support is checked at runtime.
    Wayland,
    /// Wayland session that is likely wlroots-based.
    WaylandLikelyWlroots,
    /// COSMIC Wayland session missing the data-control enablement env var.
    CosmicWaylandNeedsDataControlEnv,
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
    detect_display_server_from(|key| std::env::var(key).ok())
}

fn detect_display_server_from(get_env: impl Fn(&str) -> Option<String>) -> DisplayServer {
    let wayland_display = get_env("WAYLAND_DISPLAY").is_some();
    let display = get_env("DISPLAY").is_some();
    let cosmic_data_control = get_env("COSMIC_DATA_CONTROL_ENABLED")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    let desktop_tokens = [
        get_env("XDG_CURRENT_DESKTOP"),
        get_env("XDG_SESSION_DESKTOP"),
        get_env("DESKTOP_SESSION"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(":")
    .to_lowercase();

    let is_hyprland =
        get_env("HYPRLAND_INSTANCE_SIGNATURE").is_some() || desktop_tokens.contains("hyprland");
    let is_sway_or_wlroots = desktop_tokens.contains("sway") || desktop_tokens.contains("wlroots");
    let is_cosmic = desktop_tokens.contains("cosmic");

    if wayland_display {
        if is_hyprland || is_sway_or_wlroots {
            DisplayServer::WaylandLikelyWlroots
        } else if is_cosmic && !cosmic_data_control {
            DisplayServer::CosmicWaylandNeedsDataControlEnv
        } else {
            DisplayServer::Wayland
        }
    } else if display {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}

/// Return a user-facing error message describing what's wrong and how to fix it.
pub fn get_compositor_help(server: &DisplayServer) -> Option<&'static str> {
    match server {
        DisplayServer::Wayland | DisplayServer::WaylandLikelyWlroots => None,
        DisplayServer::CosmicWaylandNeedsDataControlEnv => Some(
            "COSMIC Wayland sessions need COSMIC_DATA_CONTROL_ENABLED=1 for clipboard history.\n\
             \n\
             On COSMIC desktop:\n\
             - This is usually set automatically. Check your session settings.\n\
             - You can set it in /etc/environment or ~/.profile:\n\
               export COSMIC_DATA_CONTROL_ENABLED=1\n\
             \n\
             Hyprland and Sway do not use this COSMIC-specific variable; \
             their support is verified by attempting to bind wlr-data-control at startup.",
        ),
        DisplayServer::X11 => Some(
            "author-clipboard requires a Wayland compositor with wlr-data-control support.\n\
             X11/Xorg is not supported.\n\
             \n\
             To use author-clipboard:\n\
             - Switch to a Wayland session (COSMIC, Sway, Hyprland, etc.)\n\
             - On COSMIC: log out and select 'COSMIC' session (not 'COSMIC (X11)')\n\
             - On Hyprland/Sway: ensure the compositor exposes wlr-data-control",
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
    use std::collections::HashMap;

    fn detect(vars: &[(&str, &str)]) -> DisplayServer {
        let env = vars.iter().copied().collect::<HashMap<_, _>>();
        detect_display_server_from(|key| env.get(key).map(|value| (*value).to_string()))
    }

    #[test]
    fn test_x11_help_is_actionable() {
        let help = get_compositor_help(&DisplayServer::X11);
        assert!(help.is_some());
        let msg = help.unwrap();
        assert!(msg.contains("X11"));
        assert!(msg.contains("Wayland"));
    }

    #[test]
    fn test_wayland_no_help() {
        assert!(get_compositor_help(&DisplayServer::Wayland).is_none());
        assert!(get_compositor_help(&DisplayServer::WaylandLikelyWlroots).is_none());
    }

    #[test]
    fn test_cosmic_help_mentions_env_var() {
        let help = get_compositor_help(&DisplayServer::CosmicWaylandNeedsDataControlEnv);
        assert!(help.unwrap().contains("COSMIC_DATA_CONTROL_ENABLED"));
    }

    #[test]
    fn test_detect_hyprland_wayland() {
        assert_eq!(
            detect(&[
                ("WAYLAND_DISPLAY", "wayland-1"),
                ("HYPRLAND_INSTANCE_SIGNATURE", "abc")
            ]),
            DisplayServer::WaylandLikelyWlroots
        );
    }

    #[test]
    fn test_detect_sway_wayland() {
        assert_eq!(
            detect(&[
                ("WAYLAND_DISPLAY", "wayland-1"),
                ("XDG_CURRENT_DESKTOP", "sway")
            ]),
            DisplayServer::WaylandLikelyWlroots
        );
    }

    #[test]
    fn test_detect_cosmic_with_env_var() {
        assert_eq!(
            detect(&[
                ("WAYLAND_DISPLAY", "wayland-1"),
                ("XDG_CURRENT_DESKTOP", "COSMIC"),
                ("COSMIC_DATA_CONTROL_ENABLED", "1")
            ]),
            DisplayServer::Wayland
        );
    }

    #[test]
    fn test_detect_cosmic_missing_env_var() {
        assert_eq!(
            detect(&[
                ("WAYLAND_DISPLAY", "wayland-1"),
                ("XDG_CURRENT_DESKTOP", "COSMIC")
            ]),
            DisplayServer::CosmicWaylandNeedsDataControlEnv
        );
    }

    #[test]
    fn test_detect_x11_only_session() {
        assert_eq!(detect(&[("DISPLAY", ":0")]), DisplayServer::X11);
    }

    #[test]
    fn test_detect_unknown_session() {
        assert_eq!(detect(&[]), DisplayServer::Unknown);
    }
}
