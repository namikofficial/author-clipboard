//! Privacy-safe item view models shared by user-facing adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::presentation::{present, ContentPresentation};
use crate::types::ClipboardItem;

/// User actions permitted for a clipboard result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemAction {
    Copy,
    QuickPaste,
    Pin,
    Star,
    Delete,
    AddToCollection,
    Reveal,
}

/// A safe-by-construction clipboard representation for UI, CLI, and MCP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ItemViewModel {
    pub id: i64,
    pub presentation: ContentPresentation,
    pub safe_preview: String,
    pub source_app: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub pinned: bool,
    pub starred: bool,
    pub sensitive: bool,
    pub encrypted: bool,
    pub actions: Vec<ItemAction>,
}

impl From<&ClipboardItem> for ItemViewModel {
    fn from(item: &ClipboardItem) -> Self {
        let presentation = present(item);
        let safe_preview = match &presentation {
            ContentPresentation::Secret { redacted_preview } => redacted_preview.clone(),
            ContentPresentation::Text { preview }
            | ContentPresentation::Url { preview, .. }
            | ContentPresentation::Code { preview, .. }
            | ContentPresentation::Json { preview, .. }
            | ContentPresentation::Unknown { preview } => preview.clone(),
            ContentPresentation::Color { hex, .. } => hex.clone(),
            ContentPresentation::Html { text_preview } => text_preview.clone(),
            ContentPresentation::Image { path, .. } => path.clone(),
            ContentPresentation::File { name, .. } => name.clone(),
        };
        let mut actions = vec![
            ItemAction::Copy,
            ItemAction::Pin,
            ItemAction::Star,
            ItemAction::Delete,
            ItemAction::AddToCollection,
        ];
        if !item.sensitive && !item.encrypted {
            actions.insert(1, ItemAction::QuickPaste);
        }
        if item.sensitive || item.encrypted {
            actions.push(ItemAction::Reveal);
        }
        Self {
            id: item.id,
            presentation,
            safe_preview,
            source_app: item.source_app.clone(),
            timestamp: item.timestamp,
            pinned: item.pinned,
            starred: item.starred,
            sensitive: item.sensitive,
            encrypted: item.encrypted,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secret_model_never_contains_raw_content() {
        let mut item = ClipboardItem::new_text("password=hunter2".to_string());
        item.sensitive = true;
        let model = ItemViewModel::from(&item);
        assert!(!model.safe_preview.contains("hunter2"));
        assert!(!model.actions.contains(&ItemAction::QuickPaste));
        assert!(model.actions.contains(&ItemAction::Reveal));
    }
}
