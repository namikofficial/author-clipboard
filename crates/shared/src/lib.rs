//! Shared library for the author-clipboard project.
//!
//! Provides common types, database operations, configuration management,
//! IPC communication, and utility modules used by both the daemon and applet.

/// Content classification (text, code, URL, path, JSON, SQL, etc.).
pub mod classify;
/// Clipboard restore helpers.
pub mod clipboard;
/// Compositor and display server detection utilities.
pub mod compositor;
/// Configuration management with JSON file persistence.
pub mod config;
/// SQLite database operations for clipboard history.
pub mod db;
/// Emoji data organized by category for picker UIs.
pub mod emoji;
/// AES-256-GCM encryption for sensitive clipboard items.
pub mod encryption;
/// File URI parsing and metadata extraction.
pub mod file_handler;
/// Image storage and thumbnail management.
pub mod image_store;
/// Versioned privacy-preserving import/export contracts.
pub mod import_export;
/// Unix domain socket IPC between daemon and clients.
pub mod ipc;
/// Kaomoji (Japanese emoticons) data and search.
pub mod kaomoji;
/// Shared picker types and logic for external menus and native pickers.
pub mod picker;
/// Query string parser for developer filters (type:, app:, project:, collection:).
pub mod query;
/// Quick paste via wtype/ydotool with wl-copy copy fallback.
pub mod quick_paste;
/// Screen lock detection for clearing sensitive clipboard items.
pub mod screen_lock;
/// Sensitive content detection (passwords, tokens, keys).
pub mod sensitive;
/// Keyboard shortcut parsing and conflict detection.
pub mod shortcut;
/// Strict command-center snippet variable compatibility layer.
pub mod snippet_template;
/// Symbol data organized by category for picker UIs.
pub mod symbols;
/// Snippet template rendering with `${name}` variable substitution.
pub mod template;
/// Pure developer-oriented clipboard transformations.
pub mod transform;
/// Core data types for clipboard items and events.
pub mod types;

pub use config::Config;
pub use db::Database;
pub use types::*;
