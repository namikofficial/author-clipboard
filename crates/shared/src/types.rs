//! Core data types for clipboard items

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The type of clipboard content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Plain text content.
    Text,
    /// Image content stored as a file path.
    Image,
    /// HTML markup content.
    Html,
    /// File list (URIs or paths).
    Files,
}

impl ContentType {
    /// Returns the string representation of the content type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Html => "html",
            Self::Files => "files",
        }
    }
}

impl std::str::FromStr for ContentType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "image" => Self::Image,
            "html" => Self::Html,
            "files" => Self::Files,
            _ => Self::Text,
        })
    }
}

/// Represents a single clipboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier
    pub id: i64,
    /// Hash of the content for deduplication
    pub content_hash: u64,
    /// The actual content (text) or file path (images)
    pub content: String,
    /// MIME type (e.g., "text/plain", "image/png")
    pub mime_type: String,
    /// Content type discriminator
    pub content_type: ContentType,
    /// When this was copied
    pub timestamp: DateTime<Utc>,
    /// Whether this item is pinned
    pub pinned: bool,
    /// Whether this item is starred (priority ranking)
    pub starred: bool,
    /// Optional: which app it came from
    pub source_app: Option<String>,
    /// Whether this item contains sensitive content
    pub sensitive: bool,
    /// Plain text representation for search indexing (used for HTML items)
    pub plain_text: Option<String>,
}

impl ClipboardItem {
    pub fn new_text(content: String) -> Self {
        let content_hash = Self::hash_content(&content);
        let sensitive = crate::sensitive::check_sensitivity(&content).is_sensitive;
        Self {
            id: 0,
            content_hash,
            content,
            mime_type: "text/plain".to_string(),
            content_type: ContentType::Text,
            timestamp: Utc::now(),
            pinned: false,
            starred: false,
            source_app: None,
            sensitive,
            plain_text: None,
        }
    }

    /// Create a new image clipboard item.
    /// `content` should be the relative path to the stored image file.
    pub fn new_image(image_path: String, mime_type: String, data_hash: u64) -> Self {
        Self {
            id: 0,
            content_hash: data_hash,
            content: image_path,
            mime_type,
            content_type: ContentType::Image,
            timestamp: Utc::now(),
            pinned: false,
            starred: false,
            source_app: None,
            sensitive: false,
            plain_text: None,
        }
    }

    /// Create a new HTML clipboard item with both HTML content and plain text for search.
    pub fn new_html(html_content: String, plain_text: String) -> Self {
        let content_hash = Self::hash_content(&html_content);
        Self {
            id: 0,
            content_hash,
            content: html_content,
            mime_type: "text/html".to_string(),
            content_type: ContentType::Html,
            timestamp: Utc::now(),
            pinned: false,
            starred: false,
            source_app: None,
            sensitive: false,
            plain_text: Some(plain_text),
        }
    }

    /// Create a new file list clipboard item.
    /// `file_list` is the raw text/uri-list content.
    pub fn new_files(file_list: String) -> Self {
        let content_hash = Self::hash_content(&file_list);
        Self {
            id: 0,
            content_hash,
            content: file_list,
            mime_type: "text/uri-list".to_string(),
            content_type: ContentType::Files,
            timestamp: Utc::now(),
            pinned: false,
            starred: false,
            source_app: None,
            sensitive: false,
            plain_text: None,
        }
    }

    /// Compute a fast hash of content for deduplication.
    pub fn hash_content(content: &str) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        u64::from_le_bytes(result[0..8].try_into().unwrap())
    }

    /// Compute a hash of raw bytes (for images).
    pub fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        u64::from_le_bytes(result[0..8].try_into().unwrap())
    }

    /// Whether this is an image item.
    pub fn is_image(&self) -> bool {
        self.content_type == ContentType::Image
    }

    /// Whether this is an HTML item.
    pub fn is_html(&self) -> bool {
        self.content_type == ContentType::Html
    }

    /// Whether this is a file list item.
    pub fn is_files(&self) -> bool {
        self.content_type == ContentType::Files
    }

    /// Parse file URIs from a text/uri-list content.
    pub fn file_names(&self) -> Vec<String> {
        if !self.is_files() {
            return Vec::new();
        }
        self.content
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|uri| {
                let path = uri.strip_prefix("file://").unwrap_or(uri);
                std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .collect()
    }

    /// Get the full image path given the data directory.
    pub fn image_path(&self, data_dir: &std::path::Path) -> Option<PathBuf> {
        if self.is_image() {
            Some(data_dir.join("images").join(&self.content))
        } else {
            None
        }
    }

    /// Get the thumbnail path given the data directory.
    pub fn thumbnail_path(&self, data_dir: &std::path::Path) -> Option<PathBuf> {
        if self.is_image() {
            Some(data_dir.join("thumbnails").join(&self.content))
        } else {
            None
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DbStats {
    pub total_items: usize,
    pub pinned_items: usize,
    pub total_size_bytes: u64,
}

/// Security audit event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventKind {
    /// A sensitive item was detected and stored
    SensitiveItemDetected,
    /// Incognito mode was toggled
    IncognitoToggled,
    /// Clipboard history was cleared
    HistoryCleared,
    /// Data was exported
    DataExported,
    /// Data was imported
    DataImported,
    /// An item was deleted
    ItemDeleted,
    /// Sensitive items were auto-cleared (e.g., on screen lock)
    SensitiveItemsCleared,
}

impl AuditEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SensitiveItemDetected => "sensitive_item_detected",
            Self::IncognitoToggled => "incognito_toggled",
            Self::HistoryCleared => "history_cleared",
            Self::DataExported => "data_exported",
            Self::DataImported => "data_imported",
            Self::ItemDeleted => "item_deleted",
            Self::SensitiveItemsCleared => "sensitive_items_cleared",
        }
    }
}

impl std::fmt::Display for AuditEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A recorded audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: i64,
    pub event_kind: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// A user-defined text snippet for quick reuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: i64,
    pub name: String,
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

/// A named collection (board) for grouping clipboard items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
