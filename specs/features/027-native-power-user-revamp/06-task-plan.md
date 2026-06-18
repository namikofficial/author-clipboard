# Task Plan: Native Power-User Revamp

> Atomic, independently verifiable tasks. This is intentionally staged so the
> app improves after every slice without mixing storage, IPC, and UI changes in
> one large patch.

---

## Task Graph

```text
P0 Stabilize native picker/install
  -> P1 Query/classification foundation
  -> P2 Inspector and action rail
  -> P3 Collections and saved filters
  -> P4 Developer workflows and snippets
  -> P5 Health/first-run/integrations
  -> P6 Performance, docs, screenshots, release gate
```

## P0: Native Picker And Install Baseline

### T001: Close lifecycle and native window mode
**Goal**: Picker opens as a resizable XDG utility by default, with optional
`--layer-shell`, close button, and reliable Esc close.

**Files**:
- `crates/ui-gtk/src/window/popup.rs`
- `crates/ui-gtk/src/controller/key.rs`
- `crates/hypr-picker/src/main.rs`
- `crates/ui-gtk/src/lib.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -p author-clipboard-hypr-picker
setsid -f author-clipboard-hypr-picker
# Manual: Esc closes window and no picker process remains.
```

### T002: Install assets parity
**Goal**: Fresh install includes picker desktop file, schemas, icon, service,
and binaries.

**Files**:
- `justfile`
- `data/com.namikofficial.author-clipboard.hypr-picker.desktop`
- packaging files as needed

**Verification**:
```bash
just install
gsettings list-keys com.namikofficial.author-clipboard.state
desktop-file-validate ~/.local/share/applications/com.namikofficial.author-clipboard.hypr-picker.desktop
```

### T003: Hyprland float rule guidance
**Goal**: Recommended rule for class
`com.namikofficial.author-clipboard.popup` is documented and emitted by helper
commands.

**Files**:
- `crates/ctl/src/main.rs`
- `docs/HYPRLAND.md`
- `README.md`

**Verification**:
```bash
author-clipboard-ctl hyprland-config | rg author-clipboard
```

## P1: Query And Classification Foundation

### T004: Add query parser
**Goal**: Parse developer filters and warnings without touching UI.

**Files**:
- `crates/shared/src/query.rs`
- `crates/shared/src/lib.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- query
```

### T005: Add content classification
**Goal**: Classify text, code, command, URL, path, JSON, SQL, image, files,
snippet, secret.

**Files**:
- `crates/shared/src/classify.rs`
- `crates/shared/src/picker.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- classify
```

### T006: Wire parsed search into picker loading
**Goal**: `type:`, `app:`, `project:`, `collection:` filters work in shared
picker load path where data exists.

**Files**:
- `crates/shared/src/picker.rs`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ctl/src/main.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- picker
cargo test -p author-clipboard-ctl
```

## P2: Inspector And Action Rail

### T007: Inspector pane widget
**Goal**: Add a reusable inspector widget for selected item metadata and preview.

**Files**:
- `crates/ui-gtk/src/widgets/preview.rs`
- `crates/ui-gtk/src/widgets/inspector.rs`
- `crates/ui-gtk/src/widgets/mod.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- preview
```

### T008: Responsive picker split layout
**Goal**: Use list-only compact layout on narrow width and list + inspector on
wide width.

**Files**:
- `crates/ui-gtk/src/window/popup.rs`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/data/style.css`

**Verification**:
```bash
cargo check -p author-clipboard-ui-gtk
just ui-smoke
```

### T009: Action rail and keyboard shortcuts
**Goal**: Copy, quick-paste, pin, star, delete, add-to-collection, reveal
available from keyboard and UI.

**Files**:
- `crates/ui-gtk/src/app.rs`
- `crates/ui-gtk/src/controller/key.rs`
- `crates/ui-gtk/src/widgets/inspector.rs`
- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/src/main.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -p author-clipboard-daemon
```

## P3: Collections And Saved Filters

### T010: Collections DB helpers
**Goal**: Add collections/memberships storage and tests.

**Files**:
- `crates/shared/src/db.rs`
- `crates/shared/src/types.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- collection
```

### T011: Collections IPC and CLI
**Goal**: Expose collection operations through daemon and ctl.

**Files**:
- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/src/main.rs`
- `crates/ctl/src/main.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ctl -p author-clipboard-daemon
author-clipboard-ctl collection --help
```

### T012: Collections UI
**Goal**: Add collection chooser, collection badges, and manager page.

**Files**:
- `crates/ui-gtk/src/pages/collections.rs`
- `crates/ui-gtk/src/pages/mod.rs`
- `crates/ui-gtk/src/window/manager.rs`
- `crates/ui-gtk/src/widgets/inspector.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk
```

### T013: Saved filters DB/CLI/UI
**Goal**: Save, list, apply, and delete named query presets.

**Files**:
- `crates/shared/src/db.rs`
- `crates/shared/src/types.rs`
- `crates/ctl/src/main.rs`
- `crates/ui-gtk/src/pages/saved_filters.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- saved_filter
cargo test -p author-clipboard-ctl
```

## P4: Developer Workflows And Snippets

### T014: Snippets in unified picker
**Goal**: Snippets render with previews and can copy/quick-paste from the same
picker shell as history.

**Files**:
- `crates/ui-gtk/src/pages/snippets.rs`
- `crates/shared/src/picker.rs`
- `crates/shared/src/template.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- template
cargo test -p author-clipboard-ui-gtk -- snippets
```

### T015: Developer metadata and project tags
**Goal**: Add manual project/tag fields first; defer automatic app detection
until a reliable source exists.

**Files**:
- `crates/shared/src/types.rs`
- `crates/shared/src/db.rs`
- `crates/ctl/src/main.rs`
- UI inspector/actions

**Verification**:
```bash
cargo test -p author-clipboard-shared -- tag
```

### T016: Import/export v2
**Goal**: Export/import history, snippets, collections, memberships, saved
filters with safe redaction defaults.

**Files**:
- `crates/shared/src/db.rs`
- `crates/ctl/src/main.rs`
- `docs/IMPORT_EXPORT.md`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- export
author-clipboard-ctl export /tmp/ac.json
author-clipboard-ctl import /tmp/ac.json --dry-run
```

## P5: Health, First-Run, Integrations

### T017: Health IPC and UI
**Goal**: Show daemon, compositor, schema, quick-paste, and dependency health in
UI and CLI.

**Files**:
- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/src/main.rs`
- `crates/ctl/src/main.rs`
- `crates/ui-gtk/src/pages/settings.rs`
- `crates/ui-gtk/src/window/popup.rs`

**Verification**:
```bash
author-clipboard-ctl health --json | jq .
cargo test -p author-clipboard-ctl
```

### T018: Bar/widget status stability
**Goal**: Preserve and document `status --json` while adding fields safely.

**Files**:
- `crates/ctl/src/main.rs`
- `contrib/waybar/clipboard.sh`
- `docs/HYPRLAND.md`

**Verification**:
```bash
author-clipboard-ctl status --json | jq .
contrib/waybar/clipboard.sh | jq .
```

## P6: Performance, Docs, Screenshots, Release Gate

### T019: Seeded large-history benchmark
**Goal**: Create a repeatable 5k-item local benchmark for launch/search/scroll.

**Files**:
- `crates/shared/tests/`
- `justfile`
- `docs/PERFORMANCE.md`

**Verification**:
```bash
just perf-seed
just perf-picker
```

### T020: Screenshots and docs
**Goal**: Refresh UI screenshots and docs after the revamp.

**Files**:
- `docs/UI.md`
- `docs/UI/snapshots/`
- `README.md`
- `docs/HYPRLAND.md`

**Verification**:
```bash
just ui-smoke
```

### T021: Full verification and release checklist
**Goal**: Green repo checks and updated review checklist.

**Files**:
- `specs/features/027-native-power-user-revamp/08-review-checklist.md`
- `CHANGELOG.md`

**Verification**:
```bash
just verify
```

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Partial | Close/native window behavior recently fixed locally; needs committed spec-linked implementation. |
| T002 | Partial | Local install recipe includes schema/desktop file; package parity still pending. |
| T003 | Partial | User dotfiles rule exists; upstream helper/docs still pending. |
| T004-T021 | Pending | Requires implementation slices after this spec. |

---

**Last Updated**: 2026-06-19
