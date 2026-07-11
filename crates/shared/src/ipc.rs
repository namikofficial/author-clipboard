//! Unix domain socket IPC module for daemon-applet communication.
//!
//! Provides message types and a simple client/server implementation
//! for communication between the clipboard daemon and the applet
//! over a Unix domain socket using JSON-line wire format.

#![allow(dead_code)]

use std::io::{BufRead, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Protocol version for IPC requests/responses.
pub const IPC_VERSION: &str = "1.0";

/// Messages exchanged between daemon and applet over IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Toggle the applet visibility.
    Toggle,
    /// Show the applet.
    Show,
    /// Hide the applet.
    Hide,
    /// Show the applet at a specific screen position.
    ShowAt { x: i32, y: i32 },
    /// Ping request (health check).
    Ping,
    /// Pong response (health check reply).
    Pong,
    /// Status report from the daemon.
    Status { visible: bool, item_count: usize },
    /// Versioned request for normalized service API.
    Request(IpcRequest),
    /// Versioned response for normalized service API.
    Response(IpcResponse),
}

/// Versioned IPC request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcRequest {
    /// Protocol version (e.g., "1.0").
    pub version: String,
    /// Command name.
    pub cmd: String,
    /// Command arguments (JSON).
    pub args: serde_json::Value,
    /// Optional request ID for tracking.
    pub request_id: Option<u64>,
}

impl IpcRequest {
    /// Create a new request with the given command and arguments.
    pub fn new(cmd: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            cmd: cmd.into(),
            args,
            request_id: None,
        }
    }

    /// Create a new request with a request ID.
    pub fn with_id(cmd: impl Into<String>, args: serde_json::Value, request_id: u64) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            cmd: cmd.into(),
            args,
            request_id: Some(request_id),
        }
    }
}

/// Versioned IPC response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Protocol version.
    pub version: String,
    /// Whether the request succeeded.
    pub ok: bool,
    /// Response data (on success).
    pub data: Option<serde_json::Value>,
    /// Error details (on failure).
    pub error: Option<IpcErrorDetail>,
}

impl IpcResponse {
    /// Create a success response.
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response.
    pub fn err(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            ok: false,
            data: None,
            error: Some(IpcErrorDetail {
                code: code.into(),
                message: message.into(),
                min_version: None,
            }),
        }
    }

    /// Create an error response with minimum version.
    pub fn err_with_min_version(
        code: impl Into<String>,
        message: impl Into<String>,
        min_version: impl Into<String>,
    ) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            ok: false,
            data: None,
            error: Some(IpcErrorDetail {
                code: code.into(),
                message: message.into(),
                min_version: Some(min_version.into()),
            }),
        }
    }
}

/// Error details in an IPC response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcErrorDetail {
    /// Error code (e.g., `UNKNOWN_COMMAND`, `SENSITIVE_CONTENT`).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Minimum protocol version required (for version errors).
    pub min_version: Option<String>,
}

/// Filter options for query commands.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterOptions {
    /// Filter by content types (e.g., `["text", "html"]`).
    pub content_type: Option<Vec<String>>,
    /// Filter by pinned state.
    pub pinned: Option<bool>,
    /// Filter by sensitive state.
    pub sensitive: Option<bool>,
    /// Filter by source app.
    pub source_app: Option<String>,
    /// Filter items newer than this many seconds.
    pub age_min_seconds: Option<u64>,
    /// Filter items older than this many seconds.
    pub age_max_seconds: Option<u64>,
}

/// Copy mode for copy operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CopyMode {
    /// Write to clipboard only.
    #[default]
    Copy,
    /// Write to clipboard and type into active window.
    QuickPaste,
    /// Strip formatting before copying.
    CopyPlainText,
    /// Replace sensitive patterns with bullets before copying.
    CopyRedacted,
}

/// IPC commands for the normalized service API.
/// These are sent as `IpcRequest` with cmd field set to the variant name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum IpcCommand {
    // ── Visibility ────────────────────────────────────────────────
    /// Toggle picker visibility.
    Toggle,
    /// Show the picker.
    Show,
    /// Hide the picker.
    Hide,
    /// Show picker at specific coordinates.
    ShowAt { x: i32, y: i32 },

    // ── Health ────────────────────────────────────────────────────
    /// Health check.
    Ping,
    /// Detailed daemon status.
    Status,

    // ── Query ────────────────────────────────────────────────────
    /// Get recent clipboard items with optional filtering.
    History {
        /// Maximum number of items to return.
        limit: usize,
        /// Offset for pagination.
        offset: Option<usize>,
        /// Filter options.
        filters: Option<FilterOptions>,
    },
    /// Get a single item by ID.
    GetItem { id: i64 },
    /// Full-text search with filtering.
    Search {
        /// Search query string.
        query: String,
        /// Maximum number of results.
        limit: Option<usize>,
        /// Filter options.
        filters: Option<FilterOptions>,
    },
    /// Get database statistics.
    GetStats,
    /// Get recent audit log entries.
    GetAuditLog {
        /// Maximum number of entries to return.
        limit: Option<usize>,
    },

    // ── Mutations ────────────────────────────────────────────────
    /// Copy an item to the clipboard.
    Copy {
        /// Item ID to copy.
        id: i64,
        /// Copy mode.
        mode: CopyMode,
        /// Optional MIME type hint for the copy operation.
        #[serde(default)]
        mime: Option<String>,
    },
    /// Apply a shared pure transformation without mutating history.
    Transform {
        content: String,
        transform: crate::transform::TransformKind,
        #[serde(default)]
        sensitive: bool,
        #[serde(default)]
        confirm_sensitive: bool,
    },
    /// Pin an item.
    Pin { id: i64 },
    /// Unpin an item.
    Unpin { id: i64 },
    /// Toggle star on an item.
    ToggleStar { id: i64 },
    /// Delete a single item.
    Delete { id: i64 },
    /// Delete all unpinned items.
    ClearUnpinned,
    /// Delete all items including pinned.
    ClearAll,

    // ── Snippets ──────────────────────────────────────────────────
    /// List all snippets.
    ListSnippets,
    /// Create or update a snippet.
    UpsertSnippet {
        /// Snippet name.
        name: String,
        /// Snippet content.
        content: String,
    },
    /// Delete a snippet.
    DeleteSnippet { id: i64 },
    /// Render a snippet template against the daemon's current context.
    ///
    /// Response payload on success:
    /// ```json
    /// { "content": "<rendered text>", "cursor_offset": <usize or null> }
    /// ```
    /// On missing id, returns an error with code `SNIPPET_NOT_FOUND`.
    ///
    /// See `specs/features/026-snippet-templates/`.
    RenderSnippet { id: i64 },

    // ── Collections ───────────────────────────────────────────────
    /// List all collections.
    ListCollections,
    /// Create a new collection.
    CreateCollection {
        /// Collection name.
        name: String,
    },
    /// Delete a collection.
    DeleteCollection {
        /// Collection ID.
        id: String,
    },
    /// Rename a collection.
    RenameCollection {
        /// Collection ID.
        id: String,
        /// New name.
        new_name: String,
    },
    /// Get items in a collection.
    GetCollectionItems {
        /// Collection ID.
        id: String,
    },
    /// Add item to a collection.
    AddToCollection {
        /// Collection ID.
        collection_id: String,
        /// Item ID.
        item_id: i64,
    },
    /// Remove item from a collection.
    RemoveFromCollection {
        /// Collection ID.
        collection_id: String,
        /// Item ID.
        item_id: i64,
    },

    // ── Config ────────────────────────────────────────────────────
    /// Get current configuration.
    GetConfig,
    /// Update configuration.
    UpdateConfig {
        /// Configuration values to update.
        config: serde_json::Value,
    },
}

/// Errors that can occur during IPC operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum IpcError {
    /// Failed to connect to the IPC socket.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Failed to send a message.
    #[error("send failed: {0}")]
    SendFailed(String),

    /// Failed to receive a message.
    #[error("receive failed: {0}")]
    ReceiveFailed(String),

    /// Received an invalid or unparseable message.
    #[error("invalid message: {0}")]
    InvalidMessage(String),

    /// The socket is already in use by another process.
    #[error("socket already in use")]
    SocketInUse,
}

/// Returns the default IPC socket path.
///
/// Uses `$XDG_RUNTIME_DIR/author-clipboard.sock` if available.
/// Falls back to a private cache directory with restricted permissions
/// instead of world-writable `/tmp` to prevent symlink attacks.
pub fn socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("author-clipboard.sock");
    }

    // Fallback: private cache directory (not /tmp)
    let cache_dir = directories::ProjectDirs::from("com", "namikofficial", "author-clipboard")
        .map_or_else(
            || {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".cache/author-clipboard")
            },
            |dirs| dirs.cache_dir().to_path_buf(),
        );
    let _ = std::fs::create_dir_all(&cache_dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&cache_dir, std::fs::Permissions::from_mode(0o700));
    }
    cache_dir.join("author-clipboard.sock")
}

/// Remove the default IPC socket file, ignoring errors if it does not exist.
pub fn remove_ipc_socket() {
    let _ = std::fs::remove_file(socket_path());
}

/// IPC server that listens for incoming connections on a Unix domain socket.
///
/// The socket file is automatically removed when the server is dropped.
#[derive(Debug)]
pub struct IpcServer {
    listener: UnixListener,
    path: PathBuf,
}

impl IpcServer {
    /// Bind to the default IPC socket path.
    pub fn bind() -> Result<Self, IpcError> {
        Self::bind_at(&socket_path())
    }

    /// Bind to a specific socket path.
    ///
    /// If the socket file already exists, attempts to detect whether it is
    /// stale (no process listening) and removes it before retrying.
    pub fn bind_at(path: &Path) -> Result<Self, IpcError> {
        match UnixListener::bind(path) {
            Ok(listener) => Ok(Self {
                listener,
                path: path.to_path_buf(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                // Check if the existing socket is stale or actively in use
                if UnixStream::connect(path).is_ok() {
                    return Err(IpcError::SocketInUse);
                }
                // Stale socket — remove and retry
                std::fs::remove_file(path)
                    .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;
                let listener = UnixListener::bind(path)
                    .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;
                Ok(Self {
                    listener,
                    path: path.to_path_buf(),
                })
            }
            Err(e) => Err(IpcError::ConnectionFailed(e.to_string())),
        }
    }

    /// Accept a single incoming connection and read one message.
    pub fn accept(&self) -> Result<IpcMessage, IpcError> {
        let (stream, _addr) = self
            .listener
            .accept()
            .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;
        let mut reader = std::io::BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;
        serde_json::from_str(line.trim()).map_err(|e| IpcError::InvalidMessage(e.to_string()))
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// IPC client that connects to the daemon's Unix domain socket.
#[derive(Debug, Clone)]
pub struct IpcClient {
    path: PathBuf,
}

impl IpcClient {
    /// Create a client that connects to the default socket path.
    pub fn new() -> Self {
        Self {
            path: socket_path(),
        }
    }

    /// Create a client that connects to a specific socket path.
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Send a message and optionally receive a response.
    ///
    /// Returns `Ok(None)` if the server closes the connection without
    /// sending a response.
    pub fn send(&self, message: &IpcMessage) -> Result<Option<IpcMessage>, IpcError> {
        let mut stream = UnixStream::connect(&self.path)
            .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;

        // Write JSON message followed by newline
        let json =
            serde_json::to_string(message).map_err(|e| IpcError::InvalidMessage(e.to_string()))?;
        writeln!(stream, "{json}").map_err(|e| IpcError::SendFailed(e.to_string()))?;
        stream
            .flush()
            .map_err(|e| IpcError::SendFailed(e.to_string()))?;

        // Signal that we are done writing
        stream
            .shutdown(std::net::Shutdown::Write)
            .map_err(|e| IpcError::SendFailed(e.to_string()))?;

        // Try to read an optional response
        let mut reader = std::io::BufReader::new(stream);
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => Ok(None),
            Ok(_) if line.trim().is_empty() => Ok(None),
            Ok(_) => {
                let msg = serde_json::from_str(line.trim())
                    .map_err(|e| IpcError::InvalidMessage(e.to_string()))?;
                Ok(Some(msg))
            }
        }
    }

    /// Convenience method to send a `Toggle` message.
    pub fn send_toggle(&self) -> Result<(), IpcError> {
        self.send(&IpcMessage::Toggle)?;
        Ok(())
    }

    /// Send a versioned request and receive a versioned response.
    pub fn send_request(&self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        let response = self.send(&IpcMessage::Request(request.clone()))?;
        match response {
            Some(IpcMessage::Response(resp)) => Ok(resp),
            Some(other) => Err(IpcError::InvalidMessage(format!(
                "Expected Response message, got: {other:?}"
            ))),
            None => Err(IpcError::ReceiveFailed(
                "Server closed connection".to_string(),
            )),
        }
    }

    /// Send a command and parse the response data.
    pub fn send_command(&self, cmd: &IpcCommand) -> Result<IpcResponse, IpcError> {
        let args =
            serde_json::to_value(cmd).map_err(|e| IpcError::InvalidMessage(e.to_string()))?;
        let cmd_name = match &cmd {
            IpcCommand::Toggle => "Toggle",
            IpcCommand::Show => "Show",
            IpcCommand::Hide => "Hide",
            IpcCommand::ShowAt { .. } => "ShowAt",
            IpcCommand::Ping => "Ping",
            IpcCommand::Status => "Status",
            IpcCommand::History { .. } => "History",
            IpcCommand::GetItem { .. } => "GetItem",
            IpcCommand::Search { .. } => "Search",
            IpcCommand::GetStats => "GetStats",
            IpcCommand::GetAuditLog { .. } => "GetAuditLog",
            IpcCommand::Copy { .. } => "Copy",
            IpcCommand::Transform { .. } => "Transform",
            IpcCommand::Pin { .. } => "Pin",
            IpcCommand::Unpin { .. } => "Unpin",
            IpcCommand::Delete { .. } => "Delete",
            IpcCommand::ClearUnpinned => "ClearUnpinned",
            IpcCommand::ClearAll => "ClearAll",
            IpcCommand::ListSnippets => "ListSnippets",
            IpcCommand::UpsertSnippet { .. } => "UpsertSnippet",
            IpcCommand::DeleteSnippet { .. } => "DeleteSnippet",
            IpcCommand::RenderSnippet { .. } => "RenderSnippet",
            IpcCommand::ToggleStar { .. } => "ToggleStar",
            IpcCommand::ListCollections => "ListCollections",
            IpcCommand::CreateCollection { .. } => "CreateCollection",
            IpcCommand::DeleteCollection { .. } => "DeleteCollection",
            IpcCommand::RenameCollection { .. } => "RenameCollection",
            IpcCommand::GetCollectionItems { .. } => "GetCollectionItems",
            IpcCommand::AddToCollection { .. } => "AddToCollection",
            IpcCommand::RemoveFromCollection { .. } => "RemoveFromCollection",
            IpcCommand::GetConfig => "GetConfig",
            IpcCommand::UpdateConfig { .. } => "UpdateConfig",
        };
        let request = IpcRequest::new(cmd_name, args);
        self.send_request(&request)
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let messages = vec![
            IpcMessage::Toggle,
            IpcMessage::Show,
            IpcMessage::Hide,
            IpcMessage::ShowAt { x: 100, y: 200 },
            IpcMessage::Ping,
            IpcMessage::Pong,
            IpcMessage::Status {
                visible: true,
                item_count: 42,
            },
        ];

        for msg in &messages {
            let json = serde_json::to_string(msg).expect("serialize");
            let deserialized: IpcMessage = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*msg, deserialized, "roundtrip failed for {json}");
        }
    }

    #[test]
    fn test_socket_path_not_empty() {
        let path = socket_path();
        assert!(
            !path.to_str().is_none_or(str::is_empty),
            "socket path should not be empty"
        );
        assert!(
            path.to_string_lossy().ends_with("author-clipboard.sock"),
            "socket path should end with author-clipboard.sock"
        );
    }

    #[test]
    fn test_server_client_roundtrip() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let sock = dir.path().join("test.sock");

        let server = IpcServer::bind_at(&sock).expect("bind server");

        let handle = std::thread::spawn(move || server.accept().expect("accept message"));

        let client = IpcClient::with_path(sock);
        let _response = client.send(&IpcMessage::Ping).expect("send message");

        let received = handle.join().expect("server thread");
        assert_eq!(received, IpcMessage::Ping);
    }

    #[test]
    fn test_client_connection_refused() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let sock = dir.path().join("nonexistent.sock");

        let client = IpcClient::with_path(sock);
        let result = client.send(&IpcMessage::Ping);
        assert!(result.is_err(), "should fail when no server is listening");
    }

    #[test]
    fn test_server_stale_socket_cleanup() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let sock = dir.path().join("stale.sock");

        // Create a stale socket file
        std::fs::write(&sock, "").expect("create stale file");

        // Binding should succeed after removing the stale file
        let server = IpcServer::bind_at(&sock);
        assert!(server.is_ok(), "should handle stale socket");
    }

    #[test]
    fn test_show_at_message() {
        let msg = IpcMessage::ShowAt { x: 50, y: 75 };
        let json = serde_json::to_string(&msg).expect("serialize");
        assert!(json.contains("ShowAt"));
        assert!(json.contains("50"));
        assert!(json.contains("75"));

        let deserialized: IpcMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn test_ipc_request_response_roundtrip() {
        let request = IpcRequest::new("History", serde_json::json!({"limit": 10}));
        let json = serde_json::to_string(&request).expect("serialize");
        let deserialized: IpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(request.version, deserialized.version);
        assert_eq!(request.cmd, deserialized.cmd);

        let response = IpcResponse::ok(serde_json::json!({"items": []}));
        let json = serde_json::to_string(&response).expect("serialize");
        let deserialized: IpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.ok);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_copy_mode_serialization() {
        let modes = vec![
            CopyMode::Copy,
            CopyMode::QuickPaste,
            CopyMode::CopyPlainText,
            CopyMode::CopyRedacted,
        ];
        for mode in &modes {
            let json = serde_json::to_string(mode).expect("serialize");
            let deserialized: CopyMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*mode, deserialized);
        }
    }

    #[test]
    fn test_filter_options_default() {
        let filters = FilterOptions::default();
        assert!(filters.content_type.is_none());
        assert!(filters.pinned.is_none());
        assert!(filters.sensitive.is_none());
    }

    #[test]
    fn test_copy_mime_defaults_to_none_when_omitted() {
        // IpcCommand uses #[serde(tag = "cmd", content = "args")], so the
        // on-wire shape is {"cmd":"Copy","args":{"id":1,"mode":"copy"}}.
        // The #[serde(default)] on IpcCommand::Copy.mime is the invariant
        // we defend; if anyone removes it, this test fails.
        let json = r#"{"cmd":"Copy","args":{"id":1,"mode":"copy"}}"#;
        let cmd: IpcCommand = serde_json::from_str(json).expect("old payload deserializes");
        match cmd {
            IpcCommand::Copy { id, mode, mime } => {
                assert_eq!(id, 1);
                assert!(matches!(mode, CopyMode::Copy));
                assert!(mime.is_none(), "mime must default to None when omitted");
            }
            _ => panic!("expected Copy variant"),
        }
    }

    #[test]
    fn test_copy_with_mime_roundtrip() {
        let cmd = IpcCommand::Copy {
            id: 7,
            mode: CopyMode::Copy,
            mime: Some("image/png".to_string()),
        };
        let v = serde_json::to_value(&cmd).expect("serialize");
        let parsed: IpcCommand = serde_json::from_value(v).expect("roundtrip");
        match parsed {
            IpcCommand::Copy { id, mime, .. } => {
                assert_eq!(id, 7);
                assert_eq!(mime.as_deref(), Some("image/png"));
            }
            _ => panic!("expected Copy variant"),
        }
    }
}
