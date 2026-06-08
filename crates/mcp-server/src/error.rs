use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("Daemon not running: {0}")]
    DaemonNotRunning(String),

    #[error("Item not found")]
    ItemNotFound,

    #[error("Sensitive content requires confirmation")]
    SensitiveConfirmation,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
