//! Versioned, privacy-preserving history import/export.

use crate::{types::ClipboardItem, Database};
use serde::{Deserialize, Serialize};

/// Requested export scope/privacy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportMode {
    Redacted,
    FullWithConfirmation,
    SnippetsOnly,
    SettingsOnly,
}

/// Safe import/export failure.
#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("full export requires explicit confirmation")]
    FullExportConfirmationRequired,
    #[error("invalid import document")]
    InvalidDocument,
    #[error("unsupported import version: {0}")]
    UnsupportedVersion(u32),
}

/// Versioned portable document. Extra sections are reserved for surface adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDocument {
    pub version: u32,
    pub mode: ExportMode,
    #[serde(default)]
    pub history: Vec<ClipboardItem>,
    #[serde(default)]
    pub snippets: Vec<serde_json::Value>,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

/// Non-mutating import summary for confirmation UIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPreview {
    pub history_count: usize,
    pub sensitive_count: usize,
    pub warnings: Vec<String>,
}

/// Export history. Redacted mode is safe by default; full mode is explicitly gated.
pub fn export_history(
    items: &[ClipboardItem],
    mode: ExportMode,
    confirmed: bool,
) -> Result<String, TransferError> {
    if mode == ExportMode::FullWithConfirmation && !confirmed {
        return Err(TransferError::FullExportConfirmationRequired);
    }
    let history = if mode == ExportMode::Redacted {
        items
            .iter()
            .cloned()
            .map(|mut item| {
                if item.sensitive || item.encrypted {
                    item.content = item.redacted_preview();
                    item.plain_text = None;
                    item.encrypted = false;
                    item.encryption_version = None;
                }
                item
            })
            .collect()
    } else if mode == ExportMode::FullWithConfirmation {
        items.to_vec()
    } else {
        Vec::new()
    };
    serde_json::to_string_pretty(&ExportDocument {
        version: 1,
        mode,
        history,
        snippets: Vec::new(),
        settings: None,
    })
    .map_err(|_| TransferError::InvalidDocument)
}

/// Validate a document and report what import would do without writing data.
pub fn preview_import(json: &str) -> Result<ImportPreview, TransferError> {
    let document: ExportDocument =
        serde_json::from_str(json).map_err(|_| TransferError::InvalidDocument)?;
    if document.version != 1 {
        return Err(TransferError::UnsupportedVersion(document.version));
    }
    let sensitive_count = document
        .history
        .iter()
        .filter(|item| Database::derive_sensitive_for_import(item))
        .count();
    let mut warnings = Vec::new();
    if sensitive_count > 0 {
        warnings.push("Sensitive content was detected and will retain safe handling.".into());
    }
    Ok(ImportPreview {
        history_count: document.history.len(),
        sensitive_count,
        warnings,
    })
}

/// Parse import items and re-derive all sensitive flags. This does not overwrite storage.
pub fn validated_history(json: &str) -> Result<Vec<ClipboardItem>, TransferError> {
    preview_import(json)?;
    let mut document: ExportDocument =
        serde_json::from_str(json).map_err(|_| TransferError::InvalidDocument)?;
    for item in &mut document.history {
        item.sensitive = Database::derive_sensitive_for_import(item);
    }
    Ok(document.history)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_redacts_and_full_requires_confirmation() {
        let mut item = ClipboardItem::new_text("ghp_1234567890abcdefghij".into());
        item.sensitive = true;
        let json = export_history(&[item.clone()], ExportMode::Redacted, false).unwrap();
        assert!(!json.contains("ghp_"));
        assert!(matches!(
            export_history(&[item], ExportMode::FullWithConfirmation, false),
            Err(TransferError::FullExportConfirmationRequired)
        ));
    }
    #[test]
    fn preview_and_import_redetect_sensitive() {
        let mut item = ClipboardItem::new_text("ghp_1234567890abcdefghij".into());
        item.sensitive = false;
        let json = export_history(&[item], ExportMode::FullWithConfirmation, true).unwrap();
        let preview = preview_import(&json).unwrap();
        assert_eq!(preview.sensitive_count, 1);
        assert!(validated_history(&json).unwrap()[0].sensitive);
    }
}
