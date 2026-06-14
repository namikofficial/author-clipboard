# Task Plan: Unified GTK4 UI

> 20 atomic tasks. Each ends with `just verify` green.

---

## Dependency Graph

```
T001 (deps) ──────────────────────────────────────────────────┐
                                                                │
T002 (ui-gtk skeleton) ─┬─ T003 (theme + icons)               │
                        ├─ T004 (state + reducer) ─────────────┤
                        ├─ T005 (focus ctrl) ──────────────────┤
                        └─ T006 (GSettings) ──────────────────┤
                                                                │
T007 (ItemRow) ────────────────────────────────────────────────┤
T008 (FilterBar) ──────────────────────────────────────────────┤
T009 (SearchEntry + debounce) ─────────────────────────────────┤
T010 (PreviewPane) ────────────────────────────────────────────┤
T011 (pages/clipboard) ────────────────────────────────────────┤
T012 (pages/{emoji,symbols,kaomoji,snippets}) ─────────────────┤
T013 (pages/settings) ─────────────────────────────────────────┤
T014 (window/popup) ───────────────────────────────────────────┤
T015 (window/manager) ─────────────────────────────────────────┤
                                                                │
T016 (slim applet) ─────────────────────────────────────────────┤
T017 (slim hypr-picker) ────────────────────────────────────────┤
T018 (update ctl picker filter) ───────────────────────────────┤
                                                                │
T019 (smoke + visual) ─────────────────────────────────────────┤
T020 (docs + screenshots) ─────────────────────────────────────┘
```

---

## T001 · Workspace GTK4 deps

**Goal**: Add GTK4, libadwaita, gtk4-layer-shell, glib, gio, sourceview5
to `[workspace.dependencies]`.

**Files**: `Cargo.toml`, `flake.nix`, `packaging/arch/PKGBUILD`,
`packaging/debian/control`.

**Implementation**:
- Add the 8 deps to `[workspace.dependencies]`.
- Bump `flake.nix` `inputs` and `pkgs` lists.
- Add `glib2-devel` (Arch) and `libglib2.0-dev-bin` (Debian) makedeps.

**Verification**:
```bash
cargo check -p author-clipboard-shared
just verify
```

**Rollback Risk**: Low — adding new code only

---

## T002 · `ui-gtk` crate skeleton

**Goal**: Empty crate with `lib.rs` exporting `pub fn run_popup(_: PopupConfig)`
and `pub fn run_manager(_: ManagerConfig)` (both `unimplemented!()`).

**Files**: `crates/ui-gtk/Cargo.toml`, `crates/ui-gtk/build.rs`,
`crates/ui-gtk/src/lib.rs`, `crates/ui-gtk/data/resources.gresource.xml`.

**Implementation**:
- Create the crate via `cargo new --lib crates/ui-gtk`.
- Write the public API.
- Empty `style.css` and `.ui` file referenced by `resources.gresource.xml`.
- `build.rs` calls `glib_build_tools::compile_resources`.

**Verification**:
```bash
cargo build -p author-clipboard-ui-gtk
```

**Rollback Risk**: Low

---

## T003 · Design tokens + icon set

**Goal**: `style.css` with the 14 design tokens; 22 SVGs in `assets/icons/`.

**Files**: `crates/ui-gtk/assets/style.css`,
`crates/ui-gtk/assets/icons/*.svg`.

**Implementation**:
- Design tokens block (colors, radii, shadows, motion) at the top
  of `style.css`.
- 22 symbolic SVGs, each < 1KB, in a 24×24 grid, 2px stroke.
- Each SVG authored by hand; see `docs/UI.md` for the geometry.

**Verification**:
```bash
glib-compile-schemas crates/ui-gtk/data/
xdg-open crates/ui-gtk/assets/icons/clipboard.svg   # eyeball test
```

**Rollback Risk**: Low

---

## T004 · `AppState` + `Action` + `reduce`

**Goal**: Pure data + a `reduce()` function with unit tests for every
Action variant. No GTK deps in this module (testability).

**Files**: `crates/ui-gtk/src/app.rs`.

**Implementation**:
- `pub struct AppState { … }` with `glib::Properties`.
- `pub enum Action { … }` (40+ variants).
- `pub enum Effect { … }`.
- `pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect>`.
- `#[cfg(test)] mod tests { … }` with one test per Action variant.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- reduce
```

**Rollback Risk**: Low

---

## T005 · `FocusChain` + Esc handler

**Goal**: Global `Capture`-phase Esc controller that implements the
US-001 semantics. Unit-tested with a mock `Focusable` trait.

**Files**: `crates/ui-gtk/src/controller/focus.rs`,
`crates/ui-gtk/src/controller/mod.rs`, `crates/ui-gtk/src/controller/key.rs`.

**Implementation**:
- `pub trait Focusable { fn search_has_focus(&self) -> bool; … }`.
- `pub fn install_esc_handler(window)`.
- A `FocusChain` enum: `List | Search | Modal | None`.
- `key.rs` wires `/`, `?`, `Ctrl+1..9`, `Ctrl+Tab` globally.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- focus
```

**Rollback Risk**: Low (new code, no behavior change in the applet
binary yet)

---

## T006 · GSettings schema + bindings

**Goal**: Compile the gschema, bind `filter`, `sort`, `last-page`,
`window-width`, `window-height` to `AppState` properties.

**Files**: `crates/ui-gtk/data/com.namikofficial.author-clipboard.gschema.xml`,
`crates/ui-gtk/src/settings.rs`, `justfile` (add `ui-check` recipe).

**Implementation**:
- Author the schema XML (see `02-domain-model.md`).
- `settings.rs` creates a `gio::Settings` instance and binds each
  key to a `glib::Property` on `AppState`.
- `just ui-check` runs `glib-compile-schemas` and validates.

**Verification**:
```bash
just ui-check
gsettings list-keys com.namikofficial.author-clipboard.state
```

**Rollback Risk**: Low

---

## T007 · `ItemRow` widget

**Goal**: One widget that renders text / image / html / files items,
with redacted preview for sensitive, hover/selected states, chip
meta line, action buttons (pin/star/delete).

**Files**: `crates/ui-gtk/src/widgets/item_row.rs`,
`crates/ui-gtk/src/widgets/chip.rs`, `crates/ui-gtk/src/model.rs`.

**Implementation**:
- `ItemRow` extends `gtk::ListBoxRow`.
- `Chip` is a small `gtk::Box` with CSS class `chip`.
- `model.rs` defines `ClipboardItemObject` (GObject).
- 4 content-type branches in `ItemRow::bind()`.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- item_row
```

**Rollback Risk**: Low (new widget only)

---

## T008 · `FilterBar` widget

**Goal**: 7 chips (All / Text / Images / Files / Pinned / Starred /
Sensitive) in a `gtk::FlowBox`. Emits `Action::SetFilter(PickerFilter)`.
CSS pill style.

**Files**: `crates/ui-gtk/src/widgets/filter_bar.rs`,
`crates/ui-gtk/src/widgets/mod.rs`.

**Implementation**:
- `FilterBar::new(state: &AppState) -> Self`.
- Each chip is a `gtk::ToggleButton` with CSS `chip` class.
- Syncs with `state.filter` via GSettings binding.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- filter_bar
```

**Rollback Risk**: Low

---

## T009 · `SearchEntry` + debounce

**Goal**: `SearchEntry` with 150ms debounce, `/` focus, Esc clearing.
Subclass of `gtk::SearchEntry` with the controller attached.

**Files**: `crates/ui-gtk/src/widgets/search.rs`,
`crates/ui-gtk/src/controller/search.rs`.

**Implementation**:
- `SearchEntry` is a `gtk::SearchEntry` with extra signals.
- `SearchController` holds the debounce source and the latest
  pending query.
- Emits `Action::SetSearch(String)` on debounce fire.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- search
```

**Rollback Risk**: Low

---

## T010 · `PreviewPane` (manager only)

**Goal**: Right pane in the manager showing the selected item. Uses
`sourceview5::View` for text, scaled `gdk_pixbuf::Pixbuf` for images,
file cards for files. Masked view for sensitive (Ctrl+Shift+R to
reveal, 5s countdown). **No WebKit in this PR** — HTML preview
ships in PR 5.5.

**Files**: `crates/ui-gtk/src/widgets/preview.rs`.

**Implementation**:
- `PreviewPane::new(state: &AppState) -> Self`.
- Subscribes to `state.model.selection-changed`.
- Reveal countdown uses `glib::timeout_add_seconds`.
- Text/Html: `sourceview5::View` (read-only, no syntax highlight).
- Images: `gdk_pixbuf::Pixbuf::from_file_at_scale` at 800×600.
- Files: `adw::ActionRow` list, click-to-open via `gio::AppInfo`.
- Sensitive: `adw::StatusPage` with lock icon + reveal button.
- Empty state: `adw::StatusPage` with clipboard icon.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- preview
```

**Rollback Risk**: Low

---

## T011 · `pages/clipboard`

**Goal**: Glue `SearchEntry` + `FilterBar` + `ItemRow` list into the
Clipboard page. Wires keyboard shortcuts from US-005.

**Files**: `crates/ui-gtk/src/pages/clipboard.rs`.

**Implementation**:
- `ClipboardPage::build(state) -> adw::NavigationPage`.
- Owns a `gtk::ListBox` bound to `state.model`.
- Wires the global key controller for the page.

**Verification**:
```bash
cargo build -p author-clipboard-ui-gtk
```

**Rollback Risk**: Low

---

## T012 · `pages/{emoji, symbols, kaomoji, snippets}`

**Goal**: Each is a `PickerGrid` (or `SnippetList` for snippets) with
category chips, recent row, search, and `Action::Copy` on Enter.

**Files**: `crates/ui-gtk/src/pages/emoji.rs`,
`symbols.rs`, `kaomoji.rs`, `snippets.rs`,
`crates/ui-gtk/src/widgets/picker_grid.rs`.

**Implementation**:
- `PickerGrid<T>` is a generic grid widget.
- Snippets adds an inline form (name + content) and a list.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- pages
```

**Rollback Risk**: Low

---

## T013 · `pages/settings`

**Goal**: `AdwPreferencesWindow` content with `General`, `Privacy`,
`Storage`, `Data`, `About` groups. Each row writes through to
`Config::save()`. 1:1 port of the existing 18 settings messages.

**Files**: `crates/ui-gtk/src/pages/settings.rs`.

**Implementation**:
- 5 `AdwPreferencesGroup`s, each with 2-6 rows.
- All rows call `Config::save()` on change.
- "About" group is read-only.

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- settings
```

**Rollback Risk**: Low

---

## T014 · `window/popup` + `window::build()`

**Goal**: `AdwWindow` with layer-shell init, SearchEntry on top, list
in the middle, hint footer, status chip in the corner. Fixed size
720×520, no decorations.

**Files**: `crates/ui-gtk/src/window/popup.rs`,
`crates/ui-gtk/src/window/mod.rs`,
`crates/ui-gtk/src/window/manager.rs`.

**Implementation**:
- `pub fn build_popup(app: &App, config: PopupConfig) -> adw::Window`.
- `gtk4_layer_shell::init_for_window(&window)`.
- `Layer::Overlay`, anchors top+left+right.
- `KeyboardMode::OnDemand`.

**Verification**:
```bash
cargo build -p author-clipboard-ui-gtk
xvfb-run target/debug/author-clipboard --popup &
sleep 1; xdotool key Escape
```

**Rollback Risk**: Medium — windowing changes

---

## T015 · `window/manager`

**Goal**: `AdwApplicationWindow` with `AdwHeaderBar`, `AdwNavigationView`,
sidebar with the 6 pages, main content area, status bar, AdwToast
overlay. 1100×720 default size persisted via GSettings.

**Files**: same as T014.

**Implementation**:
- `pub fn build_manager(app: &App) -> adw::ApplicationWindow`.
- `AdwNavigationView` + 6 `AdwNavigationPage`s.
- `AdwToastOverlay` wrapping content.
- `AdwStyleManager::default().set_color_scheme(ColorScheme::Default)`.

**Verification**:
```bash
xvfb-run target/debug/author-clipboard --manager &
# click around with xdotool
```

**Rollback Risk**: Medium

---

## T016 · Slim `applet` to 80 LOC

**Goal**: Replace 2995-LOC `applet/src/main.rs` with a clap-based
dispatcher that parses `--popup` / `--manager` and forwards to
`ui_gtk::run_popup()` / `ui_gtk::run_manager()`. The Cargo.toml
becomes 3 lines of deps.

**Files**: `crates/applet/Cargo.toml`, `crates/applet/src/main.rs`.

**Implementation**:
- `clap::Parser` with `Args` struct.
- `tracing_subscriber::fmt::init()`.
- `tokio::runtime::Runtime::new()` then `block_on` `ui_gtk::run_*`.

**Verification**:
```bash
cargo build -p author-clipboard-applet
just verify
```

**Rollback Risk**: **High** — deletes 2995 LOC. Old code preserved
at `git tag pre-023-ui-rewrite`.

---

## T017 · Slim `hypr-picker` to 40 LOC

**Goal**: Same pattern as T016. Existing CLI flags preserved for
backward compat with Hyprland keybinds.

**Files**: `crates/hypr-picker/Cargo.toml`,
`crates/hypr-picker/src/main.rs`.

**Implementation**:
- `clap::Parser` with the same `HyprPickerCli` struct as before.
- Convert into `PopupConfig` and call `ui_gtk::run_popup`.

**Verification**:
```bash
cargo build -p author-clipboard-hypr-picker
just verify
```

**Rollback Risk**: **High** — deletes 700 LOC. Old code preserved.

---

## T018 · Upgrade `shared::picker` + `ctl picker`

**Goal**: Add `PickerFilter` enum to `shared::picker`. Update
`filter_entries` to take it. Add `--filter` flag to `ctl picker`
subcommand.

**Files**: `crates/shared/src/picker.rs`,
`crates/ctl/src/main.rs`.

**Implementation**:
- `pub enum PickerFilter { … }` in `picker.rs`.
- `pub fn filter_entries(entries, query, filter) -> Vec<_>` updated.
- `pub fn build_external_rows(entries, filter, …) -> Vec<_>` updated.
- `ctl picker` subcommand gains `--filter`.

**Verification**:
```bash
cargo test -p author-clipboard-shared -- picker
cargo test -p author-clipboard-ctl
```

**Rollback Risk**: Low (additive)

---

## T019 · Smoke + visual tests

**Goal**: A `tests/smoke.sh` that launches both modes under Xvfb,
sends a few key presses, and saves a screenshot. CI runs the
test; the screenshot diffs against `docs/UI/snapshots/`.

**Files**: `crates/ui-gtk/tests/smoke.sh`,
`docs/UI/snapshots/*.png`.

**Implementation**:
- `xvfb-run -a target/debug/author-clipboard --popup &`
- `xdotool type "git"; sleep 0.3; import -window root snap.png`
- Repeat for `--manager`.
- `kill $PID`.

**Verification**:
```bash
just ui-smoke
```

**Rollback Risk**: Low

---

## T020 · Docs + screenshots

**Goal**: New `docs/UI.md` documenting design tokens, widget catalog,
keyboard shortcuts, and the popup/manager flows. Take 6 screenshots
(popup empty, popup with items, manager empty, manager with
selection, settings, sensitive reveal) and embed them in `README.md`.

**Files**: `docs/UI.md`, `README.md`, `docs/UI/*.png`.

**Implementation**:
- `docs/UI.md` with design tokens, shortcuts, and a widget catalog.
- 6 PNGs saved to `docs/UI/`.
- `README.md` updated with 2 inline images (popup, manager).

**Verification**:
```bash
grep -c "docs/UI" README.md   # ≥ 2
```

**Rollback Risk**: Low

---

## Rollback Risks Summary

- T002–T006: low (new code, no breakage)
- T007–T013: medium (state churn, but in a new crate)
- T014–T015: medium (windowing changes)
- T016–T017: **high** (deletes ~3700 LOC of existing UI). The old
  code is preserved at `git tag pre-023-ui-rewrite` so a `git revert`
  restores it instantly.
- T018: low (additive)
- T019–T020: low

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | | |
| T002 | | |
| T003 | | |
| T004 | ✅ (PR 3A + PR 3B) | AppState (plain Rust), Action (29 variants), Effect (14 variants), reduce() with 42 unit tests. No glib::Properties, no GTK deps in tests. |
| T005 | | |
| T006 | | |
| T007 | | |
| T008 | | |
| T009 | ✅ (PR 1) | RefCell swap; `clone_from` lint fix |
| T010 | ✅ (PR 5) | PreviewPane widget: text (sourceview), image (gdk-pixbuf), files (ActionRow), sensitive overlay. No WebKit — PR 5.5 adds WebView. |
| T011 | ✅ (PR 1) | `ClipboardPageProps` + `ClipboardCopyRequest`; `copy_via_ipc` always `CopyMode::Copy` + `mime`; image → PlainText branch dropped |
| T012 | | |
| T013 | | |
| T014 | ✅ (PR 1) | popup.rs builds `ClipboardPageProps` from `PopupConfig` (only `Option<String>→String` site); typed callback |
| T015 | ✅ (PR 1) | manager.rs uses `ClipboardPageProps::default()`; typed callback |
| T016 | | |
| T017 | | |
| T018 | ✅ (PR 2) | `filter_and_query` + `build_external_rows` takes `PickerFilter`; `ctl picker` uses new API |
| T019 | | |
| T020 | | |

---

**Last Updated**: 2026-06-15
