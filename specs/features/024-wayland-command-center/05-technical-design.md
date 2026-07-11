# Technical Design: Wayland Clipboard Command Center

## Design Goal

Build a polished command-center layer on top of the existing Author Clipboard
daemon, storage, IPC, UI, CLI, and MCP foundations without fragmenting state,
privacy policy, or content interpretation.

The critical rule:

> Data, privacy, and transformations live in shared/testable layers. GTK pages,
> CLI commands, and MCP handlers are presentation/adaptor surfaces.

## Current Architectural Direction

```
Wayland clipboard
      │
      ▼
clipboard-daemon
      │
      ├─ capture/rules/privacy decision
      ├─ storage write/read
      ├─ event/change notification
      ▼
shared IPC boundary
      │
      ├─ UI controller/model
      ├─ CLI commands
      └─ MCP server
```

## Target Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         User surfaces                            │
├────────────────────┬──────────────────────┬──────────────────────┤
│ GTK4 popup/manager │ CLI / ctl             │ MCP stdio server      │
│ presentation only  │ setup/automation      │ local AI boundary     │
└─────────┬──────────┴──────────┬───────────┴──────────┬───────────┘
          │                     │                      │
          ▼                     ▼                      ▼
┌──────────────────────────────────────────────────────────────────┐
│                  Shared application contracts                     │
├──────────────────────────────────────────────────────────────────┤
│ ItemViewModel / ContentPresentation / PrivacyPolicy / Transform   │
│ Snippet expansion / Rules / Diagnostics / IPC DTOs                │
└─────────────────────────────┬────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│                         Daemon + storage                          │
├──────────────────────────────────────────────────────────────────┤
│ Wayland capture / app rules / sensitive detection / encryption     │
│ SQLite / image files / thumbnails / import-export / events         │
└──────────────────────────────────────────────────────────────────┘
```

## Crate Responsibilities

### `crates/clipboard-daemon`

Owns:

- Wayland capture.
- Source application/window metadata collection where available.
- Capture rule evaluation.
- Ignore-next-copy state consumption.
- Sensitive detection before storage.
- Encryption before persistence.
- Storage writes.
- Mutation handling.
- Emitting capture/change events or exposing change snapshots.
- Never logs raw sensitive content.

Must not own:

- GTK presentation formatting.
- MCP-specific redaction behavior.
- CLI-specific output formatting.

### `crates/shared`

Owns reusable, testable contracts:

- IPC command/response DTOs.
- Item view models.
- Content classification.
- Privacy policy.
- Sensitive redaction helpers.
- Transformations.
- Snippet expansion.
- Capture rules.
- Diagnostics primitives.
- Config migrations.
- Database migrations.
- Safe import/export logic.

This crate is the center of consistency. If UI, CLI, and MCP need the same
answer, it belongs here.

### `crates/ui-gtk`

Owns presentation:

- Popup shell.
- Manager workspace.
- Model-backed list.
- Preview pane.
- Action bar.
- Shortcut overlay.
- Settings/rules pages.
- Toasts.
- Accessibility labels.
- CSS/design tokens.

Must not:

- Open the clipboard database from result page widgets.
- Implement independent privacy redaction.
- Re-parse content in a different way from shared code.
- Invent MCP or CLI behavior.

### `crates/ctl`

Owns command-line user workflows:

- Status.
- History/search display.
- Copy/clear/import/export.
- Doctor.
- Hyprland config generation.
- Transform commands.
- Ignore-next-copy command.

Must use shared diagnostics, privacy, transform, and IPC contracts.

### `crates/mcp-server`

Owns MCP protocol adaptation:

- Tool schemas.
- Resource schemas.
- Prompt templates.
- Confirmation validation at server boundary.
- Mapping shared errors into MCP-safe errors.

Must call shared privacy policy before any output.

## Data Flow: Capture

```
Wayland offers clipboard content
  → daemon captures content + MIME + source metadata
  → ignore-next-copy check
  → app rules check
  → sensitive detection
  → content hash/dedup
  → optional sensitive encryption
  → storage write
  → change event / refresh signal
```

### Capture Decision Order

1. Validate MIME/size against config.
2. Check ignore-next-copy.
3. Evaluate app rules:
   - ignore/deny,
   - force redact,
   - tag,
   - TTL override.
4. Run sensitive detection.
5. Apply storage privacy policy.
6. Store item.
7. Emit change notification.

## Data Flow: UI

```
popup opens
  → UI controller requests item snapshot through IPC
  → shared maps stored item to ItemViewModel
  → AppState stores visible model
  → list model renders ItemViewModel rows
  → selected ID drives preview and actions
  → action dispatches IPC command
  → daemon mutates
  → change event refreshes model
```

## UI Source of Truth

The UI must have one authoritative item model.

### Proposed State Shape

```rust
pub struct AppState {
    pub mode: AppMode,
    pub active_page: PageId,
    pub query: String,
    pub filter: PickerFilter,
    pub sort: SortOrder,
    pub selected_id: Option<i64>,
    pub items: Vec<ItemViewModel>,
    pub status: RuntimeStatus,
    pub incognito: bool,
    pub show_redacted_until: Option<DateTime<Utc>>,
}
```

### Selection Rules

- Selection is by item ID, not row index.
- Row index is derived from visible model only for navigation.
- Unknown ID clears selection or selects nearest visible item based on context.
- Refresh preserves selected ID when present.
- Delete moves selection to next item, then previous item, then none.

### Action Rules

Every action receives an item ID from the current `AppState.items` model. The
UI must not reconstruct IDs from stale row side tables unless the table is
generated from the same authoritative model during binding.

## List Rendering Strategy

Preferred implementation:

- Use GTK model/list factory types compatible with current dependencies.
- Bind `ItemViewModelObject` or equivalent object rows.
- Update model content without removing every visible child.
- Maintain selected ID separately from row object identity.

Fallback implementation:

- Keep a keyed row cache by item ID.
- Rebind changed rows.
- Remove only rows no longer present.
- Insert/move rows deterministically.
- Preserve focus/selection.

### Required Measurement

Add a synthetic 1,000-entry harness. Record:

- snapshot load time,
- first render time,
- refresh after one item insert,
- refresh after one item delete,
- refresh after query change.

Do not claim performance targets until measured.

## IPC and Events

### Constraints

- Existing clients should not break.
- Additive fields must use serde defaults.
- New commands require updates in all match arms and constructors.
- Protocol changes need focused tests.

### Candidate Commands

```rust
pub enum IpcCommand {
    History {
        limit: usize,
        offset: Option<usize>,
        filters: Option<FilterOptions>,
    },
    Search {
        query: String,
        limit: Option<usize>,
        filters: Option<FilterOptions>,
    },
    Snapshot {
        query: Option<String>,
        filter: PickerFilter,
        limit: usize,
        sort: SortOrder,
        include_sensitive: bool,
    },
    Watch {
        since_revision: Option<u64>,
    },
    Status,
    Copy {
        id: i64,
        mode: CopyMode,
        mime: Option<String>,
        confirmation: Option<Confirmation>,
    },
    Transform {
        id: i64,
        transform: TransformKind,
        confirmation: Option<Confirmation>,
    },
    IgnoreNextCopy {
        ttl_seconds: Option<u64>,
    },
}
```

### Candidate Event Shape

```rust
pub struct ClipboardEvent {
    pub revision: u64,
    pub kind: ClipboardEventKind,
    pub affected_id: Option<i64>,
    pub safe_message: Option<String>,
}
```

Event kinds:

- `Captured`
- `Deleted`
- `Pinned`
- `Unpinned`
- `Starred`
- `Unstarred`
- `Cleared`
- `SnippetChanged`
- `ConfigChanged`
- `IncognitoChanged`
- `DaemonStatusChanged`

If long-lived watch/subscription is not stable enough for the first slice,
implement explicit refresh triggered by window open and mutation responses, not
arbitrary fixed timeouts.

## Item View Model

Shared code should produce a UI/MCP-safe item representation.

```rust
pub struct ItemViewModel {
    pub id: i64,
    pub content_hash: u64,
    pub presentation: ContentPresentation,
    pub safe_preview: String,
    pub source_app: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub relative_time_label: String,
    pub pinned: bool,
    pub starred: bool,
    pub sensitive: bool,
    pub encrypted: bool,
    pub tags: Vec<String>,
    pub actions: Vec<ItemAction>,
}
```

The exact Rust shape can differ, but these concepts must exist.

## Content Presentation Model

`ContentPresentation` is pure and local-only.

```rust
pub enum ContentPresentation {
    Text {
        preview: String,
    },
    Url {
        preview: String,
        domain: String,
        normalized_url: String,
    },
    Color {
        hex: String,
        rgba: Option<(u8, u8, u8, f32)>,
        original: String,
    },
    Code {
        preview: String,
        language_hint: Option<String>,
    },
    Json {
        preview: String,
        valid: bool,
        root_kind: JsonRootKind,
    },
    Html {
        text_preview: String,
    },
    Image {
        thumbnail_path: Option<PathBuf>,
        width: Option<u32>,
        height: Option<u32>,
        mime: String,
    },
    File {
        name: String,
        path_hint: String,
        mime: Option<String>,
        exists: Option<bool>,
    },
    Secret {
        redacted_preview: String,
        kind: SecretKind,
    },
    Unknown {
        preview: String,
    },
}
```

## Classification Algorithm

Classification order:

1. If item is sensitive or policy says forced redaction → `Secret`.
2. If MIME is image → `Image`.
3. If MIME is `text/uri-list` or content parses as file URI list → `File`.
4. If MIME is HTML → `Html`.
5. If text parses as URL → `Url`.
6. If text parses as color → `Color`.
7. If text parses as JSON → `Json`.
8. If text looks like code/command/log → `Code`.
9. If text is present → `Text`.
10. Otherwise → `Unknown`.

### Parser Rules

- Every parser has input size limits.
- Parsers return safe fallback, not panics.
- No parser performs network I/O.
- URL parsing strips credentials from display.
- File preview avoids exposing more path than necessary in compact UI.
- JSON parser should cap pretty-preview length.
- Code detector should be heuristic, not a dependency-heavy syntax highlighter.

## Privacy Policy

Centralize privacy in shared code.

### Proposed API

```rust
pub struct PrivacyPolicy {
    pub include_sensitive: bool,
    pub confirmation: Option<Confirmation>,
    pub output_surface: OutputSurface,
}

pub enum OutputSurface {
    UiRow,
    UiPreview,
    CliHuman,
    CliJson,
    McpTool,
    McpResource,
    Export,
    Log,
}

pub enum SensitiveDecision {
    AllowFull,
    Redact,
    Refuse { reason: SafeError },
}
```

### Policy Rules

- Logs always redact.
- UI rows always redact.
- UI preview redacts unless local reveal is active.
- CLI JSON redacts by default.
- MCP redacts by default.
- Export redacts by default.
- Full sensitive content requires explicit confirmation on eligible surfaces.
- Confirmation is request-scoped and never persisted globally.

## Sensitive Encryption Migration

### Goal

New installations should encrypt sensitive items by default while preserving
existing explicit user settings.

### Config Strategy

Add a config version field if not present:

```json
{
  "config_version": 2,
  "encrypt_sensitive": true
}
```

Migration behavior:

| Existing state | Result |
|---|---|
| No config file | create default with `encrypt_sensitive=true` |
| Config has explicit `encrypt_sensitive=true` | preserve true |
| Config has explicit `encrypt_sensitive=false` | preserve false |
| Config lacks key but has older version | use decision documented in `09-decisions.md` |
| Invalid config | do not overwrite silently; report doctor error |

### Database Strategy

- Do not rewrite every existing row during config migration unless explicitly
  required and tested.
- New sensitive rows follow new policy.
- Existing rows remain readable.
- If re-encryption is implemented, it must be a separate explicit migration
  task with backup guidance.

## App Rules

### Rule Model

```rust
pub struct CaptureRule {
    pub id: String,
    pub enabled: bool,
    pub name: String,
    pub match_app: Option<String>,
    pub match_window: Option<String>,
    pub match_mime: Option<String>,
    pub action: CaptureRuleAction,
    pub tag: Option<String>,
    pub ttl_seconds: Option<u64>,
}

pub enum CaptureRuleAction {
    Ignore,
    Redact,
    Tag,
    SetTtl,
}
```

### Precedence

Default recommendation:

1. Disabled rules ignored.
2. Ignore rules win first.
3. Redact rules apply next.
4. Tag/TTL rules merge after redaction.
5. If multiple same-level rules match, earlier config order wins unless
   documented otherwise.

This must be finalized in `09-decisions.md`.

## Ignore-Next-Copy

State belongs in daemon runtime, not only UI.

```rust
pub struct IgnoreNextCopyState {
    pub armed_at: DateTime<Utc>,
    pub ttl_seconds: u64,
    pub consumed: bool,
}
```

Rules:

- Consumes exactly one eligible capture.
- Expires after TTL.
- Does not skip internal app copies unless explicitly defined.
- Produces safe feedback event.

## Transformations

### Transform Kinds

```rust
pub enum TransformKind {
    PlainText,
    MarkdownLink,
    FencedCode { language_hint: Option<String> },
    Quote,
    JsonPretty,
    JsonMinified,
    Redacted,
}
```

### Rules

- Transform functions are pure.
- Invalid input returns safe error.
- Original content is not mutated unless user explicitly copies/saves result.
- Sensitive content requires confirmation before transforms that expose raw
  value.
- `Redacted` transform is always allowed.

## Snippet Variables

### Variables

- `{date}`: local date using configured/default format.
- `{time}`: local time using configured/default format.
- `{clipboard}`: current clipboard item content.
- `{selection}`: active selected text if supported by future integration;
  otherwise empty or error based on mode.

### Escaping

- `{{date}}` renders literal `{date}`.
- Unknown variables return a safe validation error unless a compatibility mode
  is explicitly chosen.
- Sensitive variable sources require confirmation.

## MCP Safety

### Boundary

MCP server must validate output before serializing JSON.

### Tool Categories

| Tool type | Default sensitive behavior |
|---|---|
| search/list/resources | redacted previews |
| get full item | refuse without confirmation |
| copy full item | refuse without confirmation |
| transform raw sensitive item | refuse without confirmation |
| delete/clear | require destructive confirmation |
| stats | safe aggregate only |

### Error Shape

Use machine-readable errors:

```json
{
  "isError": true,
  "code": "sensitive_confirmation_required",
  "message": "This item is sensitive. Pass confirm_sensitive=true for this request."
}
```

Do not include raw sensitive content in error messages.

## CLI Doctor

### Checks

- Config parse.
- Data directory exists/writable.
- DB path exists/readable or can be created.
- Runtime directory exists.
- IPC socket reachable.
- Daemon ping.
- Session type Wayland.
- Known compositor where detectable.
- `wl-copy` availability.
- `wtype` availability.
- `ydotool` availability.
- External picker availability: `wofi`, `fuzzel`, `rofi`.
- GTK dependencies where practical.
- MCP binary availability.
- Systemd user service status where available.

### Output Modes

- Human default.
- `--json`.
- Exit codes:
  - `0`: healthy,
  - `1`: warnings,
  - `2`: errors,
  - `3`: daemon unavailable,
  - `4`: config invalid.

### `--fix`

Allowed:

- create app config directory,
- create app data directory,
- create default config,
- install/update user-owned managed config snippets only when path is explicit,
- print systemd commands rather than running privileged operations.

Not allowed:

- overwrite compositor config blindly,
- install packages,
- edit root-owned files,
- disable privacy settings.

## Hyprland Config Generator

Generated block:

```ini
# BEGIN author-clipboard
bind = SUPER, V, exec, author-clipboard-ctl picker --menu auto
bind = SUPER SHIFT, V, exec, author-clipboard --popup
bind = SUPER ALT, V, exec, author-clipboard --manager
# END author-clipboard
```

Rules:

- Print by default.
- `--write <path>` writes only a managed block.
- Existing managed block is replaced.
- Unrelated config remains untouched.
- `--dry-run` shows diff.
- `--backup` creates timestamped backup.

## UI Design System

### Shell

Popup:

- search at top,
- status pills,
- grouped result list,
- selected action bar,
- footer hint.

Manager:

- header,
- sidebar,
- content stack,
- preview/action pane,
- status/diagnostic footer.

### CSS Classes

Recommended stable classes:

- `.command-popup`
- `.command-search`
- `.status-pill`
- `.status-pill-private`
- `.status-pill-incognito`
- `.result-group`
- `.result-card`
- `.result-card-selected`
- `.content-badge`
- `.content-badge-secret`
- `.content-badge-url`
- `.content-badge-json`
- `.content-badge-code`
- `.action-bar`
- `.preview-pane`
- `.secret-card`
- `.color-swatch`
- `.doctor-status`

### Accessibility

Every button/icon needs:

- accessible name,
- tooltip where helpful,
- keyboard activation,
- focus style.

## Manager Workspace Pages

Recommended final navigation:

- Home.
- History.
- Pinned.
- Secrets.
- Images.
- Links.
- Code.
- Files.
- Snippets.
- Rules.
- MCP.
- Settings.

Implementation may hide pages until functional. Do not show blank placeholders.

## Database and Migration

Potential durable additions:

- `source_app`
- `source_window`
- `tags`
- `presentation_cache_version`
- `safe_preview`
- `redacted_preview`
- `used_count`
- `last_used_at`
- rule/config tables if not file-backed.

Migration requirements:

- Fixture from pre-feature DB.
- Forward migration test.
- Read compatibility test.
- No destructive migration without backup guidance.

## Import/Export

Default export mode is redacted.

```rust
pub enum ExportMode {
    Redacted,
    FullWithConfirmation,
    SnippetsOnly,
    SettingsOnly,
}
```

Import must:

- validate format,
- re-run sensitive detection,
- show counts,
- avoid overwriting without confirmation.

## Testing Strategy

### Unit Tests

- content classification,
- privacy policy,
- sensitive detection,
- transforms,
- snippet expansion,
- rules,
- config migration,
- selection reducer,
- MCP confirmation errors,
- doctor diagnostics.

### Integration Tests

- IPC snapshot/search/copy/transform.
- daemon capture rule path with mock source app.
- database migration fixture.
- MCP tool/resource output redaction.

### UI Tests / Smoke

- popup opens.
- search focuses.
- result action works.
- secret reveal auto-redacts.
- manager preview updates.
- empty states render.
- shortcuts overlay opens.

### Manual Matrix

- Hyprland.
- COSMIC.
- Sway.
- No daemon.
- Empty DB.
- 1,000 item DB.
- Sensitive content.
- Image/file content.
- MCP client setup.

## Observability and Logging

Rules:

- Use tracing spans with item IDs and content hashes, not raw content.
- Sensitive events log safe kind/category only.
- Doctor may include paths but not secret values.
- MCP logs tool names and IDs, not raw arguments when sensitive.
- Debug logs must pass a sensitive fixture scan.

## Rollout Plan

1. Foundation branch:
   - authoritative model,
   - correct selection,
   - refresh signaling,
   - list rendering.
2. Content branch:
   - presentation model,
   - rich cards,
   - preview pane.
3. Privacy branch:
   - secure defaults,
   - privacy policy,
   - MCP enforcement.
4. Workflow branch:
   - actions,
   - transforms,
   - snippets variables,
   - rules.
5. Adoption branch:
   - doctor,
   - Hyprland generator,
   - README,
   - screenshots,
   - release checklist.

## Compatibility Rules

- Additive IPC fields use serde defaults.
- Existing IPC command variants remain unless a documented migration is
  required.
- Existing config loads with defaults.
- Existing DB migrates forward.
- Existing CLI commands continue.
- Package names and binaries remain stable where possible.
- Any breaking change must be recorded in `09-decisions.md` and release notes.

## Open Questions

Track answers in `09-decisions.md`:

1. Which GTK model/list approach is stable with current dependencies?
2. Is daemon event subscription feasible now, or should v1 use explicit snapshot
   refresh after mutations?
3. How should older config files without explicit encryption settings migrate?
4. Which source-app metadata is available reliably across Hyprland, COSMIC, and
   Sway?
5. Which MCP client besides Codex is verified for docs?
6. Should rules live in JSON config, SQLite, or both?
7. Should usage ranking be implemented in this feature or deferred?
8. Which package formats are release-blocking?

## Done Definition

The design is implemented only when:

- GTK result pages no longer query DB directly.
- Selection by ID is correct.
- Rich cards come from shared presentation models.
- Privacy is centralized.
- MCP redaction is tested.
- CLI doctor reports useful setup state.
- README and screenshots match shipped behavior.
- `just verify`, focused tests, and manual Wayland smoke checks pass.

**Created**: 2026-07-11  
**Updated**: 2026-07-12  
**Status**: Proposed
