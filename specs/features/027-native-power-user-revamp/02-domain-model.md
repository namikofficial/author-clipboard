# Domain Model: Native Power-User Revamp

> Data and state model for the revamp. This file defines the product concepts
> that implementation tasks should map onto existing Rust types, database
> tables, IPC payloads, and UI state.

---

## Existing Concepts To Preserve

| Concept | Existing Surface | Notes |
|---------|------------------|-------|
| Clipboard item | `ClipboardItem` in `shared::types` | Preserve content, MIME, type, timestamp, pinned, starred, sensitive. |
| Sensitive preview | `redacted_preview` | Must remain safe by default. |
| Picker entry | `PickerEntry` in `shared::picker` | UI-facing row model. May be extended carefully. |
| Snippet | `Snippet` in `shared::types` | Adopt rendered-preview behavior from `026`. |
| Config | `Config` in `shared::config` | Add only durable user settings. |
| GSettings state | `ui_gtk::settings` | Window size, layout, last page, filter, sort. |

## New Or Refined Product Concepts

### ContentClass

`ContentClass` is a UI/search classification layered on top of
`ContentType`. It must not replace MIME/content storage.

```rust
pub enum ContentClass {
    Text,
    Code { language_hint: Option<String> },
    Url,
    Path,
    Command,
    Json,
    Sql,
    Html,
    Image,
    Files,
    Snippet,
    Secret,
    Unknown,
}
```

Derivation is best-effort. It may use MIME type, content type, first-line
patterns, URI parsing, JSON/SQL heuristics, and snippet source. It must never
mark a sensitive item as non-sensitive.

### ItemContext

Context metadata explains where an item came from and why it matters.

```rust
pub struct ItemContext {
    pub source_app: Option<String>,
    pub window_title: Option<String>,
    pub project_id: Option<String>,
    pub project_path: Option<String>,
    pub tags: Vec<String>,
}
```

Wayland does not expose reliable source-app metadata through
`wlr-data-control`. Populate these fields only from explicit integrations,
manual tagging, imported data, or future compositor-specific hooks.

### InspectorState

The inspector is the right-side preview/action surface.

```rust
pub struct InspectorState {
    pub selected_item_id: Option<i64>,
    pub preview_mode: PreviewMode,
    pub reveal_sensitive: bool,
    pub reveal_seconds_remaining: u8,
    pub action_feedback: Option<String>,
}

pub enum PreviewMode {
    Summary,
    Text,
    Code,
    HtmlSafe,
    Image,
    Files,
    SnippetRendered,
    SensitiveRedacted,
    Error,
}
```

### SavedFilter

Saved filters are named query/filter presets.

```rust
pub struct SavedFilter {
    pub id: String,
    pub name: String,
    pub query: String,
    pub filter: PickerFilter,
    pub source: PickerSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Examples: `Deploy commands`, `Links`, `Secrets`, `Prompt fragments`,
`Project paths`.

### Collection

Collections adopt the `015-collections` model.

```rust
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: usize,
}
```

Collections are flat for this spec. Nested collections are out of scope.

### ActionRail

The action rail is UI state, not persistent storage.

```rust
pub enum ItemAction {
    Copy,
    QuickPaste,
    PinToggle,
    StarToggle,
    Delete,
    AddToCollection,
    RevealSensitive,
    OpenContainingFile,
    CopyAsPlainText,
    CopyAsHtml,
}
```

Only actions that are valid for the selected item should be enabled.

## Database Additions

The implementation should reuse existing columns where present. Additions are
expected for collections and saved filters.

```sql
CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS collection_memberships (
    collection_id TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    added_at TEXT NOT NULL,
    PRIMARY KEY (collection_id, item_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS saved_filters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    query TEXT NOT NULL,
    filter TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_collection_memberships_collection
    ON collection_memberships(collection_id);

CREATE INDEX IF NOT EXISTS idx_collection_memberships_item
    ON collection_memberships(item_id);
```

## UI State Additions

```rust
pub struct PowerUserUiState {
    pub inspector: InspectorState,
    pub collections: Vec<Collection>,
    pub saved_filters: Vec<SavedFilter>,
    pub active_query: ParsedQuery,
    pub daemon_health: DaemonHealth,
    pub layout_mode: LayoutMode,
}

pub enum LayoutMode {
    Compact,
    Split,
    Manager,
}

pub struct DaemonHealth {
    pub running: bool,
    pub pid: Option<u32>,
    pub capture_supported: bool,
    pub quick_paste_backend: Option<String>,
    pub warnings: Vec<String>,
}
```

## Query Model

```rust
pub struct ParsedQuery {
    pub raw: String,
    pub text_terms: Vec<String>,
    pub exact_phrases: Vec<String>,
    pub type_filter: Option<ContentClass>,
    pub app_filter: Option<String>,
    pub project_filter: Option<String>,
    pub collection_filter: Option<String>,
    pub errors: Vec<QueryParseWarning>,
}
```

Parser errors are warnings. They should not make search fail.

## Security Invariants

- `Sensitive` classification always wins over display convenience.
- Raw sensitive content never appears in row titles, logs, status JSON, or bar
  tooltips by default.
- Export defaults redact encrypted/sensitive payloads unless the user passes an
  explicit unsafe flag.
- Preview reveal state is UI-local and time-limited.

---

**Last Updated**: 2026-06-19
