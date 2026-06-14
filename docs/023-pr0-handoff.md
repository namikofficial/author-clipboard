# PR 0 Handoff — Audit Complete

## What was done

- Fixed all clippy errors across all crates (correctness, suspicious, complexity, perf, style, pedantic warnings elevated to deny)
- Baseline verified: `just verify` passes (fmt → clippy → test → build)
- Created `docs/023-current-state.md` — ground truth inventory of files, entry points, known issues, and decisions
- Created `specs/features/023-unified-gtk4-ui/10-completion-plan.md` — the 9-PR execution plan
- Committed as `fa77a98` on `dev`

## Verification

```bash
just verify   # ✅ green
```

All 138 tests pass. All crates build. Zero clippy warnings.

## Current state

```
dev fa77a98 fix(audit): lint cleanup and branch audit
```

## What each subsequent PR does

| PR | Branch | Summary |
|----|--------|---------|
| 1 | `feat/023-popup-bugs` | PopupConfig propagation, count=0 fix, image MIME copy, search debounce Cell→RefCell |
| 2 | `feat/023-filter-rename` | Rename `filter_entries` → `filter_and_query` |
| 3A | `feat/023-app-foundation` | `app.rs` rewrite, `AdwNavigationView`, sidebar wiring, filter state |
| 3B | `feat/023-full-coverage` | Full UI integration, all match arms, tests |
| 4 | `feat/023-reducer-tests` | Reducer tests (no GTK init) for `AppState` |
| 5 | `feat/023-preview-pane` | PreviewPane without WebKit |
| 5.5 | `feat/023-webkit-opt-in` | Optional webkit6 behind `features = ["webview"]` |
| 6 | `feat/023-ci-wiring` | Wire `ui-check` / `ui-smoke` (manual only, not CI) |
| 7 | `feat/023-docs` | Update DEVGUIDE, LOCAL_TESTING, SPEC.md |

## Key files for next PR (PR 1)

### Must change together (atomic)

1. **`crates/shared/src/ipc.rs`** — add `mime: Option<String>` to `IpcCommand::Copy` (default `None`, backward compatible)
2. **`crates/ui-gtk/src/pages/clipboard.rs`** — pass `PopupConfig` through to clipboard page builder; fix `count=0` empty state; fix image MIME copy path
3. **`crates/ui-gtk/src/widgets/search.rs`** — change `Cell<f64>` debounce to `RefCell<u64>`

### Pattern for atomic IPC changes

Every `IpcCommand` variant addition needs updates in:
- `shared/src/ipc.rs` (variant + serializer + deserializer)
- `clipboard-daemon/src/ipc.rs` (server match arm)
- `ctl/src/main.rs` (client match arm)
- Any test files using the variant

## Critical constraints

- Reducer tests (`model.rs` tests) must **not** call any GTK API
- Widget tests (`pages/`, `widgets/`) may use `gtk::init()` and should be `#[ignore]` when no display
- IPC changes are **atomic** — update every match arm + constructor + test in the same PR
- Do **not** invent GTK/libadwaita APIs; fall back to `gtk::Box` + `gtk::ListBox`
- `ui-check` and `ui-smoke` stay **manual-only** (not wired into CI)

## Where to pick up

Start PR 1 by branching from `fa77a98`:

```bash
git checkout fa77a98
git checkout -b feat/023-popup-bugs
```

See `docs/023-current-state.md` for exact file:line references and `specs/features/023-unified-gtk4-ui/10-completion-plan.md` for the full task list.

## Rollback

```bash
git reset --hard pre-023-ui-rewrite
```

## Contacts / Context

- Plan: `specs/features/023-unified-gtk4-ui/10-completion-plan.md`
- Current state: `docs/023-current-state.md`
- Audit logs: `docs/023-audit/` (clippy.log, test.log, build.log)