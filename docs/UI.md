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

14 design tokens live in `crates/ui-gtk/assets/style.css`:

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
| `--radius-sm` | `6px` | Small chips |
| `--radius-md` | `12px` | Item rows |
| `--radius-lg` | `16px` | Cards |
| `--radius-pill` | `999px` | Pill chips |
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.06)` | Hover |
| `--shadow-md` | `0 4px 12px rgba(0,0,0,0.10)` | Selected |
| `--motion-fast` | `120ms` | Hover/select |
| `--motion-base` | `200ms` | Modal / toast |
| `--motion-slow` | `320ms` | Page transition |
| `--ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | Default |
| `--ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | Pin/star bounce |

The tokens are wired to libadwaita's `@*` aliases, so light + dark
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
  `AdwViewStack` of 6 pages, status bar, and toast overlay. No
  more 520×700 libcosmic pane.

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
* Decisions: `specs/features/023-unified-gtk4-ui/09-decisions.md`
* Source: `crates/ui-gtk/`
* Old code: `crates/applet/` (was 2995 LOC, now 152) and
  `crates/hypr-picker/` (was 737 LOC, now 97)
