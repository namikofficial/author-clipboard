//! Core data types for clipboard items

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single clipboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier
    pub id: i64,
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
        Self {
            id: 0, // Will be set by database
            content,
            mime_type: "text/plain".to_string(),
            timestamp: Utc::now(),
            pinned: false,
            source_app: None,
        }
    }
}
