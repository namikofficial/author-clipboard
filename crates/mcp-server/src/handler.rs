use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use author_clipboard_shared::ipc::{IpcClient, IpcCommand, FilterOptions, CopyMode};
use mcp_spec::protocol::{JsonRpcRequest, JsonRpcResponse};
use mcp_server::BoxError;
use serde_json::Value;
use tower::Service;

pub struct ClipboardService {
    client: IpcClient,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            client: IpcClient::new(),
        }
    }
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

impl Service<JsonRpcRequest> for ClipboardService {
    type Response = JsonRpcResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: JsonRpcRequest) -> Self::Future {
        let client = self.client.clone();
        let method = request.method.clone();
        let params = request.params;
        let id = request.id;

        Box::pin(async move {
            let result = handle_method(&client, &method, params).await;
            Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            })
        })
    }
}

async fn handle_method(
    client: &IpcClient,
    method: &str,
    params: Option<Value>,
) -> Value {
    match method {
        "initialize" => serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "author-clipboard",
                "version": "0.1.0"
            },
            "instructions": "Use clipboard.search to find items, clipboard.get to retrieve details. Sensitive items require confirm_sensitive=true on copy operations."
        }),

        "tools/list" => {
            let tools = vec![
                serde_json::json!({
                    "name": "clipboard.search",
                    "description": "Search clipboard history with filters",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {"type": "string"},
                            "limit": {"type": "number"},
                            "offset": {"type": "number"},
                            "content_type": {"type": "array", "items": {"type": "string"}},
                            "pinned": {"type": "boolean"},
                            "sensitive": {"type": "boolean"}
                        },
                        "required": ["query"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.get",
                    "description": "Get a clipboard item by ID",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "number"},
                            "include_content": {"type": "boolean"}
                        },
                        "required": ["id"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.copy",
                    "description": "Copy a clipboard item to Wayland clipboard",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "number"},
                            "mode": {"type": "string", "enum": ["copy", "quick_paste", "copy_plain_text", "copy_redacted"]},
                            "confirm_sensitive": {"type": "boolean"}
                        },
                        "required": ["id"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.pin",
                    "description": "Pin a clipboard item",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "number"}
                        },
                        "required": ["id"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.unpin",
                    "description": "Unpin a clipboard item",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "number"}
                        },
                        "required": ["id"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.delete",
                    "description": "Delete a clipboard item",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "number"},
                            "confirm": {"type": "boolean"}
                        },
                        "required": ["id", "confirm"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.stats",
                    "description": "Get clipboard statistics",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.list_snippets",
                    "description": "List all snippets",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.upsert_snippet",
                    "description": "Create or update a snippet",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "content": {"type": "string"}
                        },
                        "required": ["name", "content"]
                    }
                }),
                serde_json::json!({
                    "name": "clipboard.delete_snippet",
                    "description": "Delete a snippet",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "number"},
                            "confirm": {"type": "boolean"}
                        },
                        "required": ["id", "confirm"]
                    }
                })
            ];
            serde_json::json!({"tools": tools})
        }

        "tools/call" => {
            let params = params.unwrap_or_default();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or_default();

            match tool_name {
                "clipboard.search" => {
                    let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
                    let limit = arguments.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
                    let offset = arguments.get("offset").and_then(|v| v.as_u64()).map(|v| v as usize);
                    let content_type = arguments.get("content_type").and_then(|v| {
                        v.as_array().map(|arr| {
                            arr.iter().filter_map(|s| s.as_str().map(String::from)).collect()
                        })
                    });
                    let pinned = arguments.get("pinned").and_then(|v| v.as_bool());
                    let sensitive = arguments.get("sensitive").and_then(|v| v.as_bool());

                    let filters = FilterOptions {
                        content_type,
                        pinned,
                        sensitive,
                        source_app: None,
                        age_min_seconds: None,
                        age_max_seconds: None,
                    };

                    match client.send_command(&IpcCommand::Search {
                        query: query.to_string(),
                        limit,
                        filters: Some(filters),
                    }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.get" => {
                    let id = arguments.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let include_content = arguments.get("include_content").and_then(|v| v.as_bool()).unwrap_or(false);

                    match client.send_command(&IpcCommand::GetItem { id }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.copy" => {
                    let id = arguments.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let mode_str = arguments.get("mode").and_then(|v| v.as_str()).unwrap_or("copy");
                    let mode = match mode_str {
                        "quick_paste" => CopyMode::QuickPaste,
                        "copy_plain_text" => CopyMode::CopyPlainText,
                        "copy_redacted" => CopyMode::CopyRedacted,
                        _ => CopyMode::Copy,
                    };

                    match client.send_command(&IpcCommand::Copy { id, mode }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.pin" => {
                    let id = arguments.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

                    match client.send_command(&IpcCommand::Pin { id }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.unpin" => {
                    let id = arguments.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

                    match client.send_command(&IpcCommand::Unpin { id }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.delete" => {
                    let id = arguments.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let confirm = arguments.get("confirm").and_then(|v| v.as_bool()).unwrap_or(false);

                    if !confirm {
                        return serde_json::json!({"isError": true, "content": [{"type": "text", "text": "confirm must be true"}]});
                    }

                    match client.send_command(&IpcCommand::Delete { id }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.stats" => {
                    match client.send_command(&IpcCommand::GetStats) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.list_snippets" => {
                    match client.send_command(&IpcCommand::ListSnippets) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.upsert_snippet" => {
                    let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");

                    match client.send_command(&IpcCommand::UpsertSnippet { name: name.to_string(), content: content.to_string() }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                "clipboard.delete_snippet" => {
                    let id = arguments.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let confirm = arguments.get("confirm").and_then(|v| v.as_bool()).unwrap_or(false);

                    if !confirm {
                        return serde_json::json!({"isError": true, "content": [{"type": "text", "text": "confirm must be true"}]});
                    }

                    match client.send_command(&IpcCommand::DeleteSnippet { id }) {
                        Ok(resp) => serde_json::json!({"content": [{"type": "text", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                    }
                }

                _ => serde_json::json!({"isError": true, "content": [{"type": "text", "text": format!("Unknown tool: {}", tool_name)}]})
            }
        }

        "resources/list" => {
            serde_json::json!({
                "resources": [
                    {"uri": "clipboard://recent", "name": "Recent Clipboard Items", "mimeType": "application/json"},
                    {"uri": "clipboard://pins", "name": "Pinned Items", "mimeType": "application/json"},
                    {"uri": "clipboard://snippets", "name": "Snippets", "mimeType": "application/json"},
                    {"uri": "clipboard://stats", "name": "Statistics", "mimeType": "application/json"}
                ]
            })
        }

        "resources/read" => {
            let uri = params.as_ref().and_then(|p| p.get("uri")).and_then(|v| v.as_str()).unwrap_or("");
            let uri_path = uri.strip_prefix("clipboard://").unwrap_or(uri);

            match uri_path {
                "recent" => {
                    match client.send_command(&IpcCommand::History { limit: 50, offset: Some(0), filters: None }) {
                        Ok(resp) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": e.to_string()}]})
                    }
                }
                "pins" => {
                    let filters = FilterOptions {
                        content_type: None,
                        pinned: Some(true),
                        sensitive: None,
                        source_app: None,
                        age_min_seconds: None,
                        age_max_seconds: None,
                    };
                    match client.send_command(&IpcCommand::Search { query: String::new(), limit: Some(100), filters: Some(filters) }) {
                        Ok(resp) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": e.to_string()}]})
                    }
                }
                "snippets" => {
                    match client.send_command(&IpcCommand::ListSnippets) {
                        Ok(resp) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": e.to_string()}]})
                    }
                }
                "stats" => {
                    match client.send_command(&IpcCommand::GetStats) {
                        Ok(resp) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                        Err(e) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": e.to_string()}]})
                    }
                }
                _ if uri_path.starts_with("item/") => {
                    if let Ok(id) = uri_path.strip_prefix("item/").unwrap_or("").parse::<i64>() {
                        match client.send_command(&IpcCommand::GetItem { id }) {
                            Ok(resp) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": serde_json::to_string(&resp.data).unwrap_or_default()}]}),
                            Err(e) => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": e.to_string()}]})
                        }
                    } else {
                        serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": "Invalid item ID"}]})
                    }
                }
                _ => serde_json::json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": "Unknown resource"}]})
            }
        }

        "prompts/list" => {
            serde_json::json!({
                "prompts": [
                    {"name": "clipboard:summarize_recent", "description": "Summarize recent clipboard items", "arguments": [{"name": "limit", "description": "Number of items", "required": false}]},
                    {"name": "clipboard:find_pattern", "description": "Find items matching a pattern", "arguments": [{"name": "pattern", "description": "Search pattern", "required": true}]}
                ]
            })
        }

        "prompts/get" => {
            let name = params.as_ref().and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.as_ref().and_then(|p| p.get("arguments")).cloned().unwrap_or_default();

            match name {
                "clipboard:summarize_recent" => {
                    let limit = arguments.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(10);
                    match client.send_command(&IpcCommand::History { limit, offset: Some(0), filters: None }) {
                        Ok(resp) => {
                            let items = resp.data.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default()).unwrap_or_default();
                            serde_json::json!({
                                "description": "Summarize recent clipboard items",
                                "messages": [{
                                    "role": "user",
                                    "content": {
                                        "type": "text",
                                        "text": format!("Summarize the following {} clipboard items:\n{}", limit, items)
                                    }
                                }]
                            })
                        }
                        Err(e) => serde_json::json!({"error": e.to_string()})
                    }
                }
                "clipboard:find_pattern" => {
                    let pattern = arguments.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
                    let filters = FilterOptions {
                        content_type: None,
                        pinned: None,
                        sensitive: None,
                        source_app: None,
                        age_min_seconds: None,
                        age_max_seconds: None,
                    };
                    match client.send_command(&IpcCommand::Search { query: pattern.to_string(), limit: Some(50), filters: Some(filters) }) {
                        Ok(resp) => {
                            let items = resp.data.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default()).unwrap_or_default();
                            serde_json::json!({
                                "description": format!("Find items matching: {}", pattern),
                                "messages": [{
                                    "role": "user",
                                    "content": {
                                        "type": "text",
                                        "text": format!("Search clipboard for '{}':\n{}", pattern, items)
                                    }
                                }]
                            })
                        }
                        Err(e) => serde_json::json!({"error": e.to_string()})
                    }
                }
                _ => serde_json::json!({"error": format!("Unknown prompt: {}", name)})
            }
        }

        _ => serde_json::json!({"error": format!("Unknown method: {}", method)})
    }
}