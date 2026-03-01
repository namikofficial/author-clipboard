//! Configuration management

use directories::ProjectDirs;
use std::path::PathBuf;

pub struct Config {
    pub max_items: usize,
    pub max_item_size: usize,
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = ProjectDirs::from("com", "namik", "author-clipboard")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            max_items: 100,
            max_item_size: 1024 * 1024, // 1MB
            data_dir,
        }
    }
}
