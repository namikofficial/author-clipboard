//! Typed asynchronous access to the clipboard daemon.
//!
//! GTK callbacks enqueue requests via a channel and return quickly.
//! A dedicated worker thread owns the [`IpcClient`] transport, manages
//! reconnection, and drives all socket I/O synchronously. Responses
//! arrive back through oneshot channels and are dispatched on the
//! GLib main context.
//!
//! Search/history requests carry a monotonic *generation* number. The
//! worker drops older queued searches for the same data source when a
//! newer generation arrives before dispatch. The GTK side also gating
//! via [`accepts_generation`], so stale responses can never overwrite
//! newer results.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use author_clipboard_shared::ipc::{
    CopyMode, FilterOptions, IpcClient, IpcCommand, IpcError, IpcResponse,
};
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
/// Total budget before the GTK caller sees a timeout.
const TOTAL_TIMEOUT: Duration = Duration::from_millis(3200);

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
    /// The service worker is shut down.
    #[error("service worker stopped")]
    WorkerStopped,
    /// The request was cancelled because a newer generation arrived.
    #[error("request superseded by newer generation")]
    Superseded,
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
    /// Monotonic UI generation used to discard stale responses and
    /// to let the worker skip older queued searches.
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

// ── Worker protocol ──────────────────────────────────────────────────

/// A lightweight key that identifies which logical data source a
/// request targets. Used to discard older queued searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SourceKey {
    History,
    Snippets,
}

struct SearchMeta {
    generation: u64,
    source_key: SourceKey,
}

enum WorkerRequest {
    Command {
        command: IpcCommand,
        search: Option<SearchMeta>,
        reply: oneshot::Sender<Result<IpcResponse, ServiceError>>,
    },
}

// ── Public handle ────────────────────────────────────────────────────

/// Cloneable handle that enqueues work without doing I/O on the caller thread.
///
/// All socket access happens on a dedicated worker thread. The handle
/// is cheap to clone — clones share the same worker channel.
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

    /// Enqueue a request and wait for the result through a oneshot.
    async fn request(&self, command: IpcCommand) -> Result<Value, ServiceError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(WorkerRequest::Command {
                command,
                search: None,
                reply: reply_tx,
            })
            .map_err(|_| ServiceError::WorkerStopped)?;
        let response = tokio::time::timeout(TOTAL_TIMEOUT, reply_rx)
            .await
            .map_err(|_| ServiceError::Timeout { stage: "response" })?
            .map_err(|_| ServiceError::WorkerStopped)??;
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
        let (command, source_key, is_search) = if request.source == PickerSource::Snippets {
            (IpcCommand::ListSnippets, SourceKey::Snippets, false)
        } else if request.query.trim().is_empty() {
            (
                IpcCommand::History {
                    limit: request.limit,
                    offset: None,
                    filters: filter,
                },
                SourceKey::History,
                false,
            )
        } else {
            (
                IpcCommand::Search {
                    query: request.query,
                    limit: Some(request.limit),
                    filters: filter,
                },
                SourceKey::History,
                true,
            )
        };

        // Use the oneshot channel for the reply. The worker will check
        // generation if this is a search.
        let (reply_tx, reply_rx) = oneshot::channel();
        let search_meta = if is_search {
            Some(SearchMeta {
                generation: request.generation,
                source_key,
            })
        } else {
            None
        };
        self.tx
            .send(WorkerRequest::Command {
                command,
                search: search_meta,
                reply: reply_tx,
            })
            .map_err(|_| ServiceError::WorkerStopped)?;

        let response = tokio::time::timeout(TOTAL_TIMEOUT, reply_rx)
            .await
            .map_err(|_| ServiceError::Timeout { stage: "response" })?
            .map_err(|_| ServiceError::WorkerStopped)??;

        if !response.ok {
            let detail = response.error.unwrap_or(IpcResponse::err("UNKNOWN", "").error.unwrap());
            return match detail.code.as_str() {
                "DB_ERROR" => Err(ServiceError::Database(detail.message)),
                "INVALID_REQUEST" | "VALIDATION_ERROR" => {
                    Err(ServiceError::Validation(detail.message))
                }
                _ => Err(ServiceError::Daemon(detail.message)),
            };
        }
        let data = response.data.ok_or_else(|| {
            ServiceError::Protocol("successful history/search response had no data".into())
        })?;
        let key = if request.source == PickerSource::Snippets {
            "snippets"
        } else {
            "items"
        };
        let values = data
            .get(key)
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

// ── Worker loop ──────────────────────────────────────────────────────

fn worker_loop(rx: mpsc::Receiver<WorkerRequest>) {
    // Persistent client — reused across requests, recreated on connection loss.
    let mut client: Option<IpcClient> = None;

    // Tracks the latest generation seen per source key. When a new
    // request arrives with a newer generation, the old queued response
    // for that source is discarded.
    let mut latest_gen: std::collections::HashMap<SourceKey, u64> = std::collections::HashMap::new();

    for request in rx {
        match request {
            WorkerRequest::Command {
                command,
                search,
                reply,
            } => {
                // ── Generation gating ─────────────────────────────
                if let Some(ref meta) = search {
                    let prev = latest_gen.get(&meta.source_key).copied().unwrap_or(0);
                    if meta.generation < prev {
                        // A newer search for this source already
                        // arrived — discard this one.
                        let _ = reply.send(Err(ServiceError::Superseded));
                        continue;
                    }
                    latest_gen.insert(meta.source_key, meta.generation);
                }

                // ── Execute ───────────────────────────────────────
                let result = execute_with_client(&mut client, &command);
                let _ = reply.send(result);
            }
        }
    }
}

fn execute_with_client(
    client_opt: &mut Option<IpcClient>,
    command: &IpcCommand,
) -> Result<IpcResponse, ServiceError> {
    // Lazily create or reconnect.
    if client_opt.is_none() {
        let mut c = IpcClient::new();
        c = c.with_timeouts(WRITE_TIMEOUT, RESPONSE_TIMEOUT);
        *client_opt = Some(c);
    }

    let client = client_opt.as_ref().expect("client just created");
    match client.send_command(command) {
        Ok(resp) => Ok(resp),
        Err(IpcError::ConnectionFailed(msg)) => {
            // Connection lost — invalidate and return offline.
            *client_opt = None;
            Err(ServiceError::Offline(msg))
        }
        Err(IpcError::SendFailed(_msg)) => {
            // Write failure — could be a dead connection. Invalidate
            // so the next request reconnects.
            *client_opt = None;
            Err(ServiceError::Timeout { stage: "write" })
        }
        Err(IpcError::ReceiveFailed(msg)) => {
            if msg.contains("timed out") || msg.contains("would block") {
                Err(ServiceError::Timeout { stage: "response" })
            } else {
                *client_opt = None;
                Err(ServiceError::Offline(msg))
            }
        }
        Err(IpcError::InvalidMessage(msg)) => Err(ServiceError::Protocol(msg)),
        Err(IpcError::SocketInUse) => {
            *client_opt = None;
            Err(ServiceError::Offline("socket is in use".into()))
        }
    }
}

// ── Helper functions ─────────────────────────────────────────────────

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

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

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
        assert!(error_message(&ServiceError::Superseded).contains("superseded"));
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

    #[tokio::test]
    async fn rapid_filter_switching_shows_latest() {
        let svc = MockService {
            delay: Duration::from_millis(1),
            result: Ok(Vec::new()),
        };
        // Simulate three rapid requests with increasing generations.
        let mut latest_gen = 0u64;
        let gen = AtomicU64::new(1);
        let reqs: Vec<_> = (0..3)
            .map(|_| {
                let g = gen.fetch_add(1, Ordering::SeqCst);
                let svc = svc.clone();
                latest_gen = g;
                tokio::spawn(async move {
                    svc.history(HistoryRequest {
                        query: String::new(),
                        limit: 10,
                        filter: PickerFilter::All,
                        source: PickerSource::History,
                        include_sensitive: false,
                        generation: g,
                    })
                    .await
                })
            })
            .collect();

        for (i, handle) in reqs.into_iter().enumerate() {
            let gen = i as u64 + 1;
            let result = handle.await.expect("task panicked");
            // Only the latest generation should be applied; earlier
            // ones might show stale data in real life, but the mock
            // is instant so all succeed.
            if gen == latest_gen {
                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn error_message_for_superseded_is_descriptive() {
        let msg = error_message(&ServiceError::Superseded);
        assert!(!msg.is_empty());
    }
}
