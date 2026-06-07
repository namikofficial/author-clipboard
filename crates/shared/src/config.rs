//! Configuration management for author-clipboard.
//!
//! Provides the [`Config`] struct for managing application settings,
//! including persistent load/save to a JSON config file.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Serde default helpers ─────────────────────────────────────────

fn default_max_items() -> usize {
    100
}
fn default_max_item_size() -> usize {
    1024 * 1024 // 1 MB
}
fn default_data_dir() -> PathBuf {
    ProjectDirs::from("com", "namikofficial", "author-clipboard")
        .map_or_else(|| PathBuf::from("."), |dirs| dirs.data_dir().to_path_buf())
}
fn default_ttl_seconds() -> u64 {
    7 * 24 * 3600 // 7 days
}
fn default_cleanup_interval_seconds() -> u64 {
    300 // 5 minutes
}
fn default_keyboard_shortcut() -> String {
    "Super+V".to_string()
}
fn default_encrypt_sensitive() -> bool {
    false
}
fn default_clear_on_lock() -> bool {
    true
}
fn default_dedup_window_seconds() -> u64 {
    2 // Skip duplicate content within 2 seconds
}
fn default_mime_denylist() -> Vec<String> {
    vec!["application/x-kde-cutselection".to_string()]
}
fn default_content_denylist() -> Vec<String> {
    vec![]
}
fn default_content_pattern_mode() -> ContentPatternMode {
    ContentPatternMode::Substring
}

/// Pattern matching mode for content denylist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentPatternMode {
    Substring, // default
    Prefix,
    Suffix,
    Exact,
}

/// Default picker UI mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PickerMode {
    #[default]
    External,
    Native,
}

impl std::fmt::Display for PickerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::External => write!(f, "external"),
            Self::Native => write!(f, "native"),
        }
    }
}

/// Configuration for picker UIs (external menu and native picker).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PickerConfig {
    /// Default picker mode (`external` or `native`).
    pub default_mode: PickerMode,
    /// Default picker source (history, snippets, emoji, symbols, kaomoji, all).
    pub default_source: String,
    /// Maximum number of results to display.
    pub max_results: usize,
    /// Whether to show sensitive item previews (masked by default).
    pub show_sensitive_previews: bool,
    /// Whether to require confirmation before copying sensitive items.
    pub confirm_sensitive_copy: bool,
    /// Whether to close the picker after a successful copy.
    pub close_after_copy: bool,
    /// Whether to prefer quick-paste over copy when possible.
    pub prefer_quick_paste: bool,
    /// Default picker window width (native picker only).
    pub width: u32,
    /// Default picker window height (native picker only).
    pub height: u32,
}

impl Default for PickerConfig {
    fn default() -> Self {
        Self {
            default_mode: PickerMode::External,
            default_source: "history".to_string(),
            max_results: 50,
            show_sensitive_previews: false,
            confirm_sensitive_copy: true,
            close_after_copy: true,
            prefer_quick_paste: false,
            width: 720,
            height: 520,
        }
    }
}

/// Application configuration for author-clipboard.
///
/// Settings are persisted to `~/.config/author-clipboard/config.json`.
/// Missing fields in the JSON file are filled with defaults via serde.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Maximum number of clipboard items to retain in history.
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    /// Maximum size (in bytes) of a single clipboard item.
    #[serde(default = "default_max_item_size")]
    pub max_item_size: usize,
    /// Directory where application data (database, images) is stored.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    /// Time-to-live for unpinned items (in seconds). 0 = never expire.
    #[serde(default = "default_ttl_seconds")]
    pub ttl_seconds: u64,
    /// How often the cleanup task runs (in seconds).
    #[serde(default = "default_cleanup_interval_seconds")]
    pub cleanup_interval_seconds: u64,
    /// Keyboard shortcut to open the clipboard picker (e.g., `"Super+V"`).
    #[serde(default = "default_keyboard_shortcut")]
    pub keyboard_shortcut: String,
    /// Whether to encrypt sensitive clipboard items at rest.
    #[serde(default = "default_encrypt_sensitive")]
    pub encrypt_sensitive: bool,
    /// Whether to clear sensitive clipboard items when the screen locks.
    #[serde(default = "default_clear_on_lock")]
    pub clear_on_lock: bool,
    /// Dedup window: skip items with identical hash within this many seconds.
    #[serde(default = "default_dedup_window_seconds")]
    pub dedup_window_seconds: u64,
    /// MIME types that should never be stored in clipboard history.
    #[serde(default = "default_mime_denylist")]
    pub mime_denylist: Vec<String>,
    /// Content patterns that should never be stored (matched according to `content_pattern_mode`).
    #[serde(default = "default_content_denylist", alias = "content_regex_denylist")]
    pub content_denylist: Vec<String>,
    /// How to match content patterns: `substring`, `prefix`, `suffix`, or `exact`.
    #[serde(default = "default_content_pattern_mode")]
    pub content_pattern_mode: ContentPatternMode,
    /// Configuration for picker UIs.
    #[serde(default)]
    pub picker: PickerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_items: default_max_items(),
            max_item_size: default_max_item_size(),
            data_dir: default_data_dir(),
            ttl_seconds: default_ttl_seconds(),
            cleanup_interval_seconds: default_cleanup_interval_seconds(),
            keyboard_shortcut: default_keyboard_shortcut(),
            encrypt_sensitive: default_encrypt_sensitive(),
            clear_on_lock: default_clear_on_lock(),
            dedup_window_seconds: default_dedup_window_seconds(),
            mime_denylist: default_mime_denylist(),
            content_denylist: default_content_denylist(),
            content_pattern_mode: default_content_pattern_mode(),
            picker: PickerConfig::default(),
        }
    }
}

impl Config {
    /// Returns the path to the configuration file.
    ///
    /// Defaults to `~/.config/author-clipboard/config.json` via
    /// [`directories::ProjectDirs`].
    #[must_use]
    pub fn config_path() -> PathBuf {
        ProjectDirs::from("com", "namikofficial", "author-clipboard").map_or_else(
            || PathBuf::from("config.json"),
            |dirs| dirs.config_dir().join("config.json"),
        )
    }

    /// Load configuration from the default config file.
    ///
    /// Falls back to [`Config::default()`] if the file is missing or
    /// contains invalid JSON.
    #[must_use]
    pub fn load() -> Self {
        let path = Self::config_path();
        std::fs::read_to_string(&path).map_or_else(
            |_| Self::default(),
            |contents| serde_json::from_str(&contents).unwrap_or_default(),
        )
    }

    /// Serialize this configuration to JSON and write it to the config file.
    ///
    /// Creates parent directories if they do not exist.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, json)
    }

    /// Write a default configuration file only if one does not already exist.
    ///
    /// This is useful for first-run initialization.
    pub fn save_default_if_missing() -> std::io::Result<()> {
        let path = Self::config_path();
        if path.exists() {
            return Ok(());
        }
        Self::default().save()
    }

    /// Full path to the `SQLite` database file.
    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("clipboard.db")
    }

    /// Path to the incognito mode flag file.
    #[must_use]
    pub fn incognito_flag_path(&self) -> PathBuf {
        self.data_dir.join(".incognito")
    }

    /// Check if incognito mode is active.
    #[must_use]
    pub fn is_incognito(&self) -> bool {
        self.incognito_flag_path().exists()
    }

    /// Toggle incognito mode on/off. Returns the new state.
    pub fn set_incognito(&self, enabled: bool) -> std::io::Result<bool> {
        let path = self.incognito_flag_path();
        if enabled {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, "1")?;
        } else if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(enabled)
    }

    /// Check if a MIME type is in the denylist.
    #[must_use]
    pub fn is_mime_denied(&self, mime_type: &str) -> bool {
        self.mime_denylist
            .iter()
            .any(|denied| denied == mime_type || mime_type.starts_with(denied.as_str()))
    }

    /// Check if content matches any pattern in the denylist.
    #[must_use]
    pub fn is_content_denied(&self, content: &str) -> bool {
        self.content_denylist
            .iter()
            .any(|pattern| match self.content_pattern_mode {
                ContentPatternMode::Substring => content.contains(pattern),
                ContentPatternMode::Prefix => content.starts_with(pattern),
                ContentPatternMode::Suffix => content.ends_with(pattern),
                ContentPatternMode::Exact => content == pattern,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let cfg = Config::default();
        assert_eq!(cfg.max_items, 100);
        assert_eq!(cfg.max_item_size, 1024 * 1024);
        assert_eq!(cfg.ttl_seconds, 7 * 24 * 3600);
        assert_eq!(cfg.cleanup_interval_seconds, 300);
        assert_eq!(cfg.keyboard_shortcut, "Super+V");
        assert!(!cfg.encrypt_sensitive);
        assert!(cfg.clear_on_lock);
    }

    #[test]
    fn test_config_roundtrip() {
        let original = Config {
            max_items: 42,
            max_item_size: 2048,
            data_dir: PathBuf::from("/tmp/test-clipboard"),
            ttl_seconds: 3600,
            cleanup_interval_seconds: 60,
            keyboard_shortcut: "Ctrl+Shift+V".to_string(),
            encrypt_sensitive: true,
            clear_on_lock: false,
            dedup_window_seconds: 5,
            mime_denylist: vec!["application/x-secret".to_string()],
            content_denylist: vec!["SECRET".to_string()],
            content_pattern_mode: ContentPatternMode::Substring,
            picker: PickerConfig::default(),
        };
        let json = serde_json::to_string_pretty(&original).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_config_partial_json() {
        let json = r#"{ "max_items": 50 }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.max_items, 50);
        // All other fields should be defaults
        assert_eq!(cfg.max_item_size, 1024 * 1024);
        assert_eq!(cfg.ttl_seconds, 7 * 24 * 3600);
        assert_eq!(cfg.cleanup_interval_seconds, 300);
        assert_eq!(cfg.keyboard_shortcut, "Super+V");
        assert!(!cfg.encrypt_sensitive);
        assert!(cfg.clear_on_lock);
    }

    #[test]
    fn test_mime_denylist() {
        let config = Config {
            mime_denylist: vec!["application/x-secret".to_string()],
            ..Default::default()
        };
        assert!(config.is_mime_denied("application/x-secret"));
        assert!(!config.is_mime_denied("text/plain"));
    }

    #[test]
    fn test_picker_config_defaults() {
        let picker = PickerConfig::default();
        assert_eq!(picker.default_mode, PickerMode::External);
        assert_eq!(picker.default_source, "history");
        assert_eq!(picker.max_results, 50);
        assert!(!picker.show_sensitive_previews);
        assert!(picker.confirm_sensitive_copy);
        assert!(picker.close_after_copy);
        assert!(!picker.prefer_quick_paste);
        assert_eq!(picker.width, 720);
        assert_eq!(picker.height, 520);
    }

    #[test]
    fn test_picker_config_roundtrip() {
        let picker = PickerConfig {
            default_mode: PickerMode::Native,
            default_source: "emoji".to_string(),
            max_results: 100,
            show_sensitive_previews: true,
            confirm_sensitive_copy: false,
            close_after_copy: false,
            prefer_quick_paste: true,
            width: 800,
            height: 600,
        };
        let json = serde_json::to_string_pretty(&picker).unwrap();
        let loaded: PickerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(picker, loaded);
    }

    #[test]
    fn test_picker_config_partial_defaults_default_mode() {
        let json = r#"{ "default_source": "history" }"#;
        let picker: PickerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(picker.default_mode, PickerMode::External);
    }

    #[test]
    fn test_content_denylist_substring() {
        let config = Config {
            content_denylist: vec!["SECRET".to_string()],
            content_pattern_mode: ContentPatternMode::Substring,
            ..Default::default()
        };
        assert!(config.is_content_denied("my SECRET code"));
        assert!(!config.is_content_denied("no secret here"));
    }

    #[test]
    fn test_content_denylist_prefix() {
        let config = Config {
            content_denylist: vec!["OTP:".to_string()],
            content_pattern_mode: ContentPatternMode::Prefix,
            ..Default::default()
        };
        assert!(config.is_content_denied("OTP: 123456"));
        assert!(!config.is_content_denied("not an OTP"));
    }

    #[test]
    fn test_content_denylist_suffix() {
        let config = Config {
            content_denylist: vec![".token".to_string()],
            content_pattern_mode: ContentPatternMode::Suffix,
            ..Default::default()
        };
        assert!(config.is_content_denied("session.token"));
        assert!(!config.is_content_denied("tokenizer"));
    }

    #[test]
    fn test_content_denylist_exact() {
        let config = Config {
            content_denylist: vec!["PASSWORD".to_string()],
            content_pattern_mode: ContentPatternMode::Exact,
            ..Default::default()
        };
        assert!(config.is_content_denied("PASSWORD"));
        assert!(!config.is_content_denied("PASSWORD123"));
    }
}
