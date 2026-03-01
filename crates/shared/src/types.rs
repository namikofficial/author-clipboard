//! Core data types for clipboard items

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The type of clipboard content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
        }
    }
}

impl std::str::FromStr for ContentType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "image" => Self::Image,
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
    /// Optional: which app it came from
    pub source_app: Option<String>,
    /// Whether this item contains sensitive content
    pub sensitive: bool,
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
            source_app: None,
            sensitive,
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
            source_app: None,
            sensitive: false,
        }
    }

    /// Compute a fast hash of content for deduplication.
    pub fn hash_content(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute a hash of raw bytes (for images).
    pub fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Whether this is an image item.
    pub fn is_image(&self) -> bool {
        self.content_type == ContentType::Image
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
