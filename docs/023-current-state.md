# 023 Unified GTK4 UI — Current State

Updated after the feature 023 implementation landed on `dev`.

## Verification Status

- `just verify`: ✅ green
- `cargo test --all`: ✅ green
- `cargo build --all`: ✅ green

Latest verification run:

- `author-clipboard-shared`: 158 tests passed
- `ui_gtk`: 73 tests passed, 14 GTK widget tests ignored because they require GTK init/display

## Crate Structure

```
crates/
├── ui-gtk/           # GTK4 UI library (libcosmic-free)
│   └── src/
│       ├── lib.rs              # Entry: run_popup, run_manager
│       ├── app.rs              # AppState, Action, Effect, reduce()
│       ├── actions.rs          # Action dispatch helpers
│       ├── model.rs            # Clipboard item model objects
│       ├── theme.rs            # Theme setup
│       ├── controller/         # Keyboard/input controllers
│       │   ├── focus.rs        # Focus management, Esc handling
│       │   ├── key.rs          # Global key shortcuts
│       │   └── search.rs       # Search debouncing
│       ├── window/
│       │   ├── mod.rs
│       │   ├── popup.rs        # Popup window builder
│       │   └── manager.rs      # Manager window builder
│       ├── pages/
│       │   ├── mod.rs
│       │   ├── clipboard.rs    # Main list page
│       │   ├── settings.rs     # Settings page
│       │   ├── emoji.rs        # Emoji picker page
│       │   ├── kaomoji.rs      # Kaomoji page
│       │   ├── snippets.rs     # Snippets page
│       │   └── symbols.rs      # Symbols page
│       └── widgets/
│           ├── mod.rs
│           ├── search.rs       # SearchEntry with debounce
│           ├── filter_bar.rs   # Filter chip row
│           ├── item_row.rs     # Clipboard item row
│           ├── preview.rs      # PreviewPane
│           ├── picker_grid.rs  # Grid-based picker helpers
│           ├── chip.rs         # Filter chip widget
│           ├── empty.rs        # Empty state widget
│           ├── shortcuts_overlay.rs  # ? hotkey overlay
│           └── toast.rs        # Toast notifications
└── applet/           # Thin binary, CLI parsing only
```

## Key Entry Points

| Function | File:Line | Purpose |
|----------|-----------|---------|
| `run_popup` | `ui-gtk/src/lib.rs` | Popup UI entry |
| `run_manager` | `ui-gtk/src/lib.rs` | Manager UI entry |
| `build_popup` | `ui-gtk/src/window/popup.rs` | Popup window builder |
| `build_manager_window` | `ui-gtk/src/window/manager.rs` | Manager window builder |
| `build_clipboard_page` | `ui-gtk/src/pages/clipboard.rs` | Clipboard list page |
| `build_settings_page` | `ui-gtk/src/pages/settings.rs` | Settings page |

## Completed Surface

- `PopupConfig` now reaches the clipboard page builder and the page uses the configured filter, query, and count.
- Image copy always uses `CopyMode::Copy` with MIME preserved in the IPC payload.
- Search debounce uses `Rc<RefCell<...>>` and has a pure unit test for the second-query-wins case.
- `shared::picker` threads `PickerFilter` through both internal filtering and external picker row generation.
- `AppState` / `Action` / `Effect` / `reduce()` exist and are covered by pure tests.
- `PreviewPane` renders text, images, files, and sensitive state; HTML preview is feature-gated behind `webview`.
- Popup and manager both use the global key controller and GSettings bindings.
- `justfile` includes `ui-check` and `ui-smoke` as manual-only recipes.

## Architecture Decisions (D15–D21)

| ID | Decision | Status |
|----|----------|--------|
| D15 | Fall back to `gtk::Box` + `gtk::ListBox` if libadwaita sidebar primitives unavailable | Recorded |
| D16 | Add `mime: Option<String>` to `IpcCommand::Copy` for image MIME preservation | Recorded |
| D17 | Rename `filter_entries` → `filter_and_query` in `shared/src/picker.rs` | Recorded |
| D18 | PR 5 ships PreviewPane without WebKit; PR 5.5 adds optional webkit6 | Recorded |
| D19 | `ui-check` and `ui-smoke` are manual-only, not CI | Recorded |
| D20 | IPC changes are atomic per PR (all match arms + constructor + test) | Enforced |
| D21 | Reducer tests must not require GTK init | Enforced |

## Remaining Partials

- `crates/applet/src/main.rs` is still 154 LOC, above the 100 LOC review target.
- `crates/hypr-picker/src/main.rs` is still 97 LOC, above the 50 LOC review target.
- The review checklist still has a few manual runtime / packaging boxes that need explicit confirmation.

## GTK/libadwaita Versions in Use

Checked against `Cargo.lock` and pkg-config at build time. Reconfirm before any future GTK API work; this snapshot is tied to the current `dev` checkout.

## Pre-commit Status

- `cargo fmt -- --check`: ✅
- `cargo clippy -- -D warnings`: ✅
- Conventional commits: enforced via `commit-msg` hook

## Rollback Tag

`pre-023-ui-rewrite` — reset to this tag if subsequent PRs need rollback.
