# MCP Integration

Author Clipboard includes a local stdio MCP server. It connects to the running
clipboard daemon over the same private Unix socket used by the UI and CLI; it
does not open an HTTP listener or send clipboard history over the network.

## Start the server

Build the workspace, start the daemon, then configure your MCP client to launch:

```bash
author-clipboard-mcp --transport stdio
```

The server process must run as the same user as the daemon so it can access the
private IPC socket.

## Codex configuration

Add the local server to your Codex MCP configuration using the executable and
its stdio argument:

```toml
[mcp_servers.author-clipboard]
command = "author-clipboard-mcp"
args = ["--transport", "stdio"]
```

Restart Codex after changing the configuration.

## Claude Desktop configuration

Add this entry beneath `mcpServers` in Claude Desktop's configuration:

```json
{
  "mcpServers": {
    "author-clipboard": {
      "command": "author-clipboard-mcp",
      "args": ["--transport", "stdio"]
    }
  }
}
```

## Privacy boundary

- Search results, resources, prompts, and default `clipboard.get` calls pass
  through an MCP-owned redaction boundary regardless of UI preview settings.
- Sensitive values are masked; encrypted ciphertext is never returned as item
  content.
- Copying a sensitive item requires `confirm_sensitive=true` on that individual
  `clipboard.copy` request.
- `copy_redacted` remains available without sensitive confirmation.
- `clipboard.delete` requires `confirm=true`.
- A full sensitive `clipboard.get` requires both `include_content=true` and
  `confirm_sensitive=true` on that request.
- Confirmation is per request and is never cached by the server.
- The server currently supports stdio only.

Treat the MCP client as software with access to your local clipboard metadata.
Review its configuration and prompts before granting tool calls.

The client can request recent item metadata, redacted content, pins, snippets,
and statistics. With explicit confirmation it can copy or request sensitive
content; with destructive confirmation it can delete items or snippets. The MCP
server and daemon remain local processes, but the MCP client may send tool output
elsewhere according to that client's own privacy policy.

## Useful prompts

- “Find the stack trace I copied recently.”
- “Locate my last copied JSON payload.”
- “List recent copied notes without exposing sensitive values.”
- “Create a snippet from my last copied shell command.”
- “Copy item 42 in redacted form.”

Sensitive content should only be copied after an explicit user decision; do not
set confirmation flags globally in an MCP client wrapper.
