//! Core data types for clipboard items

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single clipboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier
    pub id: i64,
    /// Hash of the content for deduplication
    pub content_hash: u64,
    /// The actual content (text for now)
    pub content: String,
    /// MIME type (e.g., "text/plain")
    pub mime_type: String,
    /// When this was copied
    pub timestamp: DateTime<Utc>,
    /// Whether this item is pinned
    pub pinned: bool,
    /// Optional: which app it came from
    pub source_app: Option<String>,
}

impl ClipboardItem {
    pub fn new_text(content: String) -> Self {
        let content_hash = Self::hash_content(&content);
        Self {
            id: 0,
            content_hash,
            content,
            mime_type: "text/plain".to_string(),
            timestamp: Utc::now(),
            pinned: false,
            source_app: None,
        }
    }

    /// Compute a fast hash of content for deduplication.
    pub fn hash_content(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DbStats {
    pub total_items: usize,
    pub pinned_items: usize,
    pub total_size_bytes: u64,
}
