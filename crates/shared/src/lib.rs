//! Shared library for the author-clipboard project.
//!
//! Provides common types, database operations, configuration management,
//! IPC communication, and utility modules used by both the daemon and applet.

/// Compositor and display server detection utilities.
pub mod compositor;
/// Configuration management with JSON file persistence.
pub mod config;
/// SQLite database operations for clipboard history.
pub mod db;
/// AES-256-GCM encryption for sensitive clipboard items.
pub mod encryption;
/// File URI parsing and metadata extraction.
pub mod file_handler;
/// Image storage and thumbnail management.
pub mod image_store;
/// Unix domain socket IPC between daemon and clients.
pub mod ipc;
/// Quick paste via wtype or ydotool backends.
pub mod quick_paste;
/// Screen lock detection for clearing sensitive clipboard items.
pub mod screen_lock;
/// Sensitive content detection (passwords, tokens, keys).
pub mod sensitive;
/// Keyboard shortcut parsing and conflict detection.
pub mod shortcut;
/// Core data types for clipboard items and events.
pub mod types;

pub use config::Config;
pub use db::Database;
pub use types::*;
