# API Contract: Native Power-User Revamp

> IPC, CLI, and JSON contract changes required for power-user UI actions,
> organization, search, and integrations.

---

## Contract Principles

- UI actions must have CLI/IPC equivalents.
- JSON output must be stable enough for Waybar, Wayle, scripts, and editor
  integrations.
- Sensitive payloads are redacted by default at every boundary.
- IPC errors use explicit codes, not only human strings.
- Commands that mutate storage must be idempotent where practical.

## IPC Commands

Extend `IpcCommand` with the following commands if not already present:

```rust
pub enum IpcCommand {
    // Existing commands omitted.

    GetItem { id: i64, include_sensitive: bool },
    DeleteItem { id: i64 },
    PinItem { id: i64 },
    UnpinItem { id: i64 },
    StarItem { id: i64 },
    UnstarItem { id: i64 },

    ListCollections,
    CreateCollection { name: String, description: Option<String> },
    RenameCollection { id: String, name: String },
    DeleteCollection { id: String },
    AddToCollection { collection_id: String, item_id: i64 },
    RemoveFromCollection { collection_id: String, item_id: i64 },
    ListCollectionItems { collection_id: String, limit: usize, offset: usize },

    ListSavedFilters,
    CreateSavedFilter { name: String, query: String, filter: String, source: String },
    DeleteSavedFilter { id: String },

    Search { query: String, filter: String, source: String, limit: usize, offset: usize },
    Health,
}
```

## IPC Response Shapes

### `Health`

```json
{
  "running": true,
  "daemon_pid": 1234,
  "capture_supported": true,
  "compositor": "hyprland",
  "clipboard_backend": "wlr-data-control",
  "quick_paste_backend": "wtype",
  "warnings": []
}
```

### `Search`

```json
{
  "items": [
    {
      "id": 42,
      "title": "git status --short",
      "subtitle": "command · 2m ago · text/plain · 18 chars",
      "content_type": "text",
      "content_class": "command",
      "mime_type": "text/plain",
      "sensitive": false,
      "pinned": false,
      "starred": true,
      "collections": ["deploy commands"],
      "preview": "git status --short"
    }
  ],
  "warnings": []
}
```

`preview` must be redacted for sensitive entries unless the request explicitly
allows sensitive content and the daemon policy permits it.

## CLI Commands

Add or normalize the following commands:

```text
author-clipboard-ctl item get <id> [--json] [--include-sensitive]
author-clipboard-ctl item delete <id>
author-clipboard-ctl item pin <id>
author-clipboard-ctl item unpin <id>
author-clipboard-ctl item star <id>
author-clipboard-ctl item unstar <id>

author-clipboard-ctl collection list [--json]
author-clipboard-ctl collection create <name>
author-clipboard-ctl collection rename <id> <name>
author-clipboard-ctl collection delete <id>
author-clipboard-ctl collection add <collection> <item-id>
author-clipboard-ctl collection remove <collection> <item-id>

author-clipboard-ctl filter list [--json]
author-clipboard-ctl filter save <name> --query <query> [--filter <filter>] [--source <source>]
author-clipboard-ctl filter delete <id>

author-clipboard-ctl search <query> [--json] [--limit N]
author-clipboard-ctl health --json
```

## Query Language

Supported user-facing tokens:

| Token | Meaning |
|-------|---------|
| `type:text` | Plain text. |
| `type:code` | Code-like content. |
| `type:command` | Shell command-like content. |
| `type:url` | URLs. |
| `type:path` | Local paths. |
| `type:image` | Images. |
| `type:file` | File URI lists. |
| `type:secret` | Sensitive items. |
| `type:snippet` | Snippet/template entries. |
| `app:<name>` | Source app, if available. |
| `project:<name>` | Project tag/context, if available. |
| `collection:<name>` | Collection membership. |
| `"exact phrase"` | Exact phrase search. |

Invalid tokens become warnings and fall back to plain text search.

## Desktop / Install Contract

The install path must provide:

- `author-clipboard-daemon`
- `author-clipboard`
- `author-clipboard-ctl`
- `author-clipboard-hypr-picker`
- `com.namikofficial.author-clipboard.desktop`
- `com.namikofficial.author-clipboard.hypr-picker.desktop`
- `com.namikofficial.author-clipboard.gschema.xml` compiled into the user or
  system schema directory
- `author-clipboard-daemon.service`
- icon assets
- optional Waybar module assets

## Backward Compatibility

- Existing `author-clipboard-ctl picker` must keep working.
- Existing `author-clipboard-hypr-picker` flags must keep working.
- Existing `status --json` consumers must not break; add fields but do not
  remove or rename current fields without a migration window.
- Existing database migrations must be additive.

---

**Last Updated**: 2026-06-19
