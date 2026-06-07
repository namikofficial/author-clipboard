//! MCP tool definitions and implementations.

use serde::{Deserialize, Serialize};

/// Input for the clipboard search tool.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    /// Search query string.
    pub query: String,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Filter by content type (e.g., "text", "image").
    pub content_type: Option<Vec<String>>,
    /// Filter by pinned state.
    pub pinned: Option<bool>,
    /// Filter by sensitive state.
    pub sensitive: Option<bool>,
}

/// Input for the clipboard get tool.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetItemInput {
    /// Item ID to retrieve.
    pub id: i64,
}

/// Input for the clipboard copy tool.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyInput {
    /// Item ID to copy.
    pub id: i64,
    /// Copy mode: "copy", "quick_paste", "copy_plain_text", "copy_redacted".
    pub mode: Option<String>,
}

/// Input for the clipboard pin/unpin tool.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinInput {
    /// Item ID to pin/unpin.
    pub id: i64,
}

/// Input for the clipboard delete tool.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteInput {
    /// Item ID to delete.
    pub id: i64,
}

/// Input for snippet operations.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetInput {
    /// Snippet name (for upsert).
    pub name: Option<String>,
    /// Snippet content (for upsert).
    pub content: Option<String>,
    /// Snippet ID (for delete).
    pub id: Option<i64>,
}
