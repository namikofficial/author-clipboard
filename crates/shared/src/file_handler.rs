//! File URI resolution and metadata utilities.
//!
//! Handles `text/uri-list` content from clipboard file-copy operations,
//! resolving `file://` URIs to filesystem metadata.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Metadata about a single file referenced from a clipboard URI list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FileInfo {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// File name (last component of the path).
    pub name: String,
    /// File size in bytes (0 if the file does not exist).
    pub size: u64,
    /// Guessed MIME type based on file extension.
    pub mime_type: String,
    /// Whether the file currently exists on disk.
    pub exists: bool,
}

/// Parse a `text/uri-list` string into resolved [`FileInfo`] entries.
///
/// Lines starting with `#` are treated as comments and skipped.
/// Each remaining line is expected to be a `file://` URI.
#[allow(dead_code)]
pub fn parse_uri_list(text: &str) -> Vec<FileInfo> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let decoded = url_decode(line.strip_prefix("file://").unwrap_or(line));
            let path = PathBuf::from(&decoded);
            resolve_file_info(&path)
        })
        .collect()
}

/// Build a [`FileInfo`] from a filesystem path.
///
/// Non-existent files are represented with `exists = false` and `size = 0`.
#[allow(dead_code)]
pub fn resolve_file_info(path: &Path) -> FileInfo {
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let mime_type = guess_mime_type(path);

    match std::fs::metadata(path) {
        Ok(meta) => FileInfo {
            path: path.to_path_buf(),
            name,
            size: meta.len(),
            mime_type,
            exists: true,
        },
        Err(_) => FileInfo {
            path: path.to_path_buf(),
            name,
            size: 0,
            mime_type,
            exists: false,
        },
    }
}

/// Format a byte count as a human-readable size string.
///
/// Uses 1024-based units with user-friendly labels (KB, MB, GB, TB).
#[allow(dead_code)]
pub fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    #[allow(clippy::cast_precision_loss)]
    let bytes_f = bytes as f64;

    if bytes == 0 {
        "0 B".to_string()
    } else if bytes_f < KB {
        format!("{bytes} B")
    } else if bytes_f < MB {
        format!("{:.1} KB", bytes_f / KB)
    } else if bytes_f < GB {
        format!("{:.1} MB", bytes_f / MB)
    } else if bytes_f < TB {
        format!("{:.1} GB", bytes_f / GB)
    } else {
        format!("{:.1} TB", bytes_f / TB)
    }
}

/// Guess a MIME type from the file extension.
///
/// Returns `"application/octet-stream"` for unrecognised extensions.
#[allow(dead_code)]
pub fn guess_mime_type(path: &Path) -> String {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Text
        "txt" | "text" => "text/plain",
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "js" => "text/javascript",
        "ts" => "text/typescript",
        "json" => "application/json",
        "toml" => "application/toml",
        "yaml" | "yml" => "application/yaml",
        "xml" => "application/xml",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "sh" => "text/x-shellscript",

        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",

        // Documents
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/msword",
        "odt" => "application/vnd.oasis.opendocument.text",

        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "xz" => "application/x-xz",
        "7z" => "application/x-7z-compressed",

        // Media
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",

        _ => "application/octet-stream",
    }
    .to_string()
}

/// Decode percent-encoded (`%XX`) sequences in a URI string.
#[allow(dead_code)]
pub fn url_decode(s: &str) -> String {
    let mut result = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&result).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1_048_576), "1.0 MB");
        assert_eq!(format_file_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_parse_uri_list_basic() {
        let input = "file:///home/user/a.txt\nfile:///home/user/b.rs\nfile:///tmp/c.png\n";
        let files = parse_uri_list(input);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].name, "a.txt");
        assert_eq!(files[1].name, "b.rs");
        assert_eq!(files[2].name, "c.png");
        assert_eq!(files[1].mime_type, "text/x-rust");
        assert_eq!(files[2].mime_type, "image/png");
    }

    #[test]
    fn test_parse_uri_list_with_comments() {
        let input =
            "# This is a comment\nfile:///home/user/a.txt\n# Another comment\nfile:///tmp/b.rs\n";
        let files = parse_uri_list(input);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].name, "a.txt");
        assert_eq!(files[1].name, "b.rs");
    }

    #[test]
    fn test_parse_uri_list_empty() {
        assert!(parse_uri_list("").is_empty());
        assert!(parse_uri_list("\n\n\n").is_empty());
        assert!(parse_uri_list("# only comments\n# here").is_empty());
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("%2Fhome%2Fuser"), "/home/user");
        assert_eq!(url_decode("no_encoding"), "no_encoding");
        assert_eq!(url_decode("100%25"), "100%");
        // Incomplete sequence left as-is
        assert_eq!(url_decode("trailing%2"), "trailing%2");
    }

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(guess_mime_type(Path::new("file.txt")), "text/plain");
        assert_eq!(guess_mime_type(Path::new("main.rs")), "text/x-rust");
        assert_eq!(guess_mime_type(Path::new("script.py")), "text/x-python");
        assert_eq!(guess_mime_type(Path::new("photo.jpg")), "image/jpeg");
        assert_eq!(guess_mime_type(Path::new("photo.jpeg")), "image/jpeg");
        assert_eq!(guess_mime_type(Path::new("image.png")), "image/png");
        assert_eq!(guess_mime_type(Path::new("doc.pdf")), "application/pdf");
        assert_eq!(guess_mime_type(Path::new("archive.zip")), "application/zip");
        assert_eq!(
            guess_mime_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
        assert_eq!(
            guess_mime_type(Path::new("no_extension")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_resolve_nonexistent_file() {
        let info = resolve_file_info(Path::new("/tmp/nonexistent_file_abc123xyz.txt"));
        assert!(!info.exists);
        assert_eq!(info.size, 0);
        assert_eq!(info.name, "nonexistent_file_abc123xyz.txt");
        assert_eq!(info.mime_type, "text/plain");
    }

    #[test]
    fn test_resolve_existing_file() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("clipboard_test_file_handler.txt");
        let content = b"hello clipboard";
        {
            let mut f = std::fs::File::create(&file_path).expect("create temp file");
            f.write_all(content).expect("write temp file");
        }

        let info = resolve_file_info(&file_path);
        assert!(info.exists);
        assert_eq!(info.size, content.len() as u64);
        assert_eq!(info.name, "clipboard_test_file_handler.txt");
        assert_eq!(info.mime_type, "text/plain");

        // Clean up
        let _ = std::fs::remove_file(&file_path);
    }
}
