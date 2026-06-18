# Technical Design: Native Power-User Revamp

> Implementation approach for a staged revamp across UI, storage, IPC, CLI,
> compositor integration, and install reliability.

---

## Overview

This revamp should be implemented as small, verifiable slices. The current
`ui-gtk` crate is the right foundation for the picker/manager UI. The older
`016-world-class-ux` design references `crates/applet/src/*` modules and should
not be followed literally for new UI code.

Architecture direction:

- Keep `shared` as the data, DB, IPC, picker, template, and classification
  foundation.
- Keep `clipboard-daemon` responsible for capture, storage, policy, and IPC.
- Keep `ui-gtk` as the first-party GTK4/libadwaita UI implementation.
- Keep `hypr-picker` as a thin binary over `ui-gtk::run_popup`.
- Keep `ctl` as the automation surface and external-menu bridge.

## Affected Files

| Area | Files |
|------|-------|
| UI shell | `crates/ui-gtk/src/window/popup.rs`, `crates/ui-gtk/src/window/manager.rs`, `crates/ui-gtk/data/style.css` |
| UI state | `crates/ui-gtk/src/app.rs`, `crates/ui-gtk/src/model.rs`, `crates/ui-gtk/src/settings.rs` |
| Widgets | `crates/ui-gtk/src/widgets/item_row.rs`, `preview.rs`, `filter_bar.rs`, `shortcuts_overlay.rs`, new action/inspector widgets |
| Pages | `crates/ui-gtk/src/pages/clipboard.rs`, `snippets.rs`, new collections/saved-filter pages |
| Shared model | `crates/shared/src/types.rs`, `picker.rs`, `db.rs`, `config.rs`, new query/classification module |
| IPC | `crates/shared/src/ipc.rs`, `crates/clipboard-daemon/src/main.rs` |
| CLI | `crates/ctl/src/main.rs` |
| Install/package | `justfile`, `data/*.desktop`, `packaging/**`, `flake.nix`, `default.nix` |
| Docs/spec | `PROJECT_PLAN.md`, `README.md`, `docs/UI.md`, `docs/HYPRLAND.md`, this spec |
| Dotfiles integration | Optional user-machine rule in `/home/namik/Documents/code/dotfiles/hypr/conf/70-windowrules.lua`; do not treat it as upstream package logic |

## Implementation Details

### 1. Query Parser

Add a pure parser in `shared`, for example:

```rust
pub fn parse_query(raw: &str) -> ParsedQuery;
```

Rules:

- tokenize quoted phrases safely
- parse `key:value` filters from a small allowlist
- preserve unknown tokens as plain text terms
- return warnings, not hard errors
- test with Unicode and malformed quotes

### 2. Content Classification

Add a classification helper that runs after load or at display time:

```rust
pub fn classify_item(item: &ClipboardItem) -> ContentClass;
```

Start with lightweight heuristics:

- content type/MIME
- URL parse
- file path/path-like detection
- JSON parse attempt for small text
- SQL keywords
- shell command patterns
- code fences/import/function keywords

Do not block capture on classification. If classification becomes expensive,
cache it in a future migration.

### 3. Inspector

Build an `InspectorPane` in `ui-gtk`:

- summary header: type, age, MIME, size, flags
- preview content area
- action rail
- warning/redaction block for sensitive items
- collection badges

Implementation should reuse existing `PreviewPane` where possible instead of
creating a second incompatible preview system.

### 4. Action Rail

Actions should dispatch through UI state and IPC:

```rust
Action::CopyRequested(id)
Action::QuickPasteRequested(id)
Action::TogglePin(id)
Action::ToggleStar(id)
Action::DeleteItem(id)
Action::AddToCollection(id)
Action::RevealRedacted(id)
```

Each action needs:

- keyboard shortcut
- button/menu path
- disabled state
- toast/status feedback
- CLI equivalent where relevant

### 5. Collections And Saved Filters

Use the `015-collections` database model with additive migrations. Saved filters
are similar but simpler.

Collections should be manipulated through DB helpers first, then IPC, then UI.
Avoid implementing UI-only fake collections.

### 6. Native Windowing

Hypr picker default:

- normal XDG window
- resizable
- close button
- Esc close
- class `com.namikofficial.author-clipboard.popup`
- optional `--layer-shell`

Package-level install cannot force a user's Hyprland rule, but docs and helper
commands should emit the recommended class rule.

### 7. Performance Strategy

Start pragmatic:

- indexed DB search and limited result windows
- lazy preview load
- avoid rendering thousands of row widgets at once
- use pagination or GTK list model/factory if needed
- benchmark with seeded 5k-item DB before claiming "virtualized"

### 8. Import/Export V2

Extend export schema to include:

- items
- snippets
- collections
- collection memberships
- saved filters
- metadata version

Default export redacts sensitive/encrypted payloads. Unsafe export must be an
explicit flag and should warn in CLI/UI.

## Security Considerations

- Reveal flows must be explicit and time-limited.
- Logs must never include raw sensitive content.
- Query parser must not execute anything.
- HTML preview must default to safe text fallback; rendered HTML, if added,
  must be sandboxed or otherwise constrained.
- Import must validate schema and refuse path traversal for file/image data.

## Error Handling

| Error Condition | Handling Strategy |
|-----------------|-------------------|
| Daemon down | UI health state; CLI non-zero for commands needing daemon. |
| DB migration failure | Abort with clear log; do not corrupt DB. |
| Invalid query token | Warning in UI; plain search fallback. |
| Missing quick-paste backend | Disable quick paste; copy remains available. |
| Missing GSettings schema | Install recipe fixes; UI should fall back to defaults. |
| Collection not found | Show stale-state toast and refresh collections. |
| Sensitive reveal denied | Keep redacted and explain why. |

## Migration Strategy

1. Add migrations for collections and saved filters.
2. Add DB tests for empty, populated, delete, and membership cases.
3. Ensure old DBs open and migrate automatically.
4. Keep export/import versioned.
5. Record irreversible migration decisions in `09-decisions.md`.

---

**Last Updated**: 2026-06-19
