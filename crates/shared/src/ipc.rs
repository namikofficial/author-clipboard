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
/// Uses `$XDG_RUNTIME_DIR/author-clipboard.sock` if available,
/// falls back to `/tmp/author-clipboard.sock`.
pub fn socket_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
        .join("author-clipboard.sock")
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
            !path.as_os_str().is_empty(),
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
}
