# Decisions: Unified GTK4 UI

---

## D1: One native code, GTK4 + libadwaita

**Chosen**: GTK4 + libadwaita for both popup and manager.

**Rejected**:
- *Keep libcosmic*: 2995 LOC of god-state, no easy path to layer-shell
  on Hyprland without porting to iced-runtime; weaker support for
  custom CSS theming.
- *One codebase, two toolkits (cosmic + gtk)*: doubles the surface
  area; defeats the purpose.
- *Web UI (Tauri / egui)*: breaks the "native at home" feel; bad
  IME support; Wayland layer-shell is awkward.

**Trade-offs accepted**:
- Lose libcosmic's automatic adaptiveness; we re-implement with
  libadwaita + `AdwStyleManager` (effectively the same thing).
- New dependency on `gtk4-layer-shell`; we already need it for the
  existing `hypr-picker` so no new packaging burden.
- COSMIC users will see a GTK4 popup that doesn't perfectly match
  the COSMIC theme; this is acceptable because the existing
  `hypr-picker` is already GTK4 and is the recommended path on
  wlroots; the libcosmic applet is only preferred on pure COSMIC.

## D2: One binary, two modes (popup / manager)

**Chosen**: `author-clipboard --popup | --manager` is the only
binary the user needs.

**Rejected**:
- *Separate `author-clipboard-manager` binary*: the popup and
  manager are 90% the same widget tree; splitting them would
  re-introduce drift.
- *Auto-detect*: the popup needs layer-shell; the manager needs
  XDG shell. Asking for the mode explicitly is clearer and
  matches the existing `--popup` flag from `hypr-picker`.

## D3: `ui-gtk` is a library, not a binary

**Chosen**: `crates/ui-gtk/` is a `lib` only; the binaries
(`applet`, `hypr-picker`) just call into it.

**Trade-offs**:
- Forces clear public API surface (`run_popup`, `run_manager`).
- Makes the UI unit-testable without running a GTK main loop
  for pure-data modules (`app.rs`, `controller/focus.rs`).
- The `applet` crate's binary name stays `author-clipboard` (so
  packaging doesn't change).

## D4: Cute / custom aesthetic, not pure-native

**Chosen**: Custom design tokens, custom SVG icon set, soft
radii, micro-animations.

**Trade-offs**:
- More design work up front.
- A11y: we have to remember to label everything; mitigated by
  using `AdwActionRow` / `AdwButtonContent` (built-in labels).
- Updates to GNOME / libadwaita theming may need CSS touch-ups.
- The custom SVG icon set takes ~3 hours of work to draw; it's
  worth it for the brand.

## D5: Big-bang rewrite, not phased

**Chosen**: Land the whole thing in one PR.

**Rejected**:
- *Phased*: incremental rewrites of 2995 LOC tend to grow the
  file in place and never converge.
- *Bug-fix only*: leaves the user with three slightly-broken UIs.

**Risk mitigation**:
- Old `applet` and `hypr-picker` are preserved at
  `git tag pre-023-ui-rewrite` (created at task start).
- Each task in `06-task-plan.md` is independently `just verify`-able.
- If blocked, we revert one task at a time (git revert the merge
  commit of that task's PR).

## D6: External picker (`ctl picker`) is unchanged in UX

**Chosen**: The `wofi/rofi/fuzzel` external picker keeps its
single-line row format. It gains a `--filter` flag.

**Why**: the external picker is invoked from a keybind, runs in
~30ms, and is the right tool for that moment. It would be a
regression to make it open a full GTK window.

## D7: Sensitive reveal is manager-only

**Chosen**: `Ctrl+Shift+R` only works in the manager window.

**Why**: the popup is a 200ms interaction; revealing redacted
content adds cognitive load. The manager is where you audit
and edit; reveal belongs there.

## D8: GSettings via dconf

**Chosen**: Use `GSettings` (dconf) for filter / sort / window
state persistence, not a custom JSON file.

**Why**: libadwaita expects it; it integrates with
`AdwComboRow` / `AdwSwitchRow` out of the box; existing
`Config` is for user preferences (max items, denylist), not
UI state. Clear separation of concerns.

## D9: `AdwNavigationView` for the manager sidebar

**Chosen**: Use `AdwNavigationView` with `AdwNavigationPage`s
for the 6 manager pages (Clipboard, Emoji, Symbols, Kaomoji,
Snippets, Settings). Each page is a top-level navigation page.

**Rejected**:
- *Tab bar like the old applet*: harder to add a 7th page, less
  discoverable, doesn't scale to mobile.
- *Single page with internal tabs*: loses the URL/deeplink story
  (`author-clipboard --source emoji` should open the emoji page).

**Trade-offs**:
- `AdwNavigationView` is "mobile-flavored" but works fine in
  desktop mode with the sidebar pattern.

## D10: Toast + 800ms close, not immediate exit

**Chosen**: After a successful copy, show `AdwToast` "Copied to
clipboard" for 800ms, then close the popup.

**Why**: the existing `std::process::exit(0)` is jarring. The
toast confirms the action and gives a moment of "this is the
right item" before the popup disappears.

**Trade-off**: the popup stays 800ms longer, but in exchange
the user gets visual feedback that the copy succeeded.

## D11: UI always sends CopyMode::Copy; daemon decides restore path

**Chosen**: `pages::clipboard::copy_via_ipc` always sends
`CopyMode::Copy` and passes the row's MIME via the new `mime`
field on `IpcCommand::Copy` (added with `#[serde(default)]` for
backwards compatibility).

**Why**: the previous code downgraded `image/*` to
`CopyPlainText` because the popup had no way to tell the daemon
which MIME to restore. The fix is to let the UI pass the MIME
explicitly and let the daemon's existing `pick_copy_mode` logic
decide the restore path. Old clients that don't send `mime` keep
working — the daemon falls back to its mode-based behaviour.

## D12: PageState defaults derived from ClipboardPageProps, not hard-coded

**Chosen**: `PageState::default()` is removed. The page constructs
its state from `ClipboardPageProps` via `PageState::from_props()`,
which clamps `count` to `>= 1` so a refresh never loads zero items.

**Why**: the previous `PageState::default()` had `count: 0`, which
meant the first refresh of a freshly built page loaded 50 items
(hard-coded), but every subsequent refresh — including the
200 ms post-build refresh — loaded 0 items. The fix is to
initialise the state once from the props and never let `count` be 0.

## D13: WebKit is feature-gated behind `webview` feature

**Chosen**: The `webview` feature in `crates/ui-gtk/Cargo.toml` gates
the `webkit6` dependency. Default build (`cargo build`) doesn't require
`webkitgtk-6.0-dev`. The maintainer runs `cargo build --features webview`
on their local box.

**Confirmed by**: PR 5.5. The `render_html_with_webview` function is
`#[cfg(feature = "webview")]`; the ContentType::Html arm falls through
to `sourceview5::View` when the feature is off.

---

**Last Updated**: 2026-06-15
