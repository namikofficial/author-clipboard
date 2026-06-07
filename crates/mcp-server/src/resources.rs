//! MCP resource definitions.

use serde::{Deserialize, Serialize};

/// Resource types exposed by the MCP server.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// Recent clipboard items.
    Recent,
    /// A specific clipboard item by ID.
    Item { id: i64 },
    /// Pinned items.
    Pins,
    /// User snippets.
    Snippets,
    /// Database statistics.
    Stats,
    /// Recent audit log entries.
    AuditRecent,
}

/// Parse a clipboard resource URI.
#[allow(dead_code)]
pub fn parse_resource_uri(uri: &str) -> Option<ResourceType> {
    if uri.starts_with("clipboard://recent") {
        Some(ResourceType::Recent)
    } else if uri.starts_with("clipboard://item/") {
        let id = uri.strip_prefix("clipboard://item/")?.parse().ok()?;
        Some(ResourceType::Item { id })
    } else if uri.starts_with("clipboard://pins") {
        Some(ResourceType::Pins)
    } else if uri.starts_with("clipboard://snippets") {
        Some(ResourceType::Snippets)
    } else if uri.starts_with("clipboard://stats") {
        Some(ResourceType::Stats)
    } else if uri.starts_with("clipboard://audit/recent") {
        Some(ResourceType::AuditRecent)
    } else {
        None
    }
}
