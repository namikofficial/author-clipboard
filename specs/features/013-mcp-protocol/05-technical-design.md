# Technical Design: MCP Protocol Server

> Implementation approach for the MCP server using the MCP Rust SDK.

---

## Overview

The MCP server will be implemented using the `mcp` crate (official MCP Rust SDK). It will:
1. Use stdio transport by default (Codex, local OpenCode)
2. Optionally support HTTP Streamable transport (remote OpenCode)
3. Sit above the daemon's IPC interface (Feature 012)
4. Use the same policy engine as CLI/applet for consistency

---

## Affected Files

| File | Change |
|------|--------|
| `crates/mcp-server/` | New crate |
| `crates/mcp-server/Cargo.toml` | Dependencies: mcp, tokio, serde_json, tracing |
| `crates/mcp-server/src/main.rs` | Entry point, transport setup |
| `crates/mcp-server/src/server.rs` | MCP server implementation |
| `crates/mcp-server/src/tools.rs` | Tool definitions |
| `crates/mcp-server/src/resources.rs` | Resource definitions |
| `crates/mcp-server/src/prompts.rs` | Prompt templates |
| `crates/mcp-server/src/transport.rs` | stdio/HTTP transport selection |
| `crates/mcp-server/src/error.rs` | MCP-specific error types |

---

## Crate Structure

### Cargo.toml

```toml
[package]
name = "author-clipboard-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
mcp = "0.1"           # Official MCP SDK
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

[features]
default = ["stdio"]
stdio = []
http = ["axum"]       # Optional HTTP transport
```

---

## Implementation Details

### Module: main.rs

```rust
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Transport to use: stdio or http
    #[arg(long, default_value = "stdio")]
    transport: String,
    /// HTTP port (only for http transport)
    #[arg(long, default_value = "8765")]
    port: u16,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.transport.as_str() {
        "stdio" => run_stdio_server()?,
        "http" => run_http_server(args.port)?,
        _ => anyhow::bail!("Unknown transport: {}", args.transport),
    }

    Ok(())
}
```

### Module: server.rs

```rust
use mcp::server::{Server, Handler};
use mcp::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ClipboardServer {
    inner: Arc<Mutex<ServerState>>,
}

pub struct ServerState {
    daemon_client: DaemonIpcClient,
    config: McpConfig,
}

impl ClipboardServer {
    pub fn new(config: McpConfig) -> Self {
        let daemon_client = DaemonIpcClient::new();
        Self {
            inner: Arc::new(Mutex::new(ServerState {
                daemon_client,
                config,
            })),
        }
    }
}

impl Handler for ClipboardServer {
    async fn handle_request(&self, request: mcp::types::Request) -> mcp::types::Response {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            "resources/list" => self.handle_list_resources(request).await,
            "resources/read" => self.handle_read_resource(request).await,
            "prompts/list" => self.handle_list_prompts(request).await,
            "prompts/get" => self.handle_get_prompt(request).await,
            _ => Response::error("Method not found", ErrorCode::MethodNotFound),
        }
    }

    async fn handle_list_tools(&self, _request: Request) -> Response {
        let tools = vec![
            Tool {
                name: "clipboard.search".into(),
                description: "Search clipboard history".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "number" },
                        "content_type": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["query"]
                }),
            },
            // ... all other tools
        ];

        Response::success(mcp::types::ListToolsResult { tools })
    }

    async fn handle_call_tool(&self, request: Request) -> Response {
        let params: CallToolParams = serde_json::from_value(request.params)
            .map_err(|e| Response::error(&e.to_string(), ErrorCode::InvalidParams))?;

        match params.name.as_str() {
            "clipboard.search" => self.tool_search(params.arguments).await,
            "clipboard.copy" => self.tool_copy(params.arguments).await,
            // ... all other tools
            _ => Response::error("Tool not found", ErrorCode::ToolNotFound),
        }
    }
}
```

### Module: tools.rs

```rust
use serde::{Deserialize, Serialize};
use crate::error::McpError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub content_type: Option<Vec<String>>,
    pub pinned: Option<bool>,
    pub sensitive: Option<bool>,
    pub source_app: Option<String>,
    pub age_max_seconds: Option<u64>,
}

pub async fn search(state: &ServerState, input: SearchInput) -> Result<serde_json::Value, McpError> {
    let response = state.daemon_client
        .send(IpcCommand::Search {
            query: input.query,
            limit: input.limit.unwrap_or(50),
            filters: Some(FilterOptions {
                content_type: input.content_type.map(|v| v.iter().map(|s| s.parse().unwrap()).collect()),
                pinned: input.pinned,
                sensitive: input.sensitive,
                source_app: input.source_app,
                age_min_seconds: None,
                age_max_seconds: input.age_max_seconds,
                search_query: None,
            }),
        })
        .await
        .map_err(|e| McpError::DaemonError(e.to_string()))?;

    match response.data {
        Some(data) => Ok(data),
        None => Err(McpError::DaemonError(response.error.map(|e| e.message).unwrap_or_default())),
    }
}

// Similar implementations for all other tools...
```

### Module: transport.rs

```rust
use anyhow::Result;

pub async fn run_stdio_server() -> Result<()> {
    let server = ClipboardServer::new(McpConfig::load()?);
    mcp::transport::stdio::run(server).await?;
    Ok(())
}

pub async fn run_http_server(port: u16) -> Result<()> {
    let server = ClipboardServer::new(McpConfig::load()?);
    let addr = format!("127.0.0.1:{}", port);
    mcp::transport::http::run(server, &addr).await?;
    Ok(())
}
```

---

## Error Handling

| Error Type | MCP Error Code | Description |
|------------|---------------|-------------|
| DaemonNotRunning | -32603 (Internal error) | Daemon not running |
| ItemNotFound | -32602 (Invalid params) | Item ID doesn't exist |
| SensitiveConfirmation | -32602 (Invalid params) | confirm_sensitive required |
| ConfirmationRequired | -32602 (Invalid params) | confirm=true required |
| InvalidRequest | -32600 (Invalid request) | Malformed request |

---

## Security Considerations

- [ ] MCP server validates all tool inputs before IPC calls
- [ ] Sensitive content masking happens in daemon, not MCP server
- [ ] No raw sensitive data in tool responses (only masked previews)
- [ ] HTTP transport binds only to localhost
- [ ] HTTP transport rejects requests with invalid Origin headers
- [ ] Token cost optimization: use list results over full content
- [ ] Server instructions warn about sensitive data handling

---

## Performance Considerations

- **Tool response latency target**: < 100ms
- **Connection to daemon**: Keep IPC client pooled
- **Token cost**: Always use masked previews in list results
- **Memory**: Reuse buffers for serialization

---

## Testing Strategy

1. Unit tests for each tool handler
2. Integration tests with mock daemon IPC
3. Test sensitive item handling (masked vs full content)
4. Test confirmation requirements for destructive tools
5. Test pagination for large result sets
6. Test resource URI parsing

---

## Migration Strategy

### Phase 1: stdio transport
- Implement MCP server with stdio transport
- Test with Codex local configuration
- Test with OpenCode local configuration

### Phase 2: HTTP transport
- Add optional HTTP transport using axum
- Test with OpenCode remote configuration
- Document security requirements for remote

### Phase 3: Production hardening
- Add authentication for HTTP transport
- Add rate limiting
- Add audit logging for MCP tool invocations

---

**Last Updated**: Phase 15