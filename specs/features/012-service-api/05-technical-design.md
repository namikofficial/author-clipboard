# Technical Design: Service API

> Implementation approach and technical decisions for normalizing the service API.

---

## Overview

The key change is that the CLI (and all other clients) will route all operations through the daemon's IPC interface instead of accessing the database directly. The daemon becomes the single authoritative service.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/shared/src/ipc.rs` | Add IpcRequest/IpcResponse envelopes, expand IpcCommand enum, add error codes |
| `crates/clipboard-daemon/src/main.rs` | Implement new IPC command handlers, add subscription system, remove direct DB access for query operations |
| `crates/ctl/src/main.rs` | Route all commands through IpcClient, add --json/--pretty flags, remove direct Database access |
| `crates/shared/src/db.rs` | No changes (daemon still uses it internally) |
| `crates/shared/src/config.rs` | Add min_api_version field |

---

## Implementation Details

### Module: ipc.rs (shared crate)

```rust
// New request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub version: String,
    pub cmd: String,
    pub args: serde_json::Value,
    pub request_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub version: String,
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<IpcErrorDetail>,
}

// Expand IpcCommand to cover all operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum IpcCommand {
    // Visibility
    Toggle,
    Show,
    Hide,
    ShowAt { x: i32, y: i32 },
    // Health
    Ping,
    Status,
    // Query
    History { limit: usize, offset: Option<usize>, filters: Option<FilterOptions> },
    GetItem { id: i64 },
    Search { query: String, limit: Option<usize>, filters: Option<FilterOptions> },
    GetStats,
    GetAuditLog { limit: Option<usize> },
    // Mutations
    Copy { id: i64, mode: CopyMode },
    Pin { id: i64 },
    Unpin { id: i64 },
    Delete { id: i64 },
    ClearUnpinned,
    ClearAll,
    Import { items: Vec<ClipboardItem> },
    // Snippets
    ListSnippets,
    UpsertSnippet { name: String, content: String },
    DeleteSnippet { id: i64 },
    // Config
    GetConfig,
    UpdateConfig { config: serde_json::Value },
    // Subscriptions
    Subscribe { events: Vec<SubscriptionEvent> },
    Unsubscribe { subscription_id: u64 },
}

// New error type
#[derive(Debug, Clone, thiserror::Error)]
pub enum IpcCommandError {
    #[error("daemon not running")]
    DaemonNotRunning,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("item not found")]
    ItemNotFound,
    #[error("sensitive content requires confirmation")]
    SensitiveContent,
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("unsupported version: {0}, minimum: {1}")]
    UnsupportedVersion(String, String),
    #[error("internal error: {0}")]
    InternalError(String),
}
```

### Module: daemon main.rs

The daemon's IPC server loop changes to:
1. Parse incoming JSON as `IpcRequest`
2. Validate version against `min_api_version`
3. Dispatch to appropriate handler based on `cmd`
4. Build `IpcResponse` with structured data or error
5. Send response back on the socket

```rust
// In the IPC accept loop:
fn handle_command(state: &mut DaemonState, request: IpcRequest) -> IpcResponse {
    // Version check
    if request.version < state.min_api_version {
        return IpcResponse::error(
            "UNSUPPORTED_VERSION",
            format!("minimum version is {}", state.min_api_version),
            Some(state.min_api_version.clone()),
        );
    }

    // Dispatch
    match request.cmd.as_str() {
        "Toggle" => handle_toggle(state),
        "Show" => handle_show(state),
        "Hide" => handle_hide(state),
        "ShowAt" => handle_show_at(state, &request.args),
        "Ping" => handle_ping(state),
        "Status" => handle_status(state),
        "History" => handle_history(state, &request.args),
        "GetItem" => handle_get_item(state, &request.args),
        "Search" => handle_search(state, &request.args),
        // ... all other commands
        _ => IpcResponse::error("UNKNOWN_COMMAND", format!("unknown command: {}", request.cmd), Some("1.0".into())),
    }
}
```

### Subscription System

For live updates, the daemon maintains a list of subscriptions and broadcasts events:

```rust
struct Subscription {
    id: u64,
    client_id: String,
    events: Vec<SubscriptionEvent>,
    sender: tokio::sync::mpsc::Sender<IpcResponse>,
}

struct DaemonState {
    // ... existing fields ...
    subscriptions: HashMap<u64, Subscription>,
    next_subscription_id: u64,
}

impl DaemonState {
    fn broadcast(&self, event: SubscriptionEvent, data: serde_json::Value) {
        for sub in self.subscriptions.values() {
            if sub.events.contains(&event) {
                let msg = IpcResponse {
                    version: "1.0".into(),
                    ok: true,
                    data: Some(serde_json::json!({
                        "type": "event",
                        "event": event,
                        "data": data
                    })),
                    error: None,
                };
                let _ = sub.sender.send(msg);
            }
        }
    }
}
```

### CLI Changes

The CLI changes from direct database access to IPC calls:

```rust
// Before:
fn do_history(count: usize) -> Result<()> {
    let config = Config::load();
    let db = Database::open(&config.db_path())?;
    let items = db.get_recent(count)?;
    // print human-readable
}

// After:
fn do_history(count: usize, json: bool) -> Result<()> {
    let client = IpcClient::new();
    let response = client.send(&IpcCommand::History {
        limit: count,
        offset: None,
        filters: None,
    })?;
    if json {
        println!("{}", serde_json::to_string_pretty(&response.data)?);
    } else {
        // format for humans
    }
}
```

---

## Security Considerations

- [ ] IPC socket permissions: directory 0700, socket 0600
- [ ] Daemon validates all incoming JSON before parsing
- [ ] Version field prevents downgrade attacks
- [ ] Sensitive content masking happens in daemon, not client
- [ ] No raw sensitive data in IPC responses (only masked previews)

---

## Error Handling

| Error Condition | Handling Strategy |
|-----------------|-------------------|
| Malformed JSON | Return INVALID_REQUEST with parse error details |
| Unknown command | Return UNKNOWN_COMMAND with min_version |
| Item not found | Return ITEM_NOT_FOUND |
| Daemon crash | Client returns "Daemon connection lost" |
| Subscription sender dropped | Clean up subscription on next broadcast |

---

## Performance Considerations

- **IPC latency target**: < 50ms for query operations
- **Subscription system**: Uses async channels, non-blocking broadcasts
- **Connection pooling**: Single socket per client, keep alive
- **Query pagination**: All list responses include total count and has_more

---

## Testing Strategy

1. Unit tests for IPC request/response serialization
2. Integration tests for CLI-to-daemon round-trip
3. Test that CLI falls back to clear error when daemon is down
4. Test subscription lifecycle (subscribe, events, unsubscribe)
5. Test version negotiation (old client, new daemon)

---

## Migration Strategy

### Phase 1: Dual-mode CLI
- CLI tries IPC first, falls back to direct DB with warning
- Log when falling back: "Warning: daemon not reachable, using direct database access"
- This allows gradual rollout without breaking existing workflows

### Phase 2: IPC-only CLI
- After 1 release cycle, make IPC required
- Remove direct DB access from CLI
- Update error message: "Daemon not running. Start with: systemctl --user start author-clipboard-daemon"

### Phase 3: MCP foundation
- Build MCP server on top of normalized IPC
- MCP uses same command set as CLI

---

**Last Updated**: Phase 15