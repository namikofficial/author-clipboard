//! Typed asynchronous access to the clipboard daemon.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use author_clipboard_shared::ipc::{CopyMode, FilterOptions, IpcClient, IpcCommand, IpcResponse};
use author_clipboard_shared::picker::{PickerEntry, PickerFilter, PickerSource};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::oneshot;

/// Default connection timeout.
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
/// Default write timeout.
pub const WRITE_TIMEOUT: Duration = Duration::from_millis(500);
/// Default response timeout.
pub const RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);

/// Errors exposed to GTK instead of leaking transport details into callbacks.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ServiceError {
    /// The daemon socket could not be reached.
    #[error("daemon offline: {0}")]
    Offline(String),
    /// A request exceeded a transport deadline.
    #[error("daemon request timed out during {stage}")]
    Timeout {
        /// Transport phase that exceeded its deadline.
        stage: &'static str,
    },
    /// The daemon returned an invalid or mismatched message.
    #[error("protocol error: {0}")]
    Protocol(String),
    /// The request or response failed validation.
    #[error("validation error: {0}")]
    Validation(String),
    /// The daemon reported a database failure.
    #[error("database error: {0}")]
    Database(String),
    /// The daemon rejected the operation.
    #[error("daemon error: {0}")]
    Daemon(String),
    /// The service worker was shut down.
    #[error("service worker stopped")]
    WorkerStopped,
}

/// A history/search request.
#[derive(Debug, Clone)]
pub struct HistoryRequest {
    /// Search text; empty means recent history.
    pub query: String,
    /// Maximum number of entries.
    pub limit: usize,
    /// UI filter.
    pub filter: PickerFilter,
    /// Data source.
    pub source: PickerSource,
    /// Whether protected entries may be returned.
    pub include_sensitive: bool,
    /// Monotonic UI generation used to discard stale responses.
    pub generation: u64,
}

/// A copy request.
#[derive(Debug, Clone)]
pub struct CopyRequest {
    /// Item ID.
    pub id: i64,
    /// Copy mode.
    pub mode: CopyMode,
    /// MIME hint.
    pub mime: Option<String>,
}

/// A daemon status snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonStatus {
    /// Number of stored items.
    pub item_count: usize,
    /// Daemon process ID when reported.
    pub daemon_pid: Option<u32>,
}

/// Typed service boundary used by GTK pages and tests.
#[async_trait]
pub trait ClipboardService: Send + Sync {
    /// Load history or source entries.
    async fn history(&self, request: HistoryRequest) -> Result<Vec<PickerEntry>, ServiceError>;
    /// Copy one item.
    async fn copy(&self, request: CopyRequest) -> Result<(), ServiceError>;
    /// Toggle or update a selected item through a daemon command.
    async fn update_item(&self, command: IpcCommand) -> Result<Value, ServiceError>;
    /// Read daemon status.
    async fn status(&self) -> Result<DaemonStatus, ServiceError>;
    /// Execute a typed collection/snippet/config command.
    async fn command(&self, command: IpcCommand) -> Result<Value, ServiceError>;
}

enum WorkerRequest {
    Command {
        command: IpcCommand,
        reply: oneshot::Sender<Result<IpcResponse, ServiceError>>,
    },
}

/// Cloneable handle that enqueues work without doing I/O on the caller thread.
#[derive(Clone)]
pub struct ServiceHandle {
    tx: mpsc::Sender<WorkerRequest>,
}

impl std::fmt::Debug for ServiceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceHandle").finish_non_exhaustive()
    }
}

impl ServiceHandle {
    /// Start the worker and connect lazily on the first request.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("author-clipboard-ipc".into())
            .spawn(move || worker_loop(rx))
            .expect("failed to start clipboard service worker");
        Self { tx }
    }

    async fn request(&self, command: IpcCommand) -> Result<Value, ServiceError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(WorkerRequest::Command {
                command,
                reply: reply_tx,
            })
            .map_err(|_| ServiceError::WorkerStopped)?;
        let response = reply_rx.await.map_err(|_| ServiceError::WorkerStopped)??;
        if response.ok {
            response
                .data
                .ok_or_else(|| ServiceError::Protocol("successful response had no data".into()))
        } else {
            let detail = response.error.ok_or_else(|| {
                ServiceError::Protocol("error response had no error detail".into())
            })?;
            match detail.code.as_str() {
                "DB_ERROR" => Err(ServiceError::Database(detail.message)),
                "INVALID_REQUEST" | "VALIDATION_ERROR" => {
                    Err(ServiceError::Validation(detail.message))
                }
                _ => Err(ServiceError::Daemon(detail.message)),
            }
        }
    }
}

impl Default for ServiceHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClipboardService for ServiceHandle {
    async fn history(&self, request: HistoryRequest) -> Result<Vec<PickerEntry>, ServiceError> {
        let filter = Some(filter_options(request.filter, request.include_sensitive));
        let command = if request.source == PickerSource::Snippets {
            IpcCommand::ListSnippets
        } else if request.query.trim().is_empty() {
            IpcCommand::History {
                limit: request.limit,
                offset: None,
                filters: filter,
            }
        } else {
            IpcCommand::Search {
                query: request.query,
                limit: Some(request.limit),
                filters: filter,
            }
        };
        let data = self.request(command).await?;
        let values = data
            .get(if request.source == PickerSource::Snippets {
                "snippets"
            } else {
                "items"
            })
            .and_then(Value::as_array)
            .ok_or_else(|| ServiceError::Protocol("response payload had no result array".into()))?;
        if request.source == PickerSource::Snippets {
            return values.iter().map(snippet_to_entry).collect();
        }
        values.iter().map(value_to_entry).collect::<Result<_, _>>()
    }

    async fn copy(&self, request: CopyRequest) -> Result<(), ServiceError> {
        self.request(IpcCommand::Copy {
            id: request.id,
            mode: request.mode,
            mime: request.mime,
        })
        .await
        .map(|_| ())
    }

    async fn update_item(&self, command: IpcCommand) -> Result<Value, ServiceError> {
        self.request(command).await
    }

    async fn status(&self) -> Result<DaemonStatus, ServiceError> {
        let data = self.request(IpcCommand::Status).await?;
        let item_count = data
            .get("item_count")
            .and_then(Value::as_u64)
            .ok_or_else(|| ServiceError::Protocol("status has no item_count".into()))?;
        Ok(DaemonStatus {
            item_count: usize::try_from(item_count)
                .map_err(|_| ServiceError::Validation("item count overflow".into()))?,
            daemon_pid: data
                .get("daemon_pid")
                .and_then(Value::as_u64)
                .and_then(|pid| u32::try_from(pid).ok()),
        })
    }

    async fn command(&self, command: IpcCommand) -> Result<Value, ServiceError> {
        self.request(command).await
    }
}

fn worker_loop(rx: mpsc::Receiver<WorkerRequest>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("failed to build clipboard service runtime");
    for request in rx {
        match request {
            WorkerRequest::Command { command, reply } => {
                let result = runtime.block_on(send_command(command));
                let _ = reply.send(result);
            }
        }
    }
}

async fn send_command(command: IpcCommand) -> Result<IpcResponse, ServiceError> {
    let send = tokio::task::spawn_blocking(move || {
        IpcClient::new()
            .with_timeouts(WRITE_TIMEOUT, RESPONSE_TIMEOUT)
            .send_command(&command)
    });
    match tokio::time::timeout(CONNECT_TIMEOUT + WRITE_TIMEOUT + RESPONSE_TIMEOUT, send).await {
        Err(_) => Err(ServiceError::Timeout { stage: "response" }),
        Ok(Err(_)) => Err(ServiceError::WorkerStopped),
        Ok(Ok(Err(error))) => Err(map_ipc_error(error)),
        Ok(Ok(Ok(response))) => Ok(response),
    }
}

fn map_ipc_error(error: author_clipboard_shared::ipc::IpcError) -> ServiceError {
    use author_clipboard_shared::ipc::IpcError;
    match error {
        IpcError::ConnectionFailed(message) => ServiceError::Offline(message),
        IpcError::SendFailed(_message) => ServiceError::Timeout { stage: "write" },
        IpcError::ReceiveFailed(message) => {
            if message.contains("timed out") || message.contains("would block") {
                ServiceError::Timeout { stage: "response" }
            } else {
                ServiceError::Offline(message)
            }
        }
        IpcError::InvalidMessage(message) => ServiceError::Protocol(message),
        IpcError::SocketInUse => ServiceError::Offline("socket is in use".into()),
    }
}

fn filter_options(filter: PickerFilter, include_sensitive: bool) -> FilterOptions {
    let mut options = FilterOptions {
        sensitive: (!include_sensitive).then_some(false),
        ..Default::default()
    };
    match filter {
        PickerFilter::Pinned => options.pinned = Some(true),
        PickerFilter::Sensitive => options.sensitive = Some(true),
        _ => {}
    }
    options
}

fn value_to_entry(value: &Value) -> Result<PickerEntry, ServiceError> {
    let content_type = value
        .get("content_type")
        .and_then(Value::as_str)
        .ok_or_else(|| ServiceError::Protocol("result has no content_type".into()))?
        .parse()
        .map_err(|_| ServiceError::Validation("unknown content_type".into()))?;
    let content = value
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| ServiceError::Protocol("result has no content".into()))?;
    Ok(PickerEntry {
        id: value.get("id").and_then(Value::as_i64),
        source: PickerSource::History,
        content_type: Some(content_type),
        title: value
            .get("plain_text")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .unwrap_or(content)
            .to_owned(),
        subtitle: value
            .get("preview")
            .and_then(Value::as_str)
            .map(str::to_owned),
        content: content.to_owned(),
        mime_type: value
            .get("mime_type")
            .and_then(Value::as_str)
            .map(str::to_owned),
        sensitive: value
            .get("sensitive")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        pinned: value
            .get("pinned")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        starred: value
            .get("starred")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        timestamp: value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok()),
    })
}

fn snippet_to_entry(value: &Value) -> Result<PickerEntry, ServiceError> {
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ServiceError::Protocol("snippet has no name".into()))?;
    let content = value
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| ServiceError::Protocol("snippet has no content".into()))?;
    Ok(PickerEntry {
        id: value.get("id").and_then(Value::as_i64),
        source: PickerSource::Snippets,
        content_type: Some(author_clipboard_shared::types::ContentType::Text),
        title: name.to_owned(),
        subtitle: Some("snippet".into()),
        content: content.to_owned(),
        mime_type: Some("text/plain".into()),
        sensitive: false,
        pinned: false,
        starred: false,
        timestamp: None,
    })
}

/// Convert an IPC failure into a user-facing status message.
pub fn error_message(error: &ServiceError) -> String {
    error.to_string()
}

/// Return whether a response generation is still eligible to update the UI.
pub fn accepts_generation(latest: u64, response: u64) -> bool {
    latest == response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockService {
        delay: Duration,
        result: Result<Vec<PickerEntry>, ServiceError>,
    }

    #[async_trait]
    impl ClipboardService for MockService {
        async fn history(
            &self,
            _request: HistoryRequest,
        ) -> Result<Vec<PickerEntry>, ServiceError> {
            tokio::time::sleep(self.delay).await;
            self.result.clone()
        }

        async fn copy(&self, _request: CopyRequest) -> Result<(), ServiceError> {
            Ok(())
        }
        async fn update_item(&self, _command: IpcCommand) -> Result<Value, ServiceError> {
            Ok(Value::Null)
        }
        async fn status(&self) -> Result<DaemonStatus, ServiceError> {
            Err(ServiceError::Offline("mock".into()))
        }
        async fn command(&self, _command: IpcCommand) -> Result<Value, ServiceError> {
            Ok(Value::Null)
        }
    }

    #[test]
    fn stale_search_generations_are_rejected() {
        assert!(accepts_generation(4, 4));
        assert!(!accepts_generation(5, 4));
    }

    #[test]
    fn service_errors_are_explicit() {
        assert!(error_message(&ServiceError::Offline("missing socket".into())).contains("offline"));
        assert!(error_message(&ServiceError::Timeout { stage: "response" }).contains("timed out"));
        assert!(error_message(&ServiceError::Protocol("bad json".into())).contains("protocol"));
        assert!(error_message(&ServiceError::Validation("bad id".into())).contains("validation"));
        assert!(error_message(&ServiceError::Database("locked".into())).contains("database"));
    }

    #[tokio::test]
    async fn mock_service_can_model_delayed_and_offline_daemons() {
        let delayed = MockService {
            delay: Duration::from_millis(5),
            result: Ok(Vec::new()),
        };
        let result = delayed
            .history(HistoryRequest {
                query: "new".into(),
                limit: 10,
                filter: PickerFilter::All,
                source: PickerSource::History,
                include_sensitive: false,
                generation: 2,
            })
            .await;
        assert!(result.is_ok_and(|entries| entries.is_empty()));
        assert_eq!(
            delayed.status().await,
            Err(ServiceError::Offline("mock".into()))
        );
    }
}
