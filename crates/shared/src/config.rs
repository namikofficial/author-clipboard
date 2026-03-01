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
}
