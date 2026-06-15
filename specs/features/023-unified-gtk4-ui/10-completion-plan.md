# Plan: Complete the Unified GTK4 UI (feature 023) — post-review fixup

> Strategy: ship 8 sequenced PRs that finish the planned work for feature 023
> (GTK4 unification). PR 0 audits the branch before any code changes. PR 1
> fixes four visible regressions. PR 2 closes the shared-filter gap.
> PR 3A introduces the minimal `AppState` / `Action` / `Effect` reducer
> (clipboard / search / filter / page / selection only). PR 3B fills in the
> remaining reducer coverage (pin, star, delete, reveal, window, settings,
> snippets, daemon). PR 4 wires the global key controller and binds
> GSettings. PR 5 builds `PreviewPane` for text / image / files / sensitive
> state. PR 5.5 adds the optional HTML WebView behind a feature flag.
> PR 6 rewrites the manager with `AdwNavigationView` + sidebar. PR 7
> finishes CLI parity, the smoke script, and the docs.
>
> Each PR ends with `just verify` green. `ui-check` and `ui-smoke` are
> manual-only (not wired into CI). Decisions D11–D14 are recorded in
> `09-decisions.md` as deviations from the original spec are forced.
>
> Status update: PR0 through PR7 are landed on `dev`, `just verify` is
> green, and `docs/023-current-state.md` now reflects the finished state.
> This document is retained as the execution record and rollback map.

---

## Execution rules (apply to every PR)

- **Each PR must compile independently.** No "I'll fix the breakage in the
  next PR" — finish the slice or revert.
- Run `just verify` after every PR. If it fails, the PR is not done.
- **Do not invent GTK / libadwaita APIs.** If a desired widget or method
  does not exist in the current crate version, fall back to the simplest
  compiling GTK primitive (`gtk::Box`, `gtk::ListBox`, etc.) and record the
  decision in `09-decisions.md`.
- **Do not mark spec tasks ✅ until the PR actually completes them.** Tick
  the status table only at the end of the PR that ships the work.
- **Reducer tests must not require GTK init.** Pure data, pure key-map,
  pure search-debounce tests run in the standard `cargo test` invocation.
- **Widget construction tests may use `gtk::init()`** and must be `#[ignore]`-d
  or gated behind a feature when no display is available. Do not pretend a
  widget test runs "without GTK" if it actually constructs GTK widgets.
- **IPC changes are atomic.** Before touching `IpcCommand::Copy`, inspect
  every match arm and every constructor. Update daemon handling, `ctl`,
  `picker`, `ui-gtk`, all tests, and any serde fixtures in the same PR.
- **No surprise CI changes.** `ui-check` and `ui-smoke` are manual-only.
  CI stays at `just verify`.

---

## Diagnosis

The dev branch ships a working GTK4 launch path (US-001/US-002/US-003) and
a thin `ui-gtk` skeleton, but the 20-task plan is roughly 35% done. The
biggest functional gaps, in priority order, are:

1. `PopupConfig` is built and dropped on the floor — `build_popup` ignores
   it; the page hard-codes `PickerFilter::All, count=50` (clipboard.rs:117).
2. `PageState::default()` initializes `count: 0`, so after the first refresh
   all subsequent refreshes load 0 items (clipboard.rs:131).
3. `copy_via_ipc` uses `CopyMode::CopyPlainText` for `image/*` MIME types
   (clipboard.rs:247–251) — likely a copy/paste bug.
4. No `AppState` / `Action` / `Effect` / `reduce()` exists; the spec calls
   for ~40 Action variants and one test each (T004).
5. `settings.rs` is a thin wrapper; nothing in the UI reads from it or
   writes to it (T006 binding half-done).
6. `widgets/preview.rs` is a 3-line stub (T010).
7. `SearchEntry2` uses `Rc<Cell<String>>` for the pending query; spec
   prefers `Rc<RefCell<String>>` and the `.take()` leaves empty state
   behind (search.rs:20, 59).
8. `filter_entries` and `build_external_rows` ignore `PickerFilter` even
   though `apply_filter` exists (T018 half-done).
9. `justfile` has no `ui-check` or `ui-smoke` recipes (T006/T019).
10. Manager lacks `AdwNavigationView`, sidebar layout, persisted size,
    preview pane; Esc unconditionally closes the whole window
    (US-003 partial).

The branch has the right shape: a single `ui-gtk` library, a thin `applet`
and `hypr-picker` dispatcher, real clipboard + IPC wiring, and 23 SVG
icons. The fix is to **finish the planned work**, not to redesign.

---

## Locked decisions

| #   | Decision                                                                                              | Rationale                                                                                                                                                       |
| --- | ------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| D11 | UI always sends `CopyMode::Copy`; daemon decides restore path                                         | Fixes the image→plain-text bug; lets the daemon keep its MIME-aware restore logic.                                                                             |
| D12 | `PageState` defaults derived from `PopupConfig`, not hard-coded                                        | Eliminates the `count: 0` refresh regression.                                                                                                                  |
| D13 | `webkit6` is feature-gated behind `--features webview`                                                | Keeps CI green (the dev host lacks `webkitgtk-6.0-dev`); local builders opt in. Default PR 5 ships no WebKit; PR 5.5 adds it.                                |
| D14 | `ui-check` and `ui-smoke` are manual-only justfile recipes                                            | Xvfb + xdotool not in CI; recipes are for the maintainer's local box.                                                                                          |
| —   | Full `AdwNavigationView` + sidebar at >900 px in the manager                                          | Matches 04-ui-flow.md and 02-domain-model.md exactly; no `ViewSwitcher` fallback. If a libadwaita sidebar primitive is missing, fall back to `gtk::ListBox`. |
| —   | Add `mime: Option<String>` to `IpcCommand::Copy` (default `None`)                                      | Backwards compatible (Option); lets the UI pass the row's MIME. Update **every** match arm + constructor + test in the same PR.                              |
| —   | PR 3 split into 3A (foundation) and 3B (full coverage)                                                | ~900 LOC and 40+ tests in one PR is too much for an agent pass. Foundation ships first, full surface ships second.                                             |
| —   | PR 5 split: PR 5 = text/image/files preview; PR 5.5 = optional WebKit                                  | WebKit is a common local-build friction source; default path stays free of it.                                                                                 |
| —   | PR 0 (audit) runs first                                                                               | Specific line references in the plan will drift; an agent must verify they are still correct before patching.                                                  |

---

## PR 0 — Audit the current branch *(no behavior change)*

Establish ground truth before patching. The original plan has line
references like `clipboard.rs:117` and `clipboard.rs:131`; if the branch
drifts even slightly, an agent may patch the wrong thing.

### Tasks

- Run `just verify` from a clean tree. Capture the output to
  `docs/023-audit/just-verify.log`.
- Run `cargo test --workspace` and capture the output to
  `docs/023-audit/cargo-test.log`.
- Inspect current GTK / libadwaita / webkit crate versions from the
  workspace `Cargo.toml`. Record the major + minor version and the
  highest-available libadwaita sidebar / overlay / navigation primitive
  actually exposed by the Rust bindings.
- Inspect **every** `IpcCommand::Copy` constructor and **every** match
  arm. Produce a table: file, line, function, current call shape. This
  is the input list for PR 1's IPC change.
- Inspect the current GSettings schema: list all keys, types, defaults,
  and current consumers. Record in `docs/023-audit/gsettings.md`.
- Inspect all `Rc<Cell<…>>` and `Rc<RefCell<…>>` usages in `ui-gtk` for
  the search debounce path.
- Produce `docs/023-current-state.md` with the exact files / functions /
  line numbers the plan refers to, plus any drift from the
  plan's line numbers. If a referenced line moved, update the plan
  before PR 1 starts.

### Verify

```bash
just verify
cargo test --workspace
```

No code changes. Compile/test breakage discovered during the audit gets
fixed in a single tiny commit titled `fix(audit): <thing>` so the audit
PR is the only place we touch the tree.

### Rollback risk

None. The audit PR adds docs and log files; it does not change code.

---

## PR 1 — P0 bug-fix patch *(~+150 / −80 LOC)*

Stop shipping the four visible regressions: ignored `PopupConfig`,
`count=0` refresh, image→plain-text copy, brittle `Rc<Cell<String>>`
debounce.

### Files

- `crates/ui-gtk/src/pages/clipboard.rs`
  - `build()` takes `&PopupConfig`; `PageState` is initialized from it.
    Drop hard-coded `("", PickerFilter::All, 50)`.
  - `PageState::default()` no longer exists; use
    `PageState::from_config(&PopupConfig)`. `count` is `cfg.count.max(1)`;
    the page never refreshes with `count=0` unless a test asks for it
    explicitly.
  - `copy_via_ipc` always sends `CopyMode::Copy`; the MIME is passed in
    the IPC payload.
- `crates/shared/src/ipc.rs`
  - `IpcCommand::Copy { id, mode, mime: Option<String> }`. New field
    defaults to `None`. `serde` default makes old clients keep working.
  - **Update every match arm + constructor in the same PR.** This
    includes the daemon, `ctl`, `picker`, `ui-gtk`, and any test that
    constructs `IpcCommand::Copy`. Verify with `rg '\bCopy\s*\{' crates/`
    before committing.
- `crates/ui-gtk/src/widgets/search.rs`
  - Replace `Rc<Cell<String>>` pending-query with `Rc<RefCell<String>>`.
    Use `borrow()` in the timer to read the latest query and clear it
    via `replace(String::new())` after firing, not `.take()`.
  - Add unit test: schedule two debounces 50 ms apart, second wins,
    first is dropped. **This is a pure-logic test; no GTK init required.**
- `crates/ui-gtk/src/window/popup.rs`
  - Pass `&config` into `pages::clipboard::build`. Pre-fill the search
    entry text from `config.query`. Initialize `FilterBar` from
    `config.filter`.
- `specs/features/023-unified-gtk4-ui/09-decisions.md`
  - Add D11 (image MIME).
  - Add D12 (`PageState` defaults).

### Verify

```bash
rg -n 'IpcCommand::Copy' crates/    # confirm every caller was updated
cargo test -p author-clipboard-shared -- picker
cargo test -p author-clipboard-ui-gtk -- search clipboard
just verify
```

Manual: `just applet -- --popup --filter pinned --count 10 --query "git"`
opens with 10 items, filter=Pinned, query="git" pre-filled. Image paste →
Enter → image lands on clipboard, not text.

### Rollback risk

Low. Two IPC fields added with `Default` impl; old clients keep working.

---

## PR 2 — T018: thread `PickerFilter` through shared picker *(~+120 / −60 LOC)*

`filter_entries` and `build_external_rows` accept `PickerFilter` so all
three UIs (popup, manager, external) share one filter function.

### Files

- `crates/shared/src/picker.rs`
  - New `filter_and_query(entries, query, filter) -> Vec<PickerEntry>`.
    When `query.is_empty()` and `filter == All`, identity. Otherwise
    substring match **and** filter. Old `filter_entries` becomes a
    thin wrapper kept for the existing test only.
  - `build_external_rows(entries, filter, include_key_prefix)`. Applies
    `apply_filter` internally; callers stop double-filtering.
  - New tests: every `PickerFilter` × query combination, edge cases
    (empty query, All filter). **Pure data tests; no GTK init.**
- `crates/ctl/src/main.rs`
  - `run_external_picker` calls
    `build_external_rows(&entries, filter_enum, true)`; drop the
    standalone `apply_filter` call.
- `crates/ui-gtk/src/pages/clipboard.rs`
  - Use `picker::filter_and_query` in the load path.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T018 → ✅.

### Verify

```bash
cargo test -p author-clipboard-shared
```

(~16 new test cases pass.)

### Rollback risk

Low (additive; renames one public fn; all 4 callers updated atomically).

---

## PR 3A — T004 (foundation): minimal `AppState` + `Action` + `reduce()` *(~+400 LOC, ~12 tests)*

Pure-data state machine, no GTK deps in the reducer. The minimum
needed to drive the search/filter/selection/page logic in `clipboard.rs`
through a reducer. Pin/star/delete/reveal/window/snippets/daemon
actions ship in PR 3B.

### Files

- `crates/ui-gtk/src/app.rs` (rewrite, foundation slice)
  - `PageId` (`Clipboard | Emoji | Symbols | Kaomoji | Snippets | Settings`).
  - `AppMode { Popup, Manager }`.
  - `AppState` (plain Rust, **not** `glib::Properties` — that comes in
    PR 4). Fields needed for the foundation slice: `mode`,
    `active_page`, `filter`, `search_query`, `selected_index`, `focus`,
    `config`.
  - `Action` variants for the foundation slice:
    `QueryChanged(String)`, `QueryCleared`, `FilterChanged(PickerFilter)`,
    `PageChanged(PageId)`, `CyclePage(i32)`, `Focus(FocusTarget)`,
    `Select(Option<u32>)`, `MoveBy(i32)`, `MoveTo(usize)`,
    `MovePage(i32)`, `ConfigLoaded(PopupConfig)`,
    `ManagerConfigLoaded(ManagerConfig)`.
  - `Effect` variants for the foundation slice: `RefreshItems`,
    `PersistGSettings` (placeholder; the bindings are wired in PR 4).
  - `pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect>`
    — pure, no I/O, no GTK.
  - `#[cfg(test)]` with **one test per Action variant** (~12) plus
    invariants: `MoveBy` doesn't panic on empty selection, `QueryChanged("")`
    is equivalent to `QueryCleared`, `CyclePage(1)` wraps around, etc.
- `crates/ui-gtk/src/lib.rs`
  - Re-export `AppState`, `Action`, `Effect`, `reduce` (the foundation
    surface only — more types added in PR 3B).
- `specs/features/023-unified-gtk4-ui/07-test-plan.md` — tick the
  "app::reduce — All Action variants" row partially (PR 3B finishes it).
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T004 marked
  "foundation only" until PR 3B.

### Verify

```bash
cargo test -p author-clipboard-ui-gtk -- reduce
```

(~12 tests pass; no GTK init required.)

### Rollback risk

Low. The new `app` module is additive; existing call sites keep working
until later PRs migrate them.

---

## PR 3B — T004 (completion): full reducer coverage *(~+500 / −20 LOC, ~30 more tests)*

Add the actions and effects the foundation slice skipped. The reducer
now covers the full US-005 + US-007 + manager persistence + daemon
status surface.

### Files

- `crates/ui-gtk/src/app.rs` (extend)
  - `AppState` adds: `sort`, `show_redacted`, `reveal_countdown`,
    `daemon_running`, `incognito`, `items: Vec<ClipboardItem>`,
    `snippets: Vec<Snippet>`.
  - `Action` variants added: `CopyRequested`, `QuickPasteRequested`,
    `TogglePin(i64)`, `ToggleStar(i64)`, `Delete(i64)`, `RevealRedacted`,
    `HideRedacted`, `RevealTick`, `SetDaemonRunning(bool)`,
    `ItemsLoaded(Vec<ClipboardItem>)`, `SnippetsLoaded(Vec<Snippet>)`,
    `Toast(String)`, `Quit`, `IncognitoToggled(bool)`,
    `WindowResized(i32, i32)`, `WindowPageChanged(PageId)`.
  - `Effect` variants added: `CopyItem { id, mode, mime }`,
    `QuickPasteItem { id, mime }`, `PinItem`, `UnpinItem`, `StarItem`,
    `UnstarItem`, `DeleteItem`, `ClearUnpinned`, `RefreshSnippets`,
    `AddToast`, `PersistConfig`, `Quit`.
  - Extend `reduce` with the new handlers. Keep it pure.
  - `#[cfg(test)]` adds ~30 tests, one per new Action variant, plus
    invariants: `RevealTick` decrements `reveal_countdown` and emits
    `HideRedacted` at 0; debouncing belongs in the runtime, **not** in
    the reducer, so `Action::WindowResized` does not coalesce.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T004 → ✅.
- `specs/features/023-unified-gtk4-ui/07-test-plan.md` — finish
  ticking the "app::reduce — All Action variants" row.

### Verify

```bash
cargo test -p author-clipboard-ui-gtk -- reduce
```

(~40+ tests pass.)

### Rollback risk

Low. Pure additive.

---

## PR 4 — T005/T006: real Esc + global key controller + GSettings binding *(~+450 / −90 LOC)*

### Files

- `crates/ui-gtk/src/settings.rs` (rewrite)
  - Typed accessors for `filter`, `sort`, `last_page`, `popup_size`,
    `window_size` — all enums, not `String`.
  - `pub struct SettingsBinding { … }` with `Rc<RefCell<AppState>>` +
    `gio::Settings`; two-way binding via `changed` signals + explicit
    `set_*` writes from reducer effects.
  - Read on startup; write on `PersistGSettings` effect.
- `crates/ui-gtk/src/controller/focus.rs`
  - Real `install(window, state, effects_tx)`. Capture-phase controller.
    Maps `gdk::Key` + `gdk::ModifierType` → `Action` via
    `key::map_key_extended`, dispatches through `reduce`, forwards
    `Effect`s to the runtime.
  - Cover the full US-005 shortcut table. **Unit test the resolver, not
    the GTK controller** — the resolver is pure and runs without GTK.
- `crates/ui-gtk/src/controller/key.rs`
  - `map_key_extended(key, mods)` returning the 13-row shortcut set.
    Add tests for every shortcut. **Pure mapping; no GTK init.**
- `crates/ui-gtk/src/controller/search.rs` (rewrite from 6 lines)
  - Owns `Rc<RefCell<SearchDebounce>>` where
    `SearchDebounce { pending: String, last_change: Instant, source: Option<glib::SourceId> }`.
    Unit test the debounce replacement logic **without** GLib timers
    (factor the timer call out so the test can pass a fake clock).
- `crates/ui-gtk/src/actions.rs`
  - `register(app, state, effects_tx)`: `set-filter`, `set-search`,
    `set-page`, `toggle-pin`, `delete`, `toggle-star`, `reveal`,
    `quit`, `prev-page`, `next-page`.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T005 + T006 → ✅.

### Verify

```bash
cargo test -p author-clipboard-ui-gtk -- key focus search
```

(Pure data + resolver tests; no GTK main loop.) Manual:
`gsettings set com.namikofficial.author-clipboard.state filter pinned`
while manager is open → chip animates to Pinned.

### Rollback risk

Medium. The new key controller is additive; the old inline Esc handler
in `popup.rs` stays until PR 6. New GSettings binding reads new keys; if
the schema is missing it falls back to `AppState` defaults (already
supported by `Settings::new() -> Option`).

---

## PR 5 — T010 (no WebKit): `PreviewPane` for text / image / files / sensitive *(~+450 LOC widget)*

Build the preview pane without WebKit. HTML content renders as escaped
text in the text view until PR 5.5 lands.

### Files

- `crates/ui-gtk/src/widgets/preview.rs` (rewrite)
  - `PreviewPane::new(state)`. Subscribes to `state.selected_index`
    and `state.items`.
  - `ContentType::Text` → `sourceview5::View` (read-only, monospace,
    soft-wrap).
  - `ContentType::Image` → `gtk::Picture` backed by
    `gdk_pixbuf::Pixbuf::from_file_at_scale` (max 800×600). Thumbnail
    if available.
  - `ContentType::Html` → escape and render as `sourceview5::View`
    with `language-html` highlighting. PR 5.5 swaps this for WebView.
  - `ContentType::Files` → list of `AdwActionRow`s with file name +
    size; click opens in default app via
    `gio::AppInfo::launch_default_for_uri`.
  - **Redaction**: when `sensitive && !show_redacted`, render an
    `AdwStatusPage` with `lock.svg`, the redacted preview, and a
    "Reveal (5s)" button. `Action::RevealRedacted` starts the
    countdown; `Action::RevealTick` decrements every second;
    `Action::HideRedacted` reverts. A `chip-warning` shows the
    remaining seconds.
  - Empty state: `AdwStatusPage` with `empty-clipboard.svg` and
    "Select an item to preview".
- **No `webkit6` dependency** in this PR. The default build stays
  WebKit-free (D13).
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T010 marked
  "no HTML preview" until PR 5.5.

### Verify

```bash
cargo test -p author-clipboard-ui-gtk -- preview
cargo test -p author-clipboard-ui-gtk -- reduce   # regression
```

Widget construction tests may use `gtk::init()` and must be
`#[ignore]`-d or feature-gated when no display is available. Do not
pretend a widget test runs "without GTK" if it constructs GTK widgets.

### Rollback risk

Low (new widget, no existing call site).

---

## PR 5.5 — T010 (WebKit): optional HTML preview behind `webview` feature *(~+80 LOC)*

Add the `ContentType::Html` branch with `webkit6::WebView` behind a
Cargo feature. Default build is unchanged and CI stays green.

### Files

- `crates/ui-gtk/Cargo.toml`
  - Add `[features] webview = ["dep:webkit6"]`.
  - Make `webkit6` an optional dependency gated by the feature.
- `crates/ui-gtk/src/widgets/preview.rs` (extend)
  - When the `webview` feature is on and content is HTML, render
    `webkit6::WebView` with `WebContext::set_sandbox_enabled(true)`.
    Load via `data:` URL.
  - When the feature is off, keep the sourceview fallback from PR 5.
  - Branch lives in a `#[cfg(feature = "webview")]` block; no
    unconditional imports of `webkit6::*`.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — D13 records
  that WebKit is feature-gated; PR 5.5 records the addition.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T010 → ✅.

### Verify

```bash
cargo build -p author-clipboard-ui-gtk                              # default, no webkit
cargo build -p author-clipboard-ui-gtk --features webview            # opt-in
cargo test -p author-clipboard-ui-gtk --features webview -- preview
```

**The default build must compile without `webkitgtk-6.0-dev`.** CI
runs the default; the maintainer runs `--features webview` locally.

### Rollback risk

Low. Feature is additive; default build is untouched.

---

## PR 6 — T013/T015: manager rewrite + persisted size + preview wiring *(~+400 / −110 LOC)*

The big UI rewrite. Layout is full `AdwNavigationView` + sidebar (no
`ViewSwitcher` fallback). When a libadwaita sidebar primitive is
unavailable in the current crate version, fall back to `gtk::Box` +
`gtk::ListBox` and record the decision in `09-decisions.md`. **Do not
invent APIs that do not exist in the current `libadwaita` crate
version.**

### Files

- `crates/ui-gtk/src/window/manager.rs` (rewrite)
  - `AdwApplicationWindow` with `AdwToolbarView`.
  - `AdwNavigationView` + 6 `AdwNavigationPage`s
    (Clipboard / Emoji / Symbols / Kaomoji / Snippets / Settings).
    Sidebar visible at widths > 900 px.
    - If the current libadwaita version exposes a sidebar primitive
      (e.g. `adw::Sidebar`, `adw::OverlaySplitView`), use it.
    - If it does not, build the sidebar from `gtk::Box` +
      `gtk::ListBox` with row icons. Record the choice in D15.
  - **Clipboard page** is the only one that mounts `PreviewPane` next
    to the list, in a `Paned` (60% / 40%). Below 900 px the preview
    collapses to a modal sheet.
  - **Persistence**: read `(window_width, window_height)` from
    GSettings on startup; on `close-request` and on `size-allocate`
    (debounced 500 ms via `glib::timeout_add_local`) write back.
    Read `last-page`; jump to it on startup.
  - **Esc**: same chain as the popup — search has focus →
    ClearSearch; modal open → close modal; list has focus → close
    window. Driven by `Action::Focus` / `Action::Quit` so the reducer
    is the source of truth.
  - **Status bar**: item count, pinned count, daemon indicator,
    reveal countdown chip when active.
  - **Toast overlay**: wrap the toolbar view; `Effect::AddToast`
    becomes `overlay.add_toast(adw::Toast::new(&msg))`.
- `crates/ui-gtk/src/window/popup.rs` (update)
  - Replace the inline `EventControllerKey` blob with the global key
    controller from PR 4.
  - Read popup size from GSettings; write back on `size-allocate`
    (debounced).
  - Initialize search text from `PopupConfig.query`. Initialize
    `FilterBar` from `PopupConfig.filter`.
  - Initialize `PageState` from `PopupConfig` (already done in PR 1).
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T013 + T015 → ✅.
- `specs/features/023-unified-gtk4-ui/08-review-checklist.md` —
  applicable rows ticked.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — D15 (which
  sidebar primitive was used, and why).

### Verify

```bash
cargo test -p author-clipboard-ui-gtk
just verify
```

Manual: open manager, resize, close, reopen at the same size; toggle
Pinned in popup, open manager, filter still Pinned; click into Settings,
change a row, close, reopen, value persists.

### Rollback risk

Medium. The manager is rewritten; old applet preserved at
`pre-023-ui-rewrite` tag.

---

## PR 7 — T017 + T019 + T020: CLI parity + smoke + docs *(~+250 / −20 LOC)*

Manual-only. `ui-check` and `ui-smoke` live in the justfile for the
maintainer's local box; **not** wired into CI.

### Files

- `crates/hypr-picker/src/main.rs`
  - Add `--filter` flag mapped to `PickerFilter`. Default `all`.
    Preserves legacy flags.
- `crates/ui-gtk/tests/smoke.sh`
  - Add scenarios: `/` + type, Esc-then-Esc close, Pinned filter
    persistence, manager opens to last page, sensitive reveal
    countdown shows and hides.
- `justfile` (add; **not** wired into `verify`)
  - `ui-check`: `glib-compile-schemas crates/ui-gtk/data/ && cargo
    check -p author-clipboard-ui-gtk`. Fails if the schema is stale.
  - `ui-smoke`: `xvfb-run -a crates/ui-gtk/tests/smoke.sh`. Saves
    screenshots to `docs/UI/snapshots/`.
  - `ui-test`: `cargo test -p author-clipboard-ui-gtk`.
- `docs/UI.md`
  - New "PreviewPane" section.
  - New "State machine" section with the reducer's Action table.
  - New "GSettings" section listing the schema IDs and the keys
    bound to `AppState`.
- `docs/UI/snapshots/` — commit 5 PNGs (popup, manager,
  clipboard-page, settings, sensitive-reveal).
- `README.md` — embed 2 inline screenshots.
- `specs/features/023-unified-gtk4-ui/08-review-checklist.md` —
  tick every applicable row.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — all 20
  rows → ✅.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — D14:
  hypr-picker extended with `--filter`; `ui-check` / `ui-smoke` are
  manual-only.

### Verify

```bash
just verify
```

Local: `just ui-check` then `just ui-smoke`.
`author-clipboard-hypr-picker --filter pinned` shows only pinned entries.

### Rollback risk

Low (additive CLI flag; smoke test gated; docs only).

---

## Sequencing & dependencies

```
PR0 (audit) ─┬─► PR1 (P0 bugs) ─► PR2 (T018) ─► PR3A (T004 foundation)
             │                                    │
             │                                    └─► PR3B (T004 completion)
             │                                              │
             │                                              └─► PR4 (T005/T006)
             │                                                        │
             │                                                        └─► PR5 (T010, no WebKit)
             │                                                                  │
             │                                                                  └─► PR5.5 (T010 WebKit, opt-in)
             │                                                                            │
             │                                                                            └─► PR6 (T013/T015)
             │                                                                                      │
             │                                                                                      └─► PR7 (T017/19/20)
```

- **PR 0 must run first.** It produces `docs/023-current-state.md`,
  which the agent uses to update line references in PRs 1–7.
- PR 1 should land before PR 3A so the `count` / `PopupConfig`
  defaults are sane before the reducer's invariants are pinned.
- PR 2 is independent and can be parallelized with PR 1 if reviewers
  prefer.
- PR 4 depends on PR 3A + 3B (it uses `Action` / `Effect`).
- PR 5 is independent of PR 4 (widget-only).
- PR 5.5 depends on PR 5.
- PR 6 depends on PR 4 + PR 5 (it wires them in).
- PR 7 is last (just CLI/docs/smoke).

---

## Cross-cutting rules

- **`just verify`** must be green on every PR. `ui-check` and `ui-smoke`
  are **not** part of verify; they require Xvfb.
- **Status table**: tick tasks only at the end of the PR that actually
  completes them. Don't tick T004 in PR 3A — wait for PR 3B.
- **Decisions log**: every deviation from the original spec is recorded
  in `09-decisions.md` with a one-paragraph rationale. This plan
  introduces D11–D15.
- **No libcosmic** in workspace (already true; `Cargo.toml` doesn't
  list it). `08-review-checklist.md` row "No libcosmic dep remains in
  the workspace" → ✅ today.
- **Rollback tag** `pre-023-ui-rewrite` already exists; it preserves the
  old libcosmic applet and the old GTK4 hypr-picker.
- **Performance** (NFR-001..005) is **out of scope**. The list still
  rebuilds on every refresh. If scroll is janky with 1000 items, follow
  up with a `gio::ListStore` + `gtk::SingleSelection` swap. Track as a
  future task; don't promise it here.

---

## Reviewer focus map

- **PR 0**: should produce a single commit adding audit docs and the
  IPC / GSettings / GTK-version inventories. If the PR modifies any
  source file, push back.
- **PR 1**: small, security-sensitive (the image-copy fix). Two
  reviewers: one for the IPC change (must run `rg 'IpcCommand::Copy'`
  on the diff), one for the UI plumbing.
- **PR 3A**: foundation. Reviewer should confirm no GTK init is
  required for the test suite, and that the chosen `AppState` fields
  match exactly the surface PR 4 needs.
- **PR 3B**: heaviest test surface. Reviewer should specifically
  check the test for `Action::RevealTick` (countdown logic) and
  `Action::WindowResized` (debouncing belongs in the runtime, not
  the reducer).
- **PR 5**: confirm that no `webkit6::*` import is unconditional and
  the default build doesn't pull WebKit in.
- **PR 6**: biggest UI change. Reviewer should open
  `docs/UI/snapshots/` after the maintainer runs `just ui-smoke`
  locally and sign off the visuals. Confirm D15 was filed if a
  fallback sidebar primitive was used.

---

## Out of scope (called out explicitly)

- `gio::ListStore` virtualization (NFR-002). Ship the working `ListBox`
  and follow up.
- `AdwComboRow` / `AdwSwitchRow` for the Settings page (currently
  `Switch` + `SpinButton`). The spec's `04-ui-flow.md` shows combo
  rows; this is a UI polish item, not a plan blocker. Track as PR 7.5.
- WebView sandbox E2E test (we just unit-test the `WebContext` setup).
- CI install of `webkitgtk-6.0-dev` (D13 defers that to whoever first
  needs `--features webview` in CI).
