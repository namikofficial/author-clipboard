# UI Design — author-clipboard

> The unified GTK4 + libadwaita UI (feature 023). One widget tree
> powers the popup and the manager window; the external `wofi/rofi/
> fuzzel` picker shares the same filter enum and the same row
> formatter.

---

## Quick start

```bash
# Layer-shell popup (default for super+shift+v)
author-clipboard --popup

# Real manager window (default when launched from a terminal)
author-clipboard --manager

# Hyprland native picker (legacy CLI preserved for back-compat)
author-clipboard-hypr-picker

# External menu picker (wofi/rofi/fuzzel)
author-clipboard-ctl picker --menu auto
```

## Design tokens

26 design tokens live in `crates/ui-gtk/data/style.css`. The
same scale is also exposed as Rust-side constants in
`crates/ui-gtk/src/theme.rs` (`theme::spacing`, `theme::radius`,
`theme::motion`, `theme::focus`, `theme::font_size`).

### Colors

| Token | Value | Purpose |
|---|---|---|
| `--accent` | `@accent_bg_color` | Primary action color |
| `--accent-fg` | `@accent_fg_color` | Text on accent |
| `--surface-0` | `@window_bg_color` | Window background |
| `--surface-1` | `@card_bg_color` | Cards (item rows, chips) |
| `--surface-2` | `@view_bg_color` | Search entry background |
| `--text-0` | `@window_fg_color` | Primary text |
| `--text-1` | `@view_fg_color` | Secondary text |
| `--text-2` | `@dim_label_fg_color` | Tertiary / meta text |
| `--border` | `@borders_color` | 1px dividers |
| `--danger` | `@error_bg_color` | Sensitive red border |
| `--success` | `@success_bg_color` | Pinned chip |
| `--warning` | `@warning_bg_color` | Starred chip |

### Spacing

| Token | Value | Purpose |
|---|---|---|
| `--space-2xs` | `2px` | Hairline gap (title ↔ subtitle) |
| `--space-xs` | `4px` | Tight icon gap, pill padding |
| `--space-sm` | `6px` | Chip vertical padding |
| `--space-md` | `8px` | Base gap between siblings |
| `--space-lg` | `12px` | List row vertical, sidebar row gap |
| `--space-xl` | `16px` | Content area padding |
| `--space-2xl` | `24px` | Section break |

### Radii

| Token | Value | Purpose |
|---|---|---|
| `--radius-sm` | `6px` | Small chips |
| `--radius-md` | `12px` | Item rows |
| `--radius-lg` | `16px` | Cards |
| `--radius-pill` | `999px` | Pill chips, search entry |

### Shadows

| Token | Value | Purpose |
|---|---|---|
| `--shadow-sm` | `0 1px 2px alpha(@window_fg_color, 0.06)` | Hover |
| `--shadow-md` | `0 4px 12px alpha(@window_fg_color, 0.10)` | Selected |
| `--shadow-lg` | `0 8px 24px alpha(@window_fg_color, 0.14)` | Toast, menus |

### Focus ring

| Token | Value | Purpose |
|---|---|---|
| `--focus-ring-width` | `2px` | Focus halo stroke |
| `--focus-ring-offset` | `2px` | Focus halo distance from edge |
| `--focus-ring-color` | `alpha(@accent_bg_color, 0.45)` | Focus halo color |

### Motion

| Token | Value | Purpose |
|---|---|---|
| `--motion-fast` | `120ms` | Hover/select |
| `--motion-base` | `200ms` | Modal / toast |
| `--motion-slow` | `320ms` | Page transition |
| `--ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | Default |
| `--ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | Pin/star bounce |

The color tokens are wired to libadwaita's `@*` aliases, so light + dark
modes work out of the box via `AdwStyleManager::default()`.

## Keyboard shortcuts (canonical table)

| Action | Shortcut |
|---|---|
| Open search | `/` |
| Move selection | ↑ ↓ ← → (←/→ in grids only) |
| Jump to first / last | `Home` / `End` |
| Page jump | `PgUp` / `PgDn` |
| Copy selected (close popup) | `Enter` |
| Quick-paste selected | `Ctrl+Enter` |
| Toggle pin | `Ctrl+P` |
| Delete | `Delete` or `Ctrl+D` |
| Toggle star | `Ctrl+S` |
| Clear search / blur input | `Esc` (first press) |
| Close window | `Esc` (second press, or any time list is focused) |
| Next / previous tab | `Ctrl+Tab` / `Ctrl+Shift+Tab` |
| Quick-pick #1-9 | `Ctrl+1` … `Ctrl+9` |
| Show shortcuts | `?` |

These are the same in popup and manager mode. The popup inherits
the manager's focus controller (see `crates/ui-gtk/src/window/
popup.rs`); the manager uses `controller::focus::resolve_escape`
directly.

## Bug fixes (US-001 / US-002 / US-003)

* **US-001 — Esc always closes:** A `Capture`-phase
  `EventControllerKey` is attached to the popup window. It runs
  before the search entry's built-in Esc handler, so we always
  get a chance to act. The decision is in
  `controller::focus::resolve_escape` (4 unit tests cover the
  full state machine).
* **US-002 — Search doesn't steal focus:** The popup calls
  `set_focus_widget(list_box)` via `connect_map` after the window
  is mapped. The `/` key is captured globally and focuses the
  search only when something else has focus.
* **US-003 — CLI launches a real window:** `window::manager`
  builds an `AdwApplicationWindow` with `AdwHeaderBar`,
  `AdwOverlaySplitView` (sidebar with 6 pages + `gtk::Stack`
  content), status bar, and toast overlay. Size persisted via
  `GSettings`. No more 520×700 libcosmic pane.

## Widget catalog

* `widgets::ItemRow` — one row that renders text / image / html /
  files. Sensitive items show a red left border and a `🔒 redacted`
  chip. The full content is **never** rendered in the list.
* `widgets::FilterBar` — 7 chips (All / Text / Images / Files /
  Pinned / Starred / Sensitive) in a `FlowBox`. Click a chip to
  filter; the active chip has the `chip-active` CSS class.
* `widgets::SearchEntry2` — search entry with 150ms debounce. The
  Esc handler clears the query and blurs back to the list.
* `widgets::Chip` — small pill with `chip-default` /
  `chip-danger` / `chip-success` / `chip-warning` / `chip-muted`
  CSS classes.
* `widgets::PickerGrid` — `FlowBox` of buttons for emoji / symbol
  / kaomoji pickers.
* `widgets::EmptyState` — `AdwStatusPage` with custom illustrations
  for "no items", "no results", "no sensitive", "daemon down".
* `widgets::Toast` — `AdwToast` wrapper for transient feedback
  ("Copied to clipboard").

## Pages (manager sidebar)

* **Clipboard** — search + filter + list, backed by IPC. Press
  `Enter` to copy; `Delete` to remove; `Ctrl+P` to pin.
* **Emoji** — `FlowBox` of emoji with category chips. Click to
  copy.
* **Symbols** — `FlowBox` of Unicode symbols.
* **Kaomoji** — vertical list of kaomoji.
* **Snippets** — add / list / delete from the snippet DB.
* **Settings** — `AdwPreferencesGroup` rows: incognito, clear-on-
  lock, encrypt, max items, TTL, clear data, about.

## IPC contract

The new UI consumes the same `IpcCommand::History`, `Pin`, `Unpin`,
`Delete`, `Copy`, `ToggleStar`, `ClearUnpinned`, `ListSnippets`,
`UpsertSnippet`, `DeleteSnippet`, `Status` commands the previous
applet used. No new IPC commands were required.

## External picker parity

`author-clipboard-ctl picker --filter <chip>` mirrors the GTK4
filter bar via `shared::picker::PickerFilter`. Both surfaces read
from `picker::apply_filter` so the semantics are identical.

## CSS

The stylesheet is `crates/ui-gtk/data/style.css`. It's compiled
into the GResource bundle at build time and loaded automatically
on app startup via `AdwStyleManager::default()`.

## Testing

```bash
cargo test --workspace      # 149 tests pass
```

Unit tests cover:

* `controller::focus::resolve_escape` — all 4 focus × search
  combinations
* `widgets::item_row::truncate` — short, long, newline
* `widgets::item_row::entry_to_item_preserves_sensitive`
* `shared::picker::apply_filter` — one test per `PickerFilter`
* `shared::picker::PickerFilter::display_round_trip`
* `settings::schema_id_is_stable`

## Visual audit (T001 — 2026-06-15)

Baseline screenshots captured via `just ui-smoke` live in
`docs/UI/snapshots/`. The audit below lists the cohesion gaps
that the T002 token pass (and downstream T003–T005 layout / state
polish) need to address.

### Findings

1. **No spacing scale.** All widget paddings and margins are
   hardcoded pixel values in Rust (`padding: 10px 14px`,
   `margin: 2px 0`, `margin_top(4)`, etc.) with no shared tokens.
   This makes the popup and the manager feel related by accident
   rather than by design. T002 introduces a `--space-{2xs,xs,sm,md,lg,xl,2xl}`
   scale (7 steps on a 2-4-6-8-12-16-24 px rhythm).
2. **No focus-ring token.** The search entry's focus halo
   (`0 0 0 2px alpha(@accent_bg_color, 0.15)`) is inline in
   `style.css:134`. Other focusable widgets (chips, sidebar rows,
   item rows) have no shared focus style. T002 adds
   `--focus-ring-width/offset/color` tokens plus a shared
   `:focus` rule so the whole app uses one ring.
3. **Global transition is incomplete.** The `*` transition only
   covers `background-color`, `border-color`, and `box-shadow`.
   `color` and `opacity` are not transitioned, which is why chips
   and rows feel "snappy" on hover but static on selection. T002
   widens the transition set to include `color` and `opacity`.
4. **Shadows break in dark mode.** `--shadow-sm` and `--shadow-md`
   use hardcoded `rgba(0, 0, 0, 0.06)` / `rgba(0, 0, 0, 0.10)`,
   which are invisible on a dark surface. T002 switches them to
   `alpha(@window_fg_color, …)` and adds a `--shadow-lg` for
   floating surfaces (toasts, menus).
5. **Empty / redacted overlays are placeholders.** `widgets::empty`
   is a 3-line stub (`#![allow(dead_code…)]`) and the redacted
   overlay reuses the same `adw::StatusPage` as the empty state,
   making them visually indistinguishable. Tracked in T005.
6. **Preview pane chrome is missing.** `PreviewPane` has a
   `.preview-pane` CSS class defined in the spec but no actual
   rules. The pane has no surface color, no padding, and no visible
   separation from the list. Tracked in T004.
7. **No pill-chrome on item rows.** Rows use `--radius-md` (12px)
   and a 2px margin. The cute pill feel from feature 023 (a
   6-8px gap and `--radius-pill` on hover) is missing. Tracked in
   T003.
8. **Theme.rs is a stub.** It just sets
   `ColorScheme::Default` and provides no Rust-side constants
   for the design tokens. Widgets that need a spacing value have
   to hardcode pixels. T002 adds `theme::spacing`, `theme::radius`,
   `theme::motion`, `theme::focus`, and `theme::font_size` modules
   with Rust-side constants and unit tests that lock the scales.
9. **Manager sidebar text is dense.** Sidebar rows use 8px
   margins with no visual selection state beyond the default
   list-box row style. Tracked in T004.
10. **Status bar uses ASCII dot.** The `● Daemon` indicator is
    a Unicode glyph; should be a small filled circle with a
    status color. Tracked in T005.

### Smoke-test notes

* `popup.png` and `popup-search.png` are byte-identical because
  `xdotool type "git"` runs faster than GTK paints the keystroke.
  The search focus ring *is* visible in `manager.png` (the
  cyan border around the search field), confirming focus styling
  works. A real session would show the search query text. The
  test is still useful as a layout / focus-state baseline.
* The smoke run also emits a long stream of
  `Gtk-WARNING: attempt to allocate … with width 0 and height -27`
  and `… with width -27898 and height 664`. These come from
  `AdwBin` wrappers inside `ItemRow` getting unclamped width
  requests during the empty-list path (no rows → no
  constraints → GTK speculatively tries negative sizes). Tracked
  for T003 item-row polish.

### T002 resolution

The token pass addresses findings 1, 2, 3, 4, and 8 above.
Compare the new `docs/UI/snapshots/manager.png` with the prior
baseline: the focus ring around the search field is now driven
by `--focus-ring-width/offset/color`, the filter chips wrap to
two rows using the spacing scale, and the list surface picks
up a subtle `--shadow-md` (visible at the top edge).

Implementation summary:

| File | Change |
|---|---|
| `crates/ui-gtk/data/style.css` | Added `--space-{2xs..2xl}`, `--focus-ring-*`, `--shadow-lg`; widened `*` transition to `color` + `opacity`; added shared `:focus` rule; replaced hardcoded focus-halo with the new token; switched shadows to `alpha(@window_fg_color, …)` for dark-mode parity |
| `crates/ui-gtk/src/theme.rs` | Replaced the stub with five `pub mod` token modules (`spacing`, `radius`, `motion`, `focus`, `font_size`), each documented and pinned by monotonicity tests |
| `docs/UI.md` | This audit section |

Verification: `cargo test -p author-clipboard-ui-gtk` → 79
passed, 14 ignored (GTK-init-only), 0 failed. `cargo clippy
-p author-clipboard-ui-gtk -- -D warnings` clean. `just ui-smoke`
regenerates the screenshots above.

### T003 resolution

T003 addresses the remaining popup-hierarchy polish items from
the audit (findings 5, 6, 7, and 9). It introduces:

* A sectioned shell for the popup: `.popup-section-{search,filter,list}`
  give every popup surface the same horizontal padding via
  `--space-xl` and vertical rhythm from the spacing scale.
* A real `widgets::EmptyState` that replaces the 3-line
  `empty.rs` stub. The clipboard page now switches between
  the list scroller and the empty state based on entry count,
  with the variant chosen from the current query / filter
  (`NoResults` when the user has typed something, `NoSensitive`
  for the sensitive filter, `NoItems` otherwise).
* A `.item-row-bin` class on the AdwBin wrapper to clamp the
  empty-list allocation, silencing the `Gtk-WARNING: attempt to
  allocate … with width -27898` stream that the audit recorded.
* A `.item-row-cluster` class for the trailing chip cluster
  on each item row, with explicit spacing and a left margin
  from the spacing scale.
* `.empty-state` rules that give the status page generous
  vertical padding so the illustration feels intentional
  rather than missing.

Implementation summary:

| File | Change |
|---|---|
| `crates/ui-gtk/data/style.css` | Added `.popup-shell`, `.popup-section-*`, `.popup-status`, `.empty-state*`, `.item-row-bin`, `.item-row-cluster`; bumped `.search-entry` and `.chip` `min-height` to align with the spacing scale; replaced hardcoded padding values with `var(--space-*)` |
| `crates/ui-gtk/src/widgets/empty.rs` | Replaced the stub with a real `EmptyState` widget supporting 4 variants (`NoItems`, `NoResults`, `NoSensitive`, `DaemonDown`); 3 unit tests pin the variant copy |
| `crates/ui-gtk/src/widgets/filter_bar.rs` | Spacing and margins now read from `theme::spacing::*` constants |
| `crates/ui-gtk/src/widgets/item_row.rs` | Cached the inner `adw::Bin` frame so `bind` no longer traverses the widget tree; added `.item-row-bin` clamp; uses `theme::spacing::*` for cluster gaps |
| `crates/ui-gtk/src/widgets/search.rs` | Removed the 200px hardcoded `set_size_request` so the CSS `min-height: 36px` is the only sizing source |
| `crates/ui-gtk/src/window/popup.rs` | Shell container uses `.popup-shell`; status label moved to `.popup-status` so its top border and font-size come from CSS |
| `crates/ui-gtk/src/pages/clipboard.rs` | Sectioned shell (`search_section` → `filter_section` → `list_section`); empty state appended to `list_section` and toggled by the refresh closure |

Verification: `cargo test -p author-clipboard-ui-gtk` → 82
passed, 14 ignored, 0 failed. `cargo clippy -p
author-clipboard-ui-gtk --all-targets -- -D warnings` clean.
`cargo fmt --all -- --check` clean. `just ui-smoke` shows the
new empty state in `docs/UI/snapshots/{popup,manager,clipboard-page}.png`.

## Rollback

The previous libcosmic applet and the parallel GTK4 hypr-picker
are preserved at the `pre-023-ui-rewrite` git tag.

```bash
git checkout pre-023-ui-rewrite
# or, if the rewrite is on main:
git revert e4b821b
```

## Reference

* Spec: `specs/features/023-unified-gtk4-ui/00-brief.md` (and 9
  companion files)
* Polish pass: `specs/features/024-ui-cohesion-polish/`
* Decisions: `specs/features/023-unified-gtk4-ui/09-decisions.md`
* Source: `crates/ui-gtk/`
* Old code: `crates/applet/` (was 2995 LOC, now 152) and
  `crates/hypr-picker/` (was 737 LOC, now 94)
