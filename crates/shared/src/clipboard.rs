//! Clipboard restore helpers shared by the applet and CLI picker.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::image_store;
use crate::types::ClipboardItem;

/// Errors returned while restoring an item to the Wayland clipboard.
#[derive(Debug, thiserror::Error)]
pub enum ClipboardSetError {
    /// Failed to spawn or communicate with `wl-copy`.
    #[error("wl-copy failed: {0}")]
    Io(#[from] std::io::Error),
    /// Image data could not be read from disk.
    #[error("failed to read image: {0}")]
    ImageRead(std::io::Error),
    /// `wl-copy` exited with an unsuccessful status.
    #[error("wl-copy exited with status {status}: {stderr}")]
    CommandFailed {
        /// Process exit status rendered for humans.
        status: String,
        /// Standard error output from the failed command.
        stderr: String,
    },
}

/// Result metadata for a clipboard restore operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardSetResult {
    /// MIME type offered to the compositor.
    pub mime_type: String,
    /// Human-readable behavior summary.
    pub behavior: &'static str,
}

/// Restore a clipboard history item to the Wayland clipboard.
pub fn set_clipboard_item(
    item: &ClipboardItem,
    data_dir: &Path,
) -> Result<ClipboardSetResult, ClipboardSetError> {
    if item.is_image() {
        let path = image_store::image_path(data_dir, &item.content);
        set_clipboard_image(&path, &item.mime_type)
    } else if item.is_html() {
        set_clipboard_html(&item.content)
    } else if item.is_files() {
        set_clipboard_files(&item.content)
    } else {
        set_clipboard_text(&item.content)
    }
}

/// Set plain text clipboard content.
pub fn set_clipboard_text(content: &str) -> Result<ClipboardSetResult, ClipboardSetError> {
    run_wl_copy(None, content.as_bytes())?;
    Ok(ClipboardSetResult {
        mime_type: "text/plain".to_string(),
        behavior: "text/plain",
    })
}

/// Set HTML clipboard content as `text/html`.
pub fn set_clipboard_html(html: &str) -> Result<ClipboardSetResult, ClipboardSetError> {
    run_wl_copy(Some("text/html"), html.as_bytes())?;
    Ok(ClipboardSetResult {
        mime_type: "text/html".to_string(),
        behavior: "text/html",
    })
}

/// Set file URI list clipboard content as `text/uri-list`.
pub fn set_clipboard_files(uri_list: &str) -> Result<ClipboardSetResult, ClipboardSetError> {
    run_wl_copy(Some("text/uri-list"), uri_list.as_bytes())?;
    Ok(ClipboardSetResult {
        mime_type: "text/uri-list".to_string(),
        behavior: "text/uri-list",
    })
}

/// Set image clipboard content with its original MIME type.
pub fn set_clipboard_image(
    path: &Path,
    mime_type: &str,
) -> Result<ClipboardSetResult, ClipboardSetError> {
    let data = std::fs::read(path).map_err(ClipboardSetError::ImageRead)?;
    run_wl_copy(Some(mime_type), &data)?;
    Ok(ClipboardSetResult {
        mime_type: mime_type.to_string(),
        behavior: "image",
    })
}

fn run_wl_copy(mime_type: Option<&str>, data: &[u8]) -> Result<(), ClipboardSetError> {
    let mut command = Command::new("wl-copy");
    if let Some(mime_type) = mime_type {
        command.args(["--type", mime_type]);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(data)?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(ClipboardSetError::CommandFailed {
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::types::ClipboardItem;

    #[test]
    fn test_clipboard_item_result_types() {
        let text = ClipboardItem::new_text("hello".to_string());
        let html = ClipboardItem::new_html("<b>hello</b>".to_string(), "hello".to_string());
        let files = ClipboardItem::new_files("file:///tmp/a.txt\n".to_string());

        assert!(!text.is_html());
        assert!(html.is_html());
        assert!(files.is_files());
    }

    #[test]
    fn new_text_runs_sensitive_detection() {
        // Plain text items must be flagged when they look like
        // credentials / tokens / private keys.
        assert!(!ClipboardItem::new_text("hello world".to_string()).sensitive);
        assert!(
            ClipboardItem::new_text("ghp_1234567890abcdefghij".to_string()).sensitive,
            "text constructor must run check_sensitivity on plain text"
        );
        assert!(
            ClipboardItem::new_text("postgresql://u:p@host/db".to_string()).sensitive,
            "connection-string credentials must trigger"
        );
    }

    #[test]
    fn new_html_runs_sensitive_detection() {
        // Plain HTML with no secrets is not sensitive.
        let safe =
            ClipboardItem::new_html("<p>Hello world</p>".to_string(), "Hello world".to_string());
        assert!(!safe.sensitive, "innocent HTML must not be flagged");

        // Secret in the plain-text companion.
        let in_plain =
            ClipboardItem::new_html("<p>Copy me</p>".to_string(), "sk-abc123xyz".to_string());
        assert!(
            in_plain.sensitive,
            "secret in text/plain companion must trigger"
        );

        // Secret in an attribute value, no plain-text companion.
        let in_attr = ClipboardItem::new_html(
            r#"<form><input type="password" value="MyP@ssw0rd!" /></form>"#.to_string(),
            String::new(),
        );
        assert!(
            in_attr.sensitive,
            "secret in HTML attribute value must trigger"
        );

        // Credential URI in an attribute.
        let in_uri = ClipboardItem::new_html(
            r#"<img src="postgresql://admin:secret@db.example.com/x" />"#.to_string(),
            String::new(),
        );
        assert!(
            in_uri.sensitive,
            "credential URI in HTML attribute must trigger"
        );
    }

    #[test]
    fn new_files_runs_sensitive_detection() {
        // An innocent file list is not sensitive.
        let safe = ClipboardItem::new_files(
            "file:///home/me/notes.txt\nfile:///home/me/todo.md".to_string(),
        );
        assert!(!safe.sensitive, "innocent file list must not be flagged");

        // A file list containing a URI with embedded credentials is
        // flagged.
        let with_creds = ClipboardItem::new_files(
            "file:///home/me/db.dump\npostgresql://admin:secret@db.example.com/x".to_string(),
        );
        assert!(
            with_creds.sensitive,
            "credential URI in file list must trigger"
        );
    }
}
