# Task Plan: Wayland Clipboard Command Center

> Implementation-ready plan for turning Author Clipboard into a polished,
> private, keyboard-first clipboard command center for Wayland.

## Execution Rules

- Complete one task at a time.
- Do not start visual expansion before the UI model is authoritative.
- Do not duplicate privacy logic in UI, CLI, and MCP.
- Do not add network-based preview features in this feature.
- Do not update README with unshipped claims.
- Every task must leave tests or verification commands behind.
- Any behavior deviation must be recorded in `09-decisions.md`.

## High-Level Sequence

```text
Phase 0: Audit / truth
  T001

Phase 1: Foundation
  T002 → T003 → T004 → T005

Phase 2: Content-aware command UI
  T006 → T007 → T008 → T009 → T010

Phase 3: Privacy and developer workflows
  T011 → T012 → T013 → T014 → T015

Phase 4: MCP, setup, docs
  T016 → T017 → T018 → T019

Phase 5: Release validation
  T020
```

## Dependency Graph

```text
T001 Audit
 ├─ T002 UI state authority
 │   ├─ T003 Correct ID selection/actions
 │   ├─ T004 Model-backed list rendering
 │   └─ T005 Explicit refresh signaling
 │       ├─ T008 Rich UI cards
 │       ├─ T009 Popup action bar/grouping
 │       └─ T018 Doctor/config generator
 │
 ├─ T006 Content presentation model
 │   ├─ T008 Rich UI cards
 │   ├─ T013 Transformations
 │   └─ T016 MCP safety
 │
 ├─ T011 Sensitive defaults migration
 │   ├─ T012 App capture rules
 │   ├─ T013 Transformations
 │   ├─ T015 Import/export safety
 │   └─ T016 MCP safety
 │
 └─ T019 README/demo

T008 + T009 + T016 + T018 → T019 README/demo
T019 → T020 Release validation
```

---

## Phase 0 — Audit and Product Truth

### T001 — Audit Current Behavior and Claims

**Priority**: P0  
**Depends on**: none  
**Goal**: Establish the exact current behavior before changing architecture,
privacy, or public docs.

**Files to inspect/edit**:

- `README.md`
- `CHANGELOG.md`
- `Cargo.toml`
- `crates/ui-gtk/src/app.rs`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/src/window/{popup,manager}.rs`
- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/`
- `crates/mcp-server/`
- `packaging/`
- `specs/features/024-wayland-command-center/09-decisions.md` (create if needed)

**Implementation**:

1. Run the baseline checks:
   ```bash
   just verify
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   ```
2. Trace all result-loading paths:
   ```bash
   rg -n 'Database::open|load_entries|History|Search|ItemsLoaded|timeout_add_local_once' crates/ui-gtk crates/shared crates/clipboard-daemon
   ```
3. Trace all copy constructors:
   ```bash
   rg -n 'IpcCommand::Copy|CopyMode|QuickPaste' crates/
   ```
4. Trace sensitive-output surfaces:
   ```bash
   rg -n 'sensitive|redact|redacted|encrypt|log|tracing|serde_json::to_string' crates/
   ```
5. Compare README/package claims with actual release artifacts.
6. Record current gaps in `09-decisions.md` or a short audit section.

**Verification**:

```bash
just verify
rg -n 'Database::open|timeout_add_local_once' crates/ui-gtk/src
```

**Done when**:

- Current failures are listed with exact commands.
- UI state drift points are identified.
- README overclaims, if any, are corrected or noted.
- No code architecture changes are mixed into this task.

**Rollback risk**: Low — audit/docs only.

---

## Phase 1 — Foundation

### T002 — Make GTK Item State Authoritative

**Priority**: P0  
**Depends on**: T001  
**Goal**: Popup and manager render and act on the same in-memory item model.

**Files to edit**:

- `crates/ui-gtk/src/app.rs`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/src/window/popup.rs`
- `crates/ui-gtk/src/window/manager.rs`
- `crates/ui-gtk/src/controller/` (new if needed)

**Implementation**:

1. Add/finish an authoritative `items` model in `AppState`.
2. Route initial item load through an IPC/controller function.
3. Remove direct result database reads from GTK page widgets.
4. Preserve existing `PopupConfig` behavior:
   - query,
   - filter,
   - count,
   - source,
   - action.
5. Replace page-local item side tables with data derived from authoritative
   state.
6. Ensure manager and popup share the same result-loading path.

**Verification**:

```bash
cargo test -p author-clipboard-ui-gtk -- app clipboard
rg -n 'Database::open' crates/ui-gtk/src
just verify
```

**Done when**:

- `crates/ui-gtk/src/pages/clipboard.rs` no longer opens DB directly.
- Popup and manager use the same item loading function/controller.
- Existing basic copy flow still works.

**Rollback risk**: Medium — data-flow change.

**Implementation status (2026-07-12)**: In progress — authoritative state and
IPC snapshot controller implemented in the foundation increment.

---

### T003 — Fix ID-Based Selection and Action Targeting

**Priority**: P0  
**Depends on**: T002  
**Goal**: Eliminate placeholder index behavior and guarantee actions target the
selected item.

**Files to edit**:

- `crates/ui-gtk/src/app.rs`
- `crates/ui-gtk/src/actions.rs` (new if needed)
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/src/controller/key.rs`

**Implementation**:

1. Replace placeholder `Select(Some(_)) -> selected_index = Some(0)`.
2. Store `selected_id: Option<i64>` or equivalent.
3. Add reducer helpers:
   - `select_by_id`,
   - `selected_item`,
   - `selected_index`,
   - `move_selection`,
   - `selection_after_delete`,
   - `preserve_selection_after_refresh`.
4. Ensure actions derive IDs from selected item in current model.
5. Add tests for selection/action correctness.

**Verification**:

```bash
cargo test -p author-clipboard-ui-gtk -- select selected action app
just verify
```

**Done when**:

- Unknown ID never maps to row 0.
- Delete/pin/star/copy/quick-paste target selected item.
- Selection survives refresh when possible.

**Rollback risk**: Medium.

**Implementation status (2026-07-12)**: In progress — selected ID and pure
selection helpers implemented with reducer coverage.

---

### T004 — Replace Rebuild-on-Refresh Rendering

**Priority**: P0  
**Depends on**: T002, T003  
**Goal**: Make result rendering scalable and less janky.

**Files to edit**:

- `crates/ui-gtk/src/model.rs` (new)
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/src/widgets/item_row.rs`
- `crates/ui-gtk/src/widgets/empty.rs`
- `crates/ui-gtk/src/window/{popup,manager}.rs`

**Implementation status (2026-07-12)**: In progress — keyed reconciliation
model selected and covered by a 1,000-item synthetic harness.

**Implementation**:

1. Evaluate GTK model/list factory support with current dependency versions.
2. Implement preferred model-backed list if stable.
3. If not stable, implement keyed row reuse:
   - cache rows by item ID,
   - rebind changed rows,
   - insert/move/remove minimal rows.
4. Preserve keyboard focus and selected ID.
5. Add synthetic 1,000-entry render/refresh harness.
6. Record measured numbers in comments/docs/test output.

**Verification**:

```bash
cargo test -p author-clipboard-ui-gtk -- model item_row clipboard
just verify
```

**Manual smoke**:

- Open popup with empty DB.
- Open popup with 1,000 synthetic rows.
- Search.
- Delete selected row.
- Pin/unpin selected row.
- Confirm focus remains sane.

**Done when**:

- Refresh no longer removes/recreates every visible child.
- Performance is measured, not guessed.
- Empty states still work.

**Rollback risk**: Medium — GTK lifecycle/focus.

---

### T005 — Add Explicit Refresh Signaling

**Priority**: P0  
**Depends on**: T002  
**Goal**: Eliminate fixed-delay refresh as the synchronization strategy.

**Files to edit**:

- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/src/main.rs`
- `crates/clipboard-daemon/src/` handlers/modules
- `crates/ui-gtk/src/controller/refresh.rs` (new)
- `crates/ui-gtk/src/window/{popup,manager}.rs`
- `crates/ctl/src/main.rs` if command matching requires updates

**Implementation**:

1. Audit if any daemon notification/subscription already exists.
2. Choose:
   - event/watch command, or
   - explicit snapshot refresh after mutations for v1.
3. Add versioned/additive IPC fields with serde defaults.
4. Refresh on:
   - window open,
   - capture,
   - delete,
   - clear,
   - pin/unpin,
   - star/unstar,
   - snippet mutation,
   - config/incognito changes.
5. Show daemon unavailable state with retry.
6. Remove `timeout_add_local_once` refresh hacks from primary flow.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- ipc
cargo test -p author-clipboard-daemon
cargo test -p author-clipboard-ui-gtk -- refresh
rg -n 'timeout_add_local_once' crates/ui-gtk/src
just verify
```

**Done when**:

- Refresh behavior is explicit and tested.
- Existing IPC clients remain compatible or migration is documented.
- UI does not depend on arbitrary 200 ms refresh.

**Rollback risk**: Medium — IPC protocol and daemon/UI integration.

---

## Phase 2 — Content-Aware Command UI

### T006 — Implement Shared Content Presentation Model

**Priority**: P1  
**Depends on**: T001  
**Goal**: Create one pure representation for recognizable content cards.

**Files to edit**:

- `crates/shared/src/content.rs` (new)
- `crates/shared/src/lib.rs`
- `crates/shared/src/types.rs`
- `crates/shared/src/sensitive.rs`

**Implementation**:

1. Add `ContentPresentation` enum.
2. Add bounded classifiers:
   - secret,
   - URL,
   - color,
   - JSON,
   - code-like text,
   - HTML,
   - image,
   - file URI,
   - text,
   - unknown.
3. Ensure sensitive classification runs first.
4. Add size limits and safe fallbacks.
5. Add unit tests for every kind and malformed input.
6. Do not add network, OCR, WebKit, or heavy syntax-highlighting dependency.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- content sensitive
just verify
```

**Done when**:

- Classification is deterministic and local-only.
- Every parser has tests.
- Sensitive input always returns secret presentation.

**Rollback risk**: Low.

---

### T007 — Create Shared Item View Model

**Priority**: P1  
**Depends on**: T006  
**Goal**: Bind UI/MCP/CLI to one safe item representation.

**Files to edit**:

- `crates/shared/src/view.rs` (new)
- `crates/shared/src/lib.rs`
- `crates/shared/src/ipc.rs`
- `crates/ui-gtk/src/app.rs`
- `crates/mcp-server/src/handler.rs`

**Implementation**:

1. Add `ItemViewModel` or equivalent.
2. Include:
   - ID,
   - safe preview,
   - `ContentPresentation`,
   - source app,
   - timestamp,
   - pinned/starred,
   - sensitive/encrypted,
   - tags,
   - allowed actions.
3. Ensure the view model uses shared privacy policy.
4. IPC snapshot/search responses can return or map to this model.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- view content
cargo test -p author-clipboard-ui-gtk -- app
just verify
```

**Done when**:

- UI rows can render without reclassifying independently.
- Sensitive previews are safe by construction.

**Rollback risk**: Low to medium.

---

### T008 — Render Rich Result Cards and Preview Pane

**Priority**: P1  
**Depends on**: T004, T006, T007  
**Goal**: Make content awareness visible.

**Files to edit**:

- `crates/ui-gtk/src/widgets/item_row.rs`
- `crates/ui-gtk/src/widgets/preview.rs`
- `crates/ui-gtk/src/widgets/chip.rs`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/data/style.css`
- `crates/ui-gtk/assets/icons/` if needed

**Implementation**:

1. Render content badges.
2. Add visual treatments:
   - URL domain,
   - color swatch,
   - JSON/code compact preview,
   - image thumbnail/metadata,
   - file name/path hint,
   - secret redacted card.
3. Manager preview shows richer metadata.
4. Rows stay compact.
5. Add accessibility labels/tooltips.
6. Update screenshots after implementation.

**Verification**:

```bash
cargo test -p author-clipboard-ui-gtk -- item_row preview chip
just verify
```

**Manual smoke**:

- text,
- URL,
- color,
- JSON,
- code,
- image,
- file URI,
- secret.

**Done when**:

- Each supported type is visibly distinct.
- Secret raw content is not visible by default.
- Light/dark themes remain usable.

**Rollback risk**: Medium — visual regression.

---

### T009 — Add Command Popup Shell, Grouping, and Action Bar

**Priority**: P1  
**Depends on**: T003, T008  
**Goal**: Make popup feel like a launcher-style command surface.

**Files to edit**:

- `crates/ui-gtk/src/window/popup.rs`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/src/widgets/action_bar.rs` (new)
- `crates/ui-gtk/src/widgets/status_pill.rs` (new)
- `crates/ui-gtk/src/widgets/shortcuts_overlay.rs`
- `crates/ui-gtk/src/actions.rs`
- `crates/ui-gtk/data/style.css`

**Implementation**:

1. Add command popup shell:
   - search,
   - status pills,
   - grouped result list,
   - selected action bar,
   - footer hints.
2. Add groups for empty-query browsing:
   - pinned,
   - recent,
   - today,
   - links,
   - code,
   - images,
   - files,
   - secrets.
3. Search mode prioritizes relevance/chronology over grouping.
4. Add action bar:
   - copy,
   - quick paste,
   - plain text,
   - transform,
   - pin,
   - star,
   - delete,
   - create snippet,
   - reveal where applicable.
5. Keep feature-023 keyboard semantics intact.
6. Add screen-reader labels.

**Verification**:

```bash
cargo test -p author-clipboard-ui-gtk -- actions key popup
just verify
```

**Manual smoke**:

- keyboard-only open/search/select/copy/quick-paste/delete.
- `?` shortcut overlay.
- screen-reader label spot-check if tooling available.

**Done when**:

- Selected-row actions are visible.
- Keyboard shortcuts still work.
- Popup feels like command surface, not utility list.

**Rollback risk**: Medium.

---

### T010 — Upgrade Manager Workspace Navigation

**Priority**: P1  
**Depends on**: T008  
**Goal**: Make the manager a real workspace without showing broken pages.

**Files to edit**:

- `crates/ui-gtk/src/window/manager.rs`
- `crates/ui-gtk/src/pages/home.rs` (new)
- `crates/ui-gtk/src/pages/rules.rs` (new later/placeholder hidden until ready)
- `crates/ui-gtk/src/pages/mcp.rs` (new later/placeholder hidden until ready)
- `crates/ui-gtk/data/style.css`

**Implementation**:

1. Rename title to `Author Clipboard`.
2. Add Home page with:
   - daemon status,
   - item counts,
   - privacy status,
   - shortcut hint,
   - quick actions.
3. Rework sidebar labels/icons.
4. Only show pages that function.
5. Add responsive behavior for narrower windows.
6. Persist last page and window size.

**Verification**:

```bash
cargo test -p author-clipboard-ui-gtk -- manager home
just verify
```

**Manual smoke**:

- manager launch,
- page switching,
- resize below breakpoint,
- preview updates,
- settings opens.

**Done when**:

- No broken placeholder pages are visible.
- Manager communicates app status and value.

**Rollback risk**: Low to medium.

---

## Phase 3 — Privacy and Developer Workflows

### T011 — Secure Sensitive Defaults and Migration

**Priority**: P0  
**Depends on**: T001  
**Goal**: New profiles protect sensitive records by default.

**Files to edit**:

- `crates/shared/src/config.rs`
- `crates/shared/src/encryption.rs`
- `crates/shared/src/sensitive.rs`
- `crates/shared/src/db.rs`
- `crates/clipboard-daemon/` storage/capture path
- `docs/PRIVACY.md` (new)
- `specs/features/024-wayland-command-center/09-decisions.md`

**Implementation**:

1. Add config-versioned migration.
2. Default new profiles to `encrypt_sensitive=true`.
3. Preserve existing explicit setting.
4. Decide older missing-key behavior in `09-decisions.md`.
5. Ensure sensitive raw content is not indexed unsafely.
6. Audit logs for raw sensitive output.
7. Add migration fixture tests.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- config encryption sensitive db
cargo test -p author-clipboard-daemon -- sensitive
just verify
```

**Done when**:

- New config defaults secure.
- Existing explicit settings preserved.
- Tests prove migration behavior.
- Logs do not expose sensitive fixtures.

**Rollback risk**: High — privacy/storage behavior.

---

### T012 — Add Capture Rules and Ignore-Next-Copy

**Priority**: P2  
**Depends on**: T011  
**Goal**: Let users control capture before storage.

**Files to edit**:

- `crates/shared/src/rules.rs` (new)
- `crates/shared/src/config.rs`
- `crates/shared/src/types.rs`
- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/` capture path
- `crates/ui-gtk/src/pages/settings.rs`
- `crates/ui-gtk/src/pages/rules.rs`
- `crates/ctl/src/main.rs`

**Implementation**:

1. Define rule model and precedence.
2. Implement pure rule evaluator.
3. Integrate evaluator into daemon capture path.
4. Add ignore-next-copy runtime state.
5. Add CLI:
   ```bash
   author-clipboard-ctl ignore-next-copy
   ```
6. Add settings/rules UI.
7. Add feedback event/toast.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- rules
cargo test -p author-clipboard-daemon -- ignore_next capture_rules
cargo test -p author-clipboard-ui-gtk -- rules settings
just verify
```

**Done when**:

- Ignore-next-copy skips exactly one eligible capture.
- Rules can ignore/redact/tag.
- Broken rules can be disabled/reset.

**Rollback risk**: Medium.

---

### T013 — Implement Transformations

**Priority**: P2  
**Depends on**: T006, T011  
**Goal**: Add useful developer conversions through shared pure logic.

**Files to edit**:

- `crates/shared/src/transform.rs` (new)
- `crates/shared/src/lib.rs`
- `crates/shared/src/ipc.rs`
- `crates/ui-gtk/src/actions.rs`
- `crates/ui-gtk/src/widgets/action_bar.rs`
- `crates/ctl/src/main.rs`
- `crates/mcp-server/src/handler.rs`

**Implementation**:

1. Add transform enum:
   - plain text,
   - Markdown link,
   - fenced code,
   - quote,
   - JSON pretty,
   - JSON minified,
   - redacted.
2. Add pure transform functions.
3. Add safe errors.
4. Add IPC command or action path.
5. Expose in popup action bar.
6. Expose in CLI.
7. Expose in MCP with privacy confirmation.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- transform
cargo test -p author-clipboard-ui-gtk -- actions
cargo test -p author-clipboard-mcp -- transform
just verify
```

**Done when**:

- Invalid transforms do not mutate original.
- Sensitive policy is enforced.
- JSON pretty/minified tested.

**Rollback risk**: Low.

**Shared-layer status (2026-07-12)**: Complete — transform enum, pure
implementation, privacy gate, safe errors, and tests. Surface adapters remain.

---

### T014 — Add Snippet Variables

**Priority**: P2  
**Depends on**: T011, T013  
**Goal**: Make snippets more powerful for daily developer use.

**Files to edit**:

- `crates/shared/src/snippet_template.rs` (new)
- `crates/shared/src/types.rs`
- `crates/shared/src/ipc.rs`
- `crates/ui-gtk/src/pages/snippets.rs`
- `crates/ctl/src/main.rs`
- `docs/SNIPPETS.md` (new)

**Implementation**:

1. Implement variables:
   - `{date}`,
   - `{time}`,
   - `{clipboard}`,
   - `{selection}`.
2. Implement escaping:
   - `{{date}}` -> literal `{date}`.
3. Add preview/validation.
4. Enforce sensitive confirmation.
5. Update snippets UI.
6. Add docs.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- snippet_template
cargo test -p author-clipboard-ui-gtk -- snippets
just verify
```

**Done when**:

- Snippet expansion is deterministic and documented.
- Sensitive sources require confirmation.
- Unknown variables fail safely.

**Rollback risk**: Low.

**Shared-layer status (2026-07-12)**: Complete — strict compatibility syntax,
escaping, validation, sensitive-source confirmation, and tests. Surface adapters remain.

---

### T015 — Safe Import/Export

**Priority**: P2  
**Depends on**: T011  
**Goal**: Make backup and migration safe by default.

**Files to edit**:

- `crates/shared/src/export.rs`
- `crates/shared/src/import.rs`
- `crates/shared/src/ipc.rs`
- `crates/ctl/src/main.rs`
- `crates/ui-gtk/src/pages/settings.rs`
- `docs/PRIVACY.md`

**Implementation**:

1. Add export modes:
   - redacted default,
   - full with explicit confirmation,
   - snippets only,
   - settings only.
2. Import re-runs sensitive detection.
3. Import preview shows counts/warnings.
4. CLI confirms destructive imports.
5. UI settings exposes safe export.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- import export sensitive
cargo test -p author-clipboard-ctl -- export import
just verify
```

**Done when**:

- Default export is redacted.
- Full export requires explicit confirmation.
- Import does not bypass sensitive detection.

**Rollback risk**: Medium.

**Shared-layer status (2026-07-12)**: Complete for history — versioned envelope,
redacted default, full-export confirmation, preview, re-detection, and tests.
Snippet/settings serialization remains with their surface adapters.

---

## Phase 4 — MCP, Setup, and Adoption

### T016 — Enforce MCP Safety Boundary

**Priority**: P1  
**Depends on**: T006, T011  
**Goal**: Make MCP a safe, defensible product feature.

**Files to edit**:

- `crates/mcp-server/src/handler.rs`
- `crates/mcp-server/src/tools.rs` (new if splitting)
- `crates/mcp-server/src/resources.rs` (new if splitting)
- `crates/mcp-server/src/error.rs` (new)
- `crates/shared/src/privacy.rs` (new if needed)
- `specs/features/013-mcp-protocol/09-decisions.md`

**Implementation**:

1. Centralize MCP output redaction.
2. Require `confirm_sensitive=true` or structured confirmation for:
   - full get,
   - copy full sensitive,
   - sensitive transform.
3. Require `confirm=true` for destructive operations.
4. Return machine-readable MCP errors.
5. Add tests proving default results do not include raw sensitive content.
6. Update tool descriptions.

**Verification**:

```bash
cargo test -p author-clipboard-mcp -- sensitive confirmation delete resources tools
just verify
```

**Done when**:

- Search/resources are redacted by default.
- Full sensitive content requires per-request confirmation.
- Destructive operations require confirmation.

**Rollback risk**: Medium — client behavior changes.

---

### T017 — Document MCP Integrations

**Priority**: P1  
**Depends on**: T016  
**Goal**: Make the MCP advantage discoverable and safe to configure.

**Files to edit**:

- `docs/MCP.md` (new)
- `README.md`
- `crates/mcp-server/README.md` if present

**Implementation**:

1. Add stdio setup for Codex.
2. Add one verified second MCP client.
3. Add safe prompts:
   - find copied stack trace,
   - summarize recent copied notes,
   - create snippet from last command,
   - find JSON payload.
4. Explain redaction/confirmation.
5. State local-only architecture.
6. State what data the MCP client can access.

**Verification**:

```bash
rg -n 'confirm_sensitive|redacted|local-only|stdio|Codex' README.md docs/MCP.md
just verify
```

**Done when**:

- A user can configure MCP from docs.
- Privacy behavior is clear.

**Rollback risk**: Low.

---

### T018 — Add Doctor and Hyprland Config Generator

**Priority**: P2  
**Depends on**: T005, T011  
**Goal**: Reduce setup failure and make installation feel professional.

**Files to edit**:

- `crates/ctl/src/main.rs`
- `crates/shared/src/diagnostics.rs` (new)
- `crates/shared/src/compositor.rs` (new)
- `crates/shared/src/config.rs`
- `docs/LOCAL_TESTING.md`
- `docs/HYPRLAND.md`

**Implementation**:

1. Add:
   ```bash
   author-clipboard-ctl doctor
   author-clipboard-ctl doctor --json
   author-clipboard-ctl doctor --fix
   ```
2. Checks:
   - daemon,
   - IPC socket,
   - config,
   - DB,
   - data dirs,
   - Wayland session,
   - compositor,
   - wl-copy,
   - wtype,
   - ydotool,
   - picker tools.
3. Add:
   ```bash
   author-clipboard-ctl hyprland-config
   author-clipboard-ctl hyprland-config --write ~/.config/hypr/hyprland.conf
   ```
4. Managed block only.
5. Dry-run/diff behavior.
6. Safe user-owned changes only.

**Verification**:

```bash
cargo test -p author-clipboard-shared -- diagnostics compositor
cargo test -p author-clipboard-ctl -- doctor hyprland
just verify
```

**Done when**:

- Doctor reports actionable setup state.
- `--fix` does not mutate unsafe paths.
- Hyprland write is idempotent.

**Rollback risk**: Medium — user file writes.

---

### T019 — Rewrite README and Product Assets

**Priority**: P1  
**Depends on**: T008, T009, T016, T018  
**Goal**: Make the repo star-worthy and install-worthy.

**Files to edit**:

- `README.md`
- `docs/`
- `docs/UI/snapshots/`
- `docs/assets/` or `assets/`
- package readmes if applicable

**Implementation**:

1. Lead with:
   - tagline,
   - short pitch,
   - demo GIF/screenshot,
   - install path.
2. Add sections:
   - Why Author Clipboard,
   - Features,
   - Privacy model,
   - Hyprland setup,
   - COSMIC/Sway notes,
   - MCP setup,
   - CLI,
   - Development,
   - Roadmap.
3. Add screenshots:
   - popup,
   - manager,
   - secret card,
   - URL/color/code/JSON/image/file preview,
   - snippets,
   - MCP safety.
4. Do not overclaim package availability.
5. Move dev-branch warning below the value proposition.
6. Add clear comparison wording without unsupported claims.

**Verification**:

```bash
rg -n 'private clipboard command center|Install|MCP|Privacy|Hyprland' README.md
just verify
just ui-smoke
```

**Done when**:

- README first screen explains why users should care.
- Screenshots reflect real app.
- Install docs are truthful.

**Rollback risk**: Low.

---

## Phase 5 — Release Validation

### T020 — Release Candidate Test Plan and Checklist

**Priority**: P0  
**Depends on**: T019  
**Goal**: Validate the feature across supported environments before release.

**Files to create/edit**:

- `specs/features/024-wayland-command-center/07-test-plan.md`
- `specs/features/024-wayland-command-center/08-review-checklist.md`
- `specs/features/024-wayland-command-center/09-decisions.md`
- `CHANGELOG.md`
- `SECURITY.md`
- `README.md`

**Implementation**:

1. Create test matrix:
   - Hyprland,
   - COSMIC,
   - Sway,
   - daemon unavailable,
   - empty DB,
   - existing DB,
   - migrated config,
   - 1,000 items,
   - sensitive items,
   - MCP client.
2. Validate:
   - DB migration,
   - config migration,
   - redaction,
   - reveal timeout,
   - MCP confirmation,
   - transforms,
   - snippets,
   - rules,
   - doctor,
   - install docs,
   - screenshots.
3. Record accepted deviations.
4. Update changelog.
5. Confirm release artifacts actually exist before documenting them.

**Verification**:

```bash
just verify
just ui-check
just ui-smoke
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

**Done when**:

- Test plan exists.
- Review checklist exists.
- Manual matrix results are recorded.
- Release docs do not overclaim.
- Security/privacy behavior is validated.

**Rollback risk**: N/A — validation task.

---

## Suggested Branch/PR Slicing

| PR | Tasks | Name |
|---|---|---|
| PR 1 | T001 | `audit/024-command-center-truth` |
| PR 2 | T002-T003 | `feat/ui-authoritative-model` |
| PR 3 | T004 | `feat/ui-model-backed-list` |
| PR 4 | T005 | `feat/ipc-refresh-signaling` |
| PR 5 | T006-T007 | `feat/shared-content-presentation` |
| PR 6 | T008 | `feat/ui-rich-content-cards` |
| PR 7 | T009-T010 | `feat/command-popup-manager-workspace` |
| PR 8 | T011 | `feat/private-sensitive-defaults` |
| PR 9 | T012 | `feat/capture-rules-ignore-next` |
| PR 10 | T013-T014 | `feat/transforms-snippet-variables` |
| PR 11 | T015 | `feat/safe-import-export` |
| PR 12 | T016-T017 | `feat/mcp-safe-local-ai` |
| PR 13 | T018 | `feat/doctor-hyprland-setup` |
| PR 14 | T019 | `docs/product-demo-readme` |
| PR 15 | T020 | `release/command-center-rc` |

## Verification Command Reference

### Full

```bash
just verify
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

### UI

```bash
cargo test -p author-clipboard-ui-gtk
just ui-check
just ui-smoke
```

### Shared

```bash
cargo test -p author-clipboard-shared
```

### Daemon

```bash
cargo test -p author-clipboard-daemon
```

### MCP

```bash
cargo test -p author-clipboard-mcp
```

### CLI

```bash
cargo test -p author-clipboard-ctl
```

## Manual Smoke Checklist

Use fake data only.

- [ ] Empty DB popup.
- [ ] Empty DB manager.
- [ ] Daemon unavailable UI.
- [ ] Text item.
- [ ] URL item.
- [ ] Color item.
- [ ] JSON item.
- [ ] Code item.
- [ ] Image item.
- [ ] File URI item.
- [ ] Sensitive token item.
- [ ] Reveal secret and confirm auto-redact.
- [ ] Copy selected item.
- [ ] Quick-paste selected item.
- [ ] Pin/unpin.
- [ ] Star/unstar.
- [ ] Delete.
- [ ] Transform JSON pretty/minified.
- [ ] Create snippet.
- [ ] Snippet variable preview.
- [ ] Ignore-next-copy.
- [ ] App rule ignore.
- [ ] MCP search redacted.
- [ ] MCP sensitive get refusal without confirmation.
- [ ] Doctor healthy output.
- [ ] Hyprland config dry-run.
- [ ] README install command works.

## Status

| Task | Status | Priority | Notes |
|---|---|---:|---|
| T001 | Planned | P0 | Audit first; no architecture changes |
| T002 | Planned | P0 | UI source of truth |
| T003 | Planned | P0 | Correct ID selection |
| T004 | Planned | P0 | Model-backed/reuse-backed list |
| T005 | Planned | P0 | Refresh signaling |
| T006 | Planned | P1 | Content presentation |
| T007 | Planned | P1 | Shared item view model |
| T008 | Planned | P1 | Rich cards/preview |
| T009 | Planned | P1 | Popup action/grouping |
| T010 | Planned | P1 | Manager workspace |
| T011 | Planned | P0 | Secure sensitive defaults |
| T012 | Planned | P2 | Capture rules/ignore-next |
| T013 | Planned | P2 | Transformations |
| T014 | Planned | P2 | Snippet variables |
| T015 | Planned | P2 | Safe import/export |
| T016 | Planned | P1 | MCP safety |
| T017 | Planned | P1 | MCP docs |
| T018 | Planned | P2 | Doctor/config generator |
| T019 | Planned | P1 | README/demo |
| T020 | Planned | P0 | Release validation |

**Created**: 2026-07-11  
**Updated**: 2026-07-12  
**Status**: Proposed
