//! Quick paste module for typing clipboard text into the active window.
//!
//! Wraps Wayland text input tools (`wtype`, `ydotool`, `wl-copy`) to
//! simulate keyboard input from clipboard history.

use std::fmt;
use std::process::Command;

/// Available backends for pasting text into applications.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PasteBackend {
    /// Wayland text typer (preferred).
    Wtype,
    /// Generic input tool (works on both X11 and Wayland with root).
    Ydotool,
    /// Fallback: just copies text to clipboard via `wl-copy`.
    WlCopy,
}

impl fmt::Display for PasteBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wtype => write!(f, "wtype (Wayland keyboard simulator)"),
            Self::Ydotool => write!(f, "ydotool (generic input tool)"),
            Self::WlCopy => write!(f, "wl-copy (clipboard fallback)"),
        }
    }
}

/// Result of a paste operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PasteResult {
    /// Whether the paste succeeded.
    pub success: bool,
    /// Which backend was used.
    pub backend_used: PasteBackend,
    /// Optional status message or error detail.
    pub message: Option<String>,
}

/// Whether a paste operation is permitted.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PastePermission {
    /// Paste is allowed.
    Allowed,
    /// Denied because content is marked sensitive.
    DeniedSensitive,
    /// Denied because no paste backend is available.
    DeniedNoBackend,
    /// User must confirm before pasting sensitive content.
    RequiresConfirmation,
}

/// Check whether a command-line tool is available on `$PATH`.
fn tool_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Detect the best available paste backend.
///
/// Checks in order: `wtype` → `ydotool` → `wl-copy`.
#[allow(dead_code)]
pub fn detect_backend() -> Option<PasteBackend> {
    if tool_exists("wtype") {
        Some(PasteBackend::Wtype)
    } else if tool_exists("ydotool") {
        Some(PasteBackend::Ydotool)
    } else if tool_exists("wl-copy") {
        Some(PasteBackend::WlCopy)
    } else {
        None
    }
}

/// Check whether any paste backend is available.
#[allow(dead_code)]
pub fn is_available() -> bool {
    detect_backend().is_some()
}

/// Determine whether a paste is permitted given content sensitivity.
///
/// # Rules
/// - Sensitive content that hasn't been confirmed → `RequiresConfirmation`
/// - No backend detected → `DeniedNoBackend`
/// - Otherwise → `Allowed`
#[allow(dead_code)]
pub fn check_paste_permission(
    _content: &str,
    sensitive: bool,
    user_confirmed: bool,
) -> PastePermission {
    if sensitive && !user_confirmed {
        return PastePermission::RequiresConfirmation;
    }
    if detect_backend().is_none() {
        return PastePermission::DeniedNoBackend;
    }
    PastePermission::Allowed
}

/// Paste text using the specified backend.
///
/// # Errors
///
/// Returns an error if the backend process fails to spawn.
#[allow(dead_code)]
pub fn quick_paste(text: &str, backend: &PasteBackend) -> std::io::Result<PasteResult> {
    let output = match backend {
        PasteBackend::Wtype => Command::new("wtype").arg("--").arg(text).output(),
        PasteBackend::Ydotool => Command::new("ydotool")
            .arg("type")
            .arg("--")
            .arg(text)
            .output(),
        PasteBackend::WlCopy => Command::new("wl-copy").arg("--").arg(text).output(),
    }?;

    let success = output.status.success();
    let message = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).into_owned())
    };

    Ok(PasteResult {
        success,
        backend_used: backend.clone(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paste_backend_display() {
        assert_eq!(
            PasteBackend::Wtype.to_string(),
            "wtype (Wayland keyboard simulator)"
        );
        assert_eq!(
            PasteBackend::Ydotool.to_string(),
            "ydotool (generic input tool)"
        );
        assert_eq!(
            PasteBackend::WlCopy.to_string(),
            "wl-copy (clipboard fallback)"
        );
    }

    #[test]
    fn test_permission_sensitive_unconfirmed() {
        let perm = check_paste_permission("secret-token", true, false);
        assert_eq!(perm, PastePermission::RequiresConfirmation);
    }

    #[test]
    fn test_permission_sensitive_confirmed() {
        // When confirmed, result depends on backend availability—but it should
        // NOT be RequiresConfirmation.
        let perm = check_paste_permission("secret-token", true, true);
        assert_ne!(perm, PastePermission::RequiresConfirmation);
        assert_ne!(perm, PastePermission::DeniedSensitive);
    }

    #[test]
    fn test_permission_not_sensitive() {
        let perm = check_paste_permission("hello world", false, false);
        // Should never require confirmation for non-sensitive content.
        assert_ne!(perm, PastePermission::RequiresConfirmation);
        assert_ne!(perm, PastePermission::DeniedSensitive);
    }

    #[test]
    fn test_paste_result_creation() {
        let result = PasteResult {
            success: true,
            backend_used: PasteBackend::Wtype,
            message: None,
        };
        assert!(result.success);
        assert_eq!(result.backend_used, PasteBackend::Wtype);
        assert!(result.message.is_none());

        let result_fail = PasteResult {
            success: false,
            backend_used: PasteBackend::WlCopy,
            message: Some("command not found".to_string()),
        };
        assert!(!result_fail.success);
        assert_eq!(result_fail.message.as_deref(), Some("command not found"));
    }

    #[test]
    fn test_detect_backend_returns_option() {
        // Just verify it doesn't panic; result depends on the host system.
        let _backend = detect_backend();
    }
}
