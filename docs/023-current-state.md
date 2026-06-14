# 023 Unified GTK4 UI — Current State

Generated after PR 0 (audit/fix). Baseline for all subsequent PRs.

## Verification Status

- `just fmt`: ✅ clean
- `cargo clippy -D warnings`: ✅ clean (0 errors)
- `cargo test --all`: ✅ 138 tests pass (0 ignored, 0 failed)
- `cargo build --all`: ✅ all crates build

Logs: `docs/023-audit/clippy.log`, `test.log`, `build.log`

## Crate Structure

```
crates/
├── ui-gtk/           # GTK4 UI library (libcosmic-free)
│   └── src/
│       ├── lib.rs              # Entry: run_popup, run_manager
│       ├── app.rs              # Shared app setup
│       ├── model.rs            # AppState reducer (no GTK init needed)
│       ├── actions.rs          # Action enum + reducer
│       ├── theme.rs            # Theme setup
│       ├── controller/         # Keyboard/input controllers
│       │   ├── focus.rs        # Focus management, Esc handling
│       │   ├── key.rs          # Global key shortcuts
│       │   └── search.rs       # Search debouncing
│       ├── window/
│       │   ├── mod.rs
│       │   ├── popup.rs        # build_popup() → PopupWindow
│       │   └── manager.rs      # build_manager_window() → ManagerWindow
│       ├── pages/
│       │   ├── mod.rs
│       │   ├── clipboard.rs    # Main list page
│       │   ├── settings.rs     # Settings page
│       │   ├── emoji.rs        # Emoji picker page (stub)
│       │   ├── kaomoji.rs      # Kaomoji page (stub)
│       │   ├── snippets.rs     # Snippets page (stub)
│       │   └── symbols.rs      # Symbols page (stub)
│       └── widgets/
│           ├── mod.rs
│           ├── search.rs        # SearchBar with debounce
│           ├── filter_bar.rs    # FilterChip row
│           ├── item_row.rs      # ClipboardItem row
│           ├── preview.rs       # 3-line stub (→ PreviewPane in PR 5)
│           ├── picker_grid.rs   # Grid-based picker (unused in popup)
│           ├── chip.rs          # Filter chip widget
│           ├── empty.rs         # Empty state widget
│           ├── shortcuts_overlay.rs  # ? hotkey overlay
│           └── toast.rs         # Toast notifications
└── applet/           # Thin binary, CLI parsing only
```

## Key Entry Points

| Function | File:Line | Purpose |
|----------|-----------|---------|
| `run_popup` | `ui-gtk/src/lib.rs:~50` | Popup UI entry |
| `run_manager` | `ui-gtk/src/lib.rs:~60` | Manager UI entry |
| `build_popup` | `ui-gtk/src/window/popup.rs:40` | Popup window builder |
| `build_manager_window` | `ui-gtk/src/window/manager.rs:45` | Manager window builder |
| `build_clipboard_page` | `ui-gtk/src/pages/clipboard.rs:~50` | Clipboard list page |
| `build_settings_page` | `ui-gtk/src/pages/settings.rs:~40` | Settings page |

## Known Issues (addressed in PRs 1–7)

| ID | Issue | File | PR |
|----|-------|------|-----|
| B1 | `PopupConfig` not propagated to clipboard page builder | `pages/clipboard.rs` | 1 |
| B2 | `count=0` shows no items but "No items" empty state suppressed | `pages/clipboard.rs` | 1 |
| B3 | Image copy lacks MIME type → preview broken on re-paste | `pages/clipboard.rs`, `shared/src/ipc.rs` | 1 |
| B4 | Search debounce uses `Cell<f64>` instead of `RefCell<u64>` | `widgets/search.rs` | 1 |
| B5 | `filter_entries` name doesn't reflect dual filter+query role | `shared/src/picker.rs` | 2 |
| UI1 | `AdwNavigationView` + sidebar not wired to model state | `app.rs`, `window/manager.rs` | 3A/3B |
| UI2 | Manager page list doesn't reflect filter state | `pages/clipboard.rs` | 3A |
| P1 | Preview pane is a 3-line stub | `widgets/preview.rs` | 5 |
| W1 | WebKit preview opt-in behind `features = ["webview"]` | `widgets/preview.rs`, `Cargo.toml` | 5.5 |

## Architecture Decisions (D15–D21)

| ID | Decision | Status |
|----|----------|--------|
| D15 | Fall back to `gtk::Box` + `gtk::ListBox` if libadwaita sidebar primitives unavailable | Recorded |
| D16 | Add `mime: Option<String>` to `IpcCommand::Copy` for image MIME preservation | Pending PR 1 |
| D17 | Rename `filter_entries` → `filter_and_query` in `shared/src/picker.rs` | Pending PR 2 |
| D18 | PR 5 ships PreviewPane without WebKit; PR 5.5 adds optional webkit6 | Recorded |
| D19 | `ui-check` and `ui-smoke` are manual-only, not CI | Recorded |
| D20 | IPC changes are atomic per PR (all match arms + constructor + test) | Enforced |
| D21 | Reducer tests must not require GTK init | Enforced |

## GTK/libadwaita Versions in Use

Checked against `Cargo.lock` and pkg-config at build time.

## Pre-commit Status

- `cargo fmt -- --check`: ✅
- `cargo clippy -- -D warnings`: ✅
- Conventional commits: enforced via `commit-msg` hook

## Rollback Tag

`pre-023-ui-rewrite` — reset to this tag if subsequent PRs need rollback.