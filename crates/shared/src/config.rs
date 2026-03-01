//! Configuration management

use directories::ProjectDirs;
use std::path::PathBuf;

pub struct Config {
    pub max_items: usize,
    pub max_item_size: usize,
    pub data_dir: PathBuf,
    /// Time-to-live for unpinned items (in seconds). 0 = never expire.
    pub ttl_seconds: u64,
    /// How often the cleanup task runs (in seconds).
    pub cleanup_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = ProjectDirs::from("com", "namikofficial", "author-clipboard")
            .map_or_else(|| PathBuf::from("."), |dirs| dirs.data_dir().to_path_buf());

        Self {
            max_items: 100,
            max_item_size: 1024 * 1024, // 1MB
            data_dir,
            ttl_seconds: 7 * 24 * 3600,    // 7 days
            cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

impl Config {
    /// Full path to the `SQLite` database file.
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("clipboard.db")
    }

    /// Path to the incognito mode flag file.
    pub fn incognito_flag_path(&self) -> PathBuf {
        self.data_dir.join(".incognito")
    }

    /// Check if incognito mode is active.
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
