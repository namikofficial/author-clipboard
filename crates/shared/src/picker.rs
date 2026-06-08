//! Shared picker types and logic for external menus and native pickers.
//!
//! Provides reusable loading, formatting, masking, and restore logic
//! consumed by `author-clipboard-ctl picker` (external menu) and
//! `author-clipboard-hypr-picker` (first-party native picker).

use crate::clipboard;
use crate::config::Config;
use crate::db::Database;
use crate::file_handler;
use crate::types::{ClipboardItem, ContentType, Snippet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Errors that can occur in picker operations.
#[derive(Debug, thiserror::Error)]
pub enum PickerError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// Clipboard restore error.
    #[error("clipboard error: {0}")]
    Clipboard(#[from] clipboard::ClipboardSetError),
    /// Sensitive entry needs explicit user confirmation before restore.
    #[error("sensitive confirmation required")]
    SensitiveConfirmationRequired,
}

// ── Types ─────────────────────────────────────────────────────────

/// The data source for picker entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerSource {
    History,
    Snippets,
    Emoji,
    Symbols,
    Kaomoji,
    All,
}

impl std::fmt::Display for PickerSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::History => write!(f, "history"),
            Self::Snippets => write!(f, "snippets"),
            Self::Emoji => write!(f, "emoji"),
            Self::Symbols => write!(f, "symbols"),
            Self::Kaomoji => write!(f, "kaomoji"),
            Self::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for PickerSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "history" => Ok(Self::History),
            "snippets" => Ok(Self::Snippets),
            "emoji" => Ok(Self::Emoji),
            "symbols" => Ok(Self::Symbols),
            "kaomoji" => Ok(Self::Kaomoji),
            "all" => Ok(Self::All),
            _ => Err(format!("unknown picker source: {s}")),
        }
    }
}

/// What to do with the selected item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerAction {
    Copy,
    QuickPaste,
}

impl std::fmt::Display for PickerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Copy => write!(f, "copy"),
            Self::QuickPaste => write!(f, "quick-paste"),
        }
    }
}

impl std::str::FromStr for PickerAction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "copy" => Ok(Self::Copy),
            "quick-paste" | "quick_paste" | "paste" => Ok(Self::QuickPaste),
            _ => Err(format!("unknown picker action: {s}")),
        }
    }
}

/// Options controlling which entries are loaded and how they are displayed.
#[derive(Debug, Clone)]
pub struct PickerOptions {
    pub source: PickerSource,
    pub limit: usize,
    pub query: Option<String>,
    pub include_sensitive: bool,
    pub action: PickerAction,
}

impl Default for PickerOptions {
    fn default() -> Self {
        Self {
            source: PickerSource::History,
            limit: 50,
            query: None,
            include_sensitive: false,
            action: PickerAction::Copy,
        }
    }
}

/// A single entry ready for display in any picker UI.
#[derive(Debug, Clone)]
pub struct PickerEntry {
    /// Database row id (for clipboard/snippet items). `None` for expression entries.
    pub id: Option<i64>,
    pub source: PickerSource,
    pub content_type: Option<ContentType>,
    /// Main display line.
    pub title: String,
    /// Secondary metadata line.
    pub subtitle: Option<String>,
    /// The raw content that will be copied/pasted.
    pub content: String,
    /// MIME type for clipboard restore.
    pub mime_type: Option<String>,
    pub sensitive: bool,
    pub pinned: bool,
    pub timestamp: Option<DateTime<Utc>>,
}

// ── UI-Facing Models ─────────────────────────────────────────────

/// Unified error type for clipboard UI operations.
/// This provides a clear, user-friendly error hierarchy that all
/// UI components (applet, picker, CLI) can use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardUiError {
    /// Database operation failed.
    Database(String),
    /// Clipboard restore operation failed.
    Clipboard(String),
    /// Item requires explicit user confirmation before copy (sensitive item).
    SensitiveConfirmationRequired { id: i64 },
    /// Permission denied (e.g., quick-paste requires setup).
    PermissionRequired(String),
    /// Daemon is not running or IPC is unavailable.
    DaemonUnavailable,
    /// Item is encrypted and decryption failed or is not available.
    EncryptedContentUnavailable { id: i64 },
    /// Item not found.
    NotFound { id: i64 },
    /// I/O error (file read/write, etc.).
    Io(String),
    /// Invalid configuration.
    Config(String),
}

impl std::fmt::Display for ClipboardUiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Database error: {msg}"),
            Self::Clipboard(msg) => write!(f, "Clipboard error: {msg}"),
            Self::SensitiveConfirmationRequired { id } => {
                write!(f, "Sensitive item {id} requires confirmation")
            }
            Self::PermissionRequired(msg) => write!(f, "Permission required: {msg}"),
            Self::DaemonUnavailable => write!(f, "Clipboard daemon is not running"),
            Self::EncryptedContentUnavailable { id } => {
                write!(f, "Cannot access encrypted item {id}")
            }
            Self::NotFound { id } => write!(f, "Item {id} not found"),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
        }
    }
}

impl std::error::Error for ClipboardUiError {}

impl From<rusqlite::Error> for ClipboardUiError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<clipboard::ClipboardSetError> for ClipboardUiError {
    fn from(e: clipboard::ClipboardSetError) -> Self {
        Self::Clipboard(e.to_string())
    }
}

/// State of an action for UI feedback.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ActionState {
    /// No action in progress.
    #[default]
    Idle,
    /// Action is currently executing.
    Loading { message: Option<String> },
    /// Action completed successfully.
    Success { message: Option<String> },
    /// Action failed.
    Failed { error: ClipboardUiError },
    /// Action requires user confirmation (e.g., sensitive item copy).
    AwaitingConfirmation { id: i64, action: String },
    /// Daemon is not available.
    DaemonUnavailable,
}

/// Filter options for clipboard history queries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClipboardFilterState {
    /// Filter by content type.
    pub content_types: Option<Vec<ContentType>>,
    /// Filter by pinned status (Some(true) = pinned only, Some(false) = unpinned only).
    pub pinned: Option<bool>,
    /// Filter by sensitive flag.
    pub sensitive: Option<bool>,
    /// Filter by source application.
    pub source_app: Option<String>,
}

impl ClipboardFilterState {
    /// Returns true if no filters are active (show all items).
    pub fn is_empty(&self) -> bool {
        self.content_types.is_none()
            && self.pinned.is_none()
            && self.sensitive.is_none()
            && self.source_app.is_none()
    }
}

/// Search options with pagination.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClipboardSearchState {
    /// The search query string.
    pub query: String,
    /// Additional filters to apply.
    pub filters: ClipboardFilterState,
    /// Number of items to skip (for pagination).
    pub offset: usize,
    /// Maximum number of items to return.
    pub limit: usize,
}

impl ClipboardSearchState {
    /// Create a new search state with default pagination.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            filters: ClipboardFilterState::default(),
            offset: 0,
            limit: 50,
        }
    }

    /// Returns true if this is an empty search (show recent items).
    pub fn is_empty(&self) -> bool {
        self.query.is_empty() && self.filters.is_empty()
    }
}

// ── Content-type icon helpers ─────────────────────────────────────

/// Return a short icon string for the content type (for external menus).
pub fn content_type_icon(ct: &ContentType) -> &'static str {
    match ct {
        ContentType::Text => "text",
        ContentType::Image => "image",
        ContentType::Html => "html",
        ContentType::Files => "files",
    }
}

/// Human-friendly age string.
pub fn format_age(ts: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*ts);
    let secs = diff.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

// ── Entry loaders ─────────────────────────────────────────────────

/// Load recent clipboard history items as picker entries.
///
/// # Errors
/// Returns a database error if the query fails.
pub fn load_history_entries(
    db: &Database,
    config: &Config,
    options: &PickerOptions,
) -> Result<Vec<PickerEntry>, PickerError> {
    let reveal_sensitive = options.include_sensitive || config.picker.show_sensitive_previews;
    let items = db.get_recent(options.limit)?;
    let entries = items
        .into_iter()
        .filter(|item| options.include_sensitive || !item.sensitive)
        .map(|item| entry_preview(&item, reveal_sensitive, config))
        .collect();
    Ok(entries)
}

/// Load snippet entries as picker entries.
///
/// # Errors
/// Returns a database error if the query fails.
pub fn load_snippet_entries(
    db: &Database,
    options: &PickerOptions,
) -> Result<Vec<PickerEntry>, PickerError> {
    let snippets = if let Some(ref q) = options.query {
        db.search_snippets(q)?
    } else {
        db.list_snippets()?
    };
    let entries = snippets
        .into_iter()
        .take(options.limit)
        .map(snippet_preview)
        .collect();
    Ok(entries)
}

/// Convert a database [`ClipboardItem`] into a [`PickerEntry`] with masked
/// sensitive previews and human-readable metadata.
pub fn entry_preview(item: &ClipboardItem, reveal_sensitive: bool, config: &Config) -> PickerEntry {
    let (title, display_content) = match item.content_type {
        ContentType::Image => {
            let path = item.image_path(&config.data_dir);
            let label = path.as_ref().and_then(|p| p.file_name()).map_or_else(
                || "Image".to_string(),
                |n| format!("Image: {}", n.to_string_lossy()),
            );
            (label, item.content.clone())
        }
        ContentType::Html => {
            let plain = item.plain_text.as_deref().unwrap_or("HTML content");
            (truncate_plain(plain, 96), item.content.clone())
        }
        ContentType::Files => {
            let files = file_handler::parse_uri_list(&item.content);
            if files.is_empty() {
                ("Files".to_string(), item.content.clone())
            } else {
                let names: Vec<&str> = files.iter().take(3).map(|f| f.name.as_str()).collect();
                (names.join(", "), item.content.clone())
            }
        }
        ContentType::Text => (truncate_plain(&item.content, 96), item.content.clone()),
    };

    let mut entry = PickerEntry {
        id: Some(item.id),
        source: PickerSource::History,
        content_type: Some(item.content_type.clone()),
        title,
        subtitle: Some(format!(
            "{} · {}",
            content_type_icon(&item.content_type),
            format_age(&item.timestamp),
        )),
        content: display_content,
        mime_type: Some(item.mime_type.clone()),
        sensitive: item.sensitive,
        pinned: item.pinned,
        timestamp: Some(item.timestamp),
    };

    if item.sensitive && !(reveal_sensitive || config.picker.show_sensitive_previews) {
        mask_sensitive_preview(&mut entry);
    }

    entry
}

/// Convert a [`Snippet`] into a [`PickerEntry`].
pub fn snippet_preview(snippet: Snippet) -> PickerEntry {
    PickerEntry {
        id: Some(snippet.id),
        source: PickerSource::Snippets,
        content_type: Some(ContentType::Text),
        title: snippet.name.clone(),
        subtitle: Some("snippet".to_string()),
        content: snippet.content,
        mime_type: Some("text/plain".to_string()),
        sensitive: false,
        pinned: false,
        timestamp: Some(snippet.updated_at),
    }
}

// ── Expression entry helpers ──────────────────────────────────────

/// Build picker entries for emoji characters.
pub fn emoji_entries(query: &str) -> Vec<PickerEntry> {
    let q = query.to_lowercase();
    let mut entries = Vec::new();
    for cat in crate::emoji::CATEGORIES {
        if q.is_empty() || cat.name.to_lowercase().contains(&q) {
            for &emoji in cat.emojis {
                entries.push(PickerEntry {
                    id: None,
                    source: PickerSource::Emoji,
                    content_type: None,
                    title: format!("{emoji}  {name}", name = cat.name),
                    subtitle: Some(format!("emoji · {}", cat.name)),
                    content: emoji.to_string(),
                    mime_type: Some("text/plain".to_string()),
                    sensitive: false,
                    pinned: false,
                    timestamp: None,
                });
            }
        } else {
            for &emoji in cat.emojis {
                if emoji.contains(query) {
                    entries.push(PickerEntry {
                        id: None,
                        source: PickerSource::Emoji,
                        content_type: None,
                        title: format!("{emoji}  {name}", name = cat.name),
                        subtitle: Some(format!("emoji · {}", cat.name)),
                        content: emoji.to_string(),
                        mime_type: Some("text/plain".to_string()),
                        sensitive: false,
                        pinned: false,
                        timestamp: None,
                    });
                }
            }
        }
    }
    entries
}

/// Build picker entries for symbols.
pub fn symbol_entries(query: &str) -> Vec<PickerEntry> {
    let q = query.to_lowercase();
    let mut entries = Vec::new();
    for cat in crate::symbols::CATEGORIES {
        if q.is_empty() || cat.name.to_lowercase().contains(&q) {
            for &(sym, desc) in cat.symbols {
                entries.push(PickerEntry {
                    id: None,
                    source: PickerSource::Symbols,
                    content_type: None,
                    title: format!("{sym}  {desc}"),
                    subtitle: Some(format!("symbol · {}", cat.name)),
                    content: sym.to_string(),
                    mime_type: Some("text/plain".to_string()),
                    sensitive: false,
                    pinned: false,
                    timestamp: None,
                });
            }
        } else {
            for &(sym, desc) in cat.symbols {
                if desc.to_lowercase().contains(&q) || sym.contains(query) {
                    entries.push(PickerEntry {
                        id: None,
                        source: PickerSource::Symbols,
                        content_type: None,
                        title: format!("{sym}  {desc}"),
                        subtitle: Some(format!("symbol · {}", cat.name)),
                        content: sym.to_string(),
                        mime_type: Some("text/plain".to_string()),
                        sensitive: false,
                        pinned: false,
                        timestamp: None,
                    });
                }
            }
        }
    }
    entries
}

/// Build picker entries for kaomoji.
pub fn kaomoji_entries(query: &str) -> Vec<PickerEntry> {
    let q = query.to_lowercase();
    let mut entries = Vec::new();
    for cat in crate::kaomoji::CATEGORIES {
        if q.is_empty() || cat.name.to_lowercase().contains(&q) {
            for &item in cat.items {
                entries.push(PickerEntry {
                    id: None,
                    source: PickerSource::Kaomoji,
                    content_type: None,
                    title: item.to_string(),
                    subtitle: Some(format!("kaomoji · {}", cat.name)),
                    content: item.to_string(),
                    mime_type: Some("text/plain".to_string()),
                    sensitive: false,
                    pinned: false,
                    timestamp: None,
                });
            }
        } else {
            for &item in cat.items {
                if item.contains(&q) {
                    entries.push(PickerEntry {
                        id: None,
                        source: PickerSource::Kaomoji,
                        content_type: None,
                        title: item.to_string(),
                        subtitle: Some(format!("kaomoji · {}", cat.name)),
                        content: item.to_string(),
                        mime_type: Some("text/plain".to_string()),
                        sensitive: false,
                        pinned: false,
                        timestamp: None,
                    });
                }
            }
        }
    }
    entries
}

/// Load all entries matching the given source and query.
///
/// # Errors
/// Returns a database error if clipboard or snippet queries fail.
pub fn load_entries(
    db: &Database,
    config: &Config,
    options: &PickerOptions,
) -> Result<Vec<PickerEntry>, PickerError> {
    match options.source {
        PickerSource::History => load_history_entries(db, config, options),
        PickerSource::Snippets => load_snippet_entries(db, options),
        PickerSource::Emoji => Ok(emoji_entries(options.query.as_deref().unwrap_or(""))),
        PickerSource::Symbols => Ok(symbol_entries(options.query.as_deref().unwrap_or(""))),
        PickerSource::Kaomoji => Ok(kaomoji_entries(options.query.as_deref().unwrap_or(""))),
        PickerSource::All => {
            let mut all = Vec::new();
            all.extend(load_history_entries(db, config, options)?);
            let remaining = options.limit.saturating_sub(all.len());
            if remaining > 0 {
                let snippet_opts = PickerOptions {
                    limit: remaining,
                    ..options.clone()
                };
                all.extend(load_snippet_entries(db, &snippet_opts)?);
            }
            Ok(all)
        }
    }
}

// ── Restore ───────────────────────────────────────────────────────

/// Restore a picker entry to the Wayland clipboard, or quick-paste it.
///
/// # Errors
/// Returns a clipboard-set error if `wl-copy` fails.
pub fn restore_entry(
    entry: &PickerEntry,
    config: &Config,
    action: PickerAction,
    confirmed_sensitive: bool,
) -> Result<clipboard::ClipboardSetResult, PickerError> {
    if entry.sensitive && config.picker.confirm_sensitive_copy && !confirmed_sensitive {
        return Err(PickerError::SensitiveConfirmationRequired);
    }

    // Try IPC first for proper encryption/decryption handling.
    // This ensures encrypted items are properly decrypted before clipboard restore.
    if let Some(id) = entry.id {
        let copy_mode = match action {
            PickerAction::Copy => crate::ipc::CopyMode::Copy,
            PickerAction::QuickPaste => crate::ipc::CopyMode::QuickPaste,
        };
        let client = crate::ipc::IpcClient::new();
        if let Ok(response) = client.send_command(&crate::ipc::IpcCommand::Copy { id, mode: copy_mode }) {
            if response.ok {
                // IPC succeeded - parse the result
                if let Some(data) = response.data {
                    let mime_type = data.get("mime_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text/plain")
                        .to_string();
                    return Ok(clipboard::ClipboardSetResult {
                        mime_type,
                        behavior: if matches!(action, PickerAction::QuickPaste) { "quick-paste" } else { "copy" },
                    });
                }
            }
            // IPC failed with error - fall through to fallback
        }
        // IPC unavailable or failed - fall through to direct access
    }

    // Fallback: direct clipboard access (may not work for encrypted items)
    match action {
        PickerAction::Copy => {
            if let Some(id) = entry.id {
                if let Some(item) = Database::open(&config.db_path())
                    .ok()
                    .and_then(|db| db.get_by_id(id).ok())
                    .flatten()
                {
                    return clipboard::set_clipboard_item(&item, &config.data_dir)
                        .map_err(PickerError::from);
                }
            }
            clipboard::set_clipboard_text(&entry.content).map_err(PickerError::from)
        }
        PickerAction::QuickPaste => {
            if entry
                .content_type
                .as_ref()
                .is_some_and(|ct| matches!(ct, ContentType::Text | ContentType::Html))
                || entry.content_type.is_none()
            {
                use crate::quick_paste;
                if let Some(backend) = quick_paste::detect_backend() {
                    let text = if entry
                        .content_type
                        .as_ref()
                        .is_some_and(|ct| matches!(ct, ContentType::Html))
                    {
                        entry
                            .content
                            .split("<body")
                            .last()
                            .and_then(|s| s.split("</body>").next())
                            .unwrap_or(&entry.content)
                            .to_string()
                    } else {
                        entry.content.clone()
                    };
                    let _result = quick_paste::quick_paste(&text, &backend)
                        .map_err(clipboard::ClipboardSetError::Io)
                        .map_err(PickerError::from)?;
                    Ok(clipboard::ClipboardSetResult {
                        mime_type: "text/plain".to_string(),
                        behavior: "quick-paste",
                    })
                } else {
                    clipboard::set_clipboard_text(&entry.content).map_err(PickerError::from)
                }
            } else {
                if let Some(id) = entry.id {
                    if let Some(item) = Database::open(&config.db_path())
                        .ok()
                        .and_then(|db| db.get_by_id(id).ok())
                        .flatten()
                    {
                        return clipboard::set_clipboard_item(&item, &config.data_dir)
                            .map_err(PickerError::from);
                    }
                }
                clipboard::set_clipboard_text(&entry.content).map_err(PickerError::from)
            }
        }
    }
}

// ── Sensitive masking ─────────────────────────────────────────────

/// Replace the title and content of a sensitive entry with a masked placeholder.
pub fn mask_sensitive_preview(entry: &mut PickerEntry) {
    entry.title = "Sensitive item".to_string();
    entry.content = "[hidden]".to_string();
}

// ── Formatting helpers ────────────────────────────────────────────

/// Format a picker entry for display in an external dmenu-style picker.
///
/// The format is: `{title}  ·  {subtitle}`
/// For clipboard items that need id mapping, prepend `{id}\t`.
pub fn format_external_label(entry: &PickerEntry, include_id: bool) -> String {
    let subtitle = entry.subtitle.as_deref().unwrap_or("");
    if include_id {
        if let Some(id) = entry.id {
            format!("{id}\t{}  ·  {}", entry.title, subtitle)
        } else {
            format!("{}\t{}  ·  {}", entry.content, entry.title, subtitle)
        }
    } else {
        format!("{}  ·  {}", entry.title, subtitle)
    }
}

/// Parse an external menu selection back to an `Option<i64>` item id.
pub fn parse_external_selection(selected: &str) -> Option<i64> {
    selected
        .split_once('\t')
        .and_then(|(id_str, _)| id_str.parse::<i64>().ok())
}

/// A rendered row for external menu pickers.
#[derive(Debug, Clone)]
pub struct ExternalPickerRow {
    pub key: usize,
    pub label: String,
}

/// Build stable-key external rows so UI labels can stay human-readable.
pub fn build_external_rows(
    entries: &[PickerEntry],
    include_key_prefix: bool,
) -> Vec<ExternalPickerRow> {
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let mut label = format_external_label(entry, false);
            if include_key_prefix {
                label = format!("{index}\t{label}");
            }
            ExternalPickerRow { key: index, label }
        })
        .collect()
}

/// Parse a selected menu row and resolve back to the original entry index.
pub fn parse_external_row_selection(
    selected: &str,
    rows: &[ExternalPickerRow],
    include_key_prefix: bool,
) -> Option<usize> {
    if include_key_prefix {
        return selected
            .split_once('\t')
            .and_then(|(prefix, _)| prefix.parse::<usize>().ok())
            .filter(|idx| *idx < rows.len());
    }

    rows.iter()
        .find(|row| row.label == selected)
        .map(|row| row.key)
}

/// Filter entries by a search query (case-insensitive substring match on title + content).
pub fn filter_entries(entries: &[PickerEntry], query: &str) -> Vec<PickerEntry> {
    if query.is_empty() {
        return entries.to_vec();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .filter(|e| e.title.to_lowercase().contains(&q) || e.content.to_lowercase().contains(&q))
        .cloned()
        .collect()
}

// ── Private helpers ───────────────────────────────────────────────

fn truncate_plain(text: &str, max_len: usize) -> String {
    let single_line = text.replace(['\n', '\r', '\t'], " ");
    if single_line.chars().count() > max_len {
        format!("{}…", single_line.chars().take(max_len).collect::<String>())
    } else {
        single_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ClipboardItem;

    fn make_item(content: &str, sensitive: bool) -> ClipboardItem {
        let mut item = ClipboardItem::new_text(content.to_string());
        item.sensitive = sensitive;
        item
    }

    #[test]
    fn test_entry_preview_masks_sensitive() {
        let item = make_item("hunter2-password", true);
        let config = Config::default();
        let entry = entry_preview(&item, false, &config);
        assert!(entry.sensitive);
        assert_eq!(entry.title, "Sensitive item");
        assert_eq!(entry.content, "[hidden]");
    }

    #[test]
    fn test_entry_preview_shows_normal_content() {
        let item = make_item("hello world", false);
        let config = Config::default();
        let entry = entry_preview(&item, false, &config);
        assert!(!entry.sensitive);
        assert_eq!(entry.title, "hello world");
        assert_eq!(entry.content, "hello world");
    }

    #[test]
    fn test_format_external_label_with_id() {
        let entry = PickerEntry {
            id: Some(42),
            source: PickerSource::History,
            content_type: Some(ContentType::Text),
            title: "hello".to_string(),
            subtitle: Some("text · 2m ago".to_string()),
            content: "hello".to_string(),
            mime_type: Some("text/plain".to_string()),
            sensitive: false,
            pinned: false,
            timestamp: None,
        };
        let label = format_external_label(&entry, true);
        assert!(label.starts_with("42\t"));
        assert!(label.contains("hello"));
        assert!(label.contains("text"));
    }

    #[test]
    fn test_entry_preview_can_show_sensitive_with_config() {
        let item = make_item("hunter2-password", true);
        let mut config = Config::default();
        config.picker.show_sensitive_previews = true;
        let entry = entry_preview(&item, true, &config);
        assert_eq!(entry.title, "hunter2-password");
        assert_eq!(entry.content, "hunter2-password");
    }

    #[test]
    fn test_parse_external_selection_valid() {
        assert_eq!(parse_external_selection("42\ttext\thello"), Some(42));
        assert_eq!(parse_external_selection("0\timage\tfoo"), Some(0));
    }

    #[test]
    fn test_parse_external_selection_invalid() {
        assert_eq!(parse_external_selection("not-a-number\thello"), None);
        assert_eq!(parse_external_selection(""), None);
    }

    #[test]
    fn test_external_row_mapping_handles_tabs_and_newlines() {
        let entries = vec![PickerEntry {
            id: None,
            source: PickerSource::Emoji,
            content_type: None,
            title: "line1\tline2".to_string(),
            subtitle: Some("meta\nrow".to_string()),
            content: "line1\tline2\nmeta".to_string(),
            mime_type: Some("text/plain".to_string()),
            sensitive: false,
            pinned: false,
            timestamp: None,
        }];
        let rows = build_external_rows(&entries, true);
        let selected = rows[0].label.clone();
        assert_eq!(
            parse_external_row_selection(&selected, &rows, true),
            Some(0)
        );
    }

    #[test]
    fn test_filter_entries() {
        let entries = vec![
            PickerEntry {
                id: Some(1),
                source: PickerSource::History,
                content_type: Some(ContentType::Text),
                title: "foo bar".to_string(),
                subtitle: None,
                content: "foo bar".to_string(),
                mime_type: None,
                sensitive: false,
                pinned: false,
                timestamp: None,
            },
            PickerEntry {
                id: Some(2),
                source: PickerSource::History,
                content_type: Some(ContentType::Text),
                title: "baz qux".to_string(),
                subtitle: None,
                content: "baz qux".to_string(),
                mime_type: None,
                sensitive: false,
                pinned: false,
                timestamp: None,
            },
        ];
        let filtered = filter_entries(&entries, "foo");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "foo bar");
    }

    #[test]
    fn test_format_age_just_now() {
        let now = Utc::now();
        assert_eq!(format_age(&now), "just now");
    }

    #[test]
    fn test_content_type_icon() {
        assert_eq!(content_type_icon(&ContentType::Text), "text");
        assert_eq!(content_type_icon(&ContentType::Image), "image");
        assert_eq!(content_type_icon(&ContentType::Html), "html");
        assert_eq!(content_type_icon(&ContentType::Files), "files");
    }

    #[test]
    fn test_picker_source_roundtrip() {
        let sources = [
            PickerSource::History,
            PickerSource::Snippets,
            PickerSource::Emoji,
            PickerSource::Symbols,
            PickerSource::Kaomoji,
            PickerSource::All,
        ];
        for src in sources {
            let s = src.to_string();
            let parsed: PickerSource = s.parse().unwrap();
            assert_eq!(parsed, src);
        }
    }

    #[test]
    fn test_picker_action_roundtrip() {
        let actions = [PickerAction::Copy, PickerAction::QuickPaste];
        for act in actions {
            let s = act.to_string();
            let parsed: PickerAction = s.parse().unwrap();
            assert_eq!(parsed, act);
        }
    }

    #[test]
    fn test_snippet_preview() {
        let snippet = Snippet {
            id: 1,
            name: "greeting".to_string(),
            content: "hello world".to_string(),
            updated_at: Utc::now(),
        };
        let entry = snippet_preview(snippet);
        assert_eq!(entry.id, Some(1));
        assert_eq!(entry.title, "greeting");
        assert_eq!(entry.content, "hello world");
        assert!(!entry.sensitive);
    }
}
