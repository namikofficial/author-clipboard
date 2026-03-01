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
        let data_dir = ProjectDirs::from("com", "namik", "author-clipboard")
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
}
