# Requirements: Unified GTK4 UI

> Functional and non-functional requirements for the unified GTK4
> UI rewrite. Bug fixes for Esc / focus / CLI-launch come first.

---

## Bug-Fix User Stories (must ship first)

### US-001: Esc always closes the popup
**As** a keyboard user
**I want** pressing Esc to close the popup regardless of focus
**So that** I can dismiss it instantly without clicking the list first

**Acceptance**:
- Given the popup is open and the search input has focus, when I press
  Esc, then the search clears, the list gets focus, and a second Esc
  closes the popup.
- Given the popup is open and the list has focus, when I press Esc, then
  the popup closes.
- Given the popup is open and any other widget has focus, when I press
  Esc, then the popup closes.

### US-002: Search does not steal focus on open
**As** a keyboard user
**I want** the popup to open with the list focused
**So that** I can immediately press Enter on the first item

**Acceptance**:
- Given the popup is opened with `super+shift+v`, when it appears, then
  the list has focus and the first item is the visual selection.
- Given I want to search, when I press `/`, then the search input gets
  focus.
- Given I want to search, when I click the search input, then it gets
  focus.
- Given the search input has focus and is empty, when I press Esc, then
  focus returns to the list (not the popup closing).

### US-003: CLI launches a real manager window
**As** a user who runs `author-clipboard` from the terminal
**I want** to see a proper window with headerbar, sidebar, and settings
**So that** the app doesn't look broken

**Acceptance**:
- Given I run `author-clipboard` from a terminal, when the window
  appears, then it is a normal `AdwApplicationWindow` with headerbar,
  sidebar, content area, status bar, and a settings page.
- Given I run `author-clipboard --popup`, when it appears, then it is
  the layer-shell popup.
- Given I click the `.desktop` file, when it launches, then the manager
  window opens.
- Given the manager window is open, when I close it, then the process
  exits cleanly.

## New-Architecture User Stories

### US-004: One UI library, two binaries
**As** a maintainer
**I want** one widget set powering popup and manager
**So that** adding a feature means editing one place

**Acceptance**:
- `crates/ui-gtk/src/` contains every widget, model, and action.
- `crates/applet/src/main.rs` and `crates/hypr-picker/src/main.rs` are
  each under 100 LOC of `main()` + `ui_gtk::run_popup(...)` /
  `ui_gtk::run_manager(...)` glue.
- No widget code lives in any binary crate.

### US-005: One keyboard shortcut table
**As** a user
**I want** the same shortcuts in popup and manager
**So that** I don't relearn

**Acceptance** (and the canonical shortcut table):

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

### US-006: Filter bar is one widget
**As** a user
**I want** the same `All / Text / Images / Files / Pinned / Sensitive`
chips in popup and manager and external
**So that** I don't relearn the filter UI

**Acceptance**:
- `ui_gtk::widgets::FilterBar` renders the chips with the cute pill
  design.
- `shared::picker::PickerFilter` is a single enum matching the chips.
- The external picker's `--filter` flag maps to the same enum.
- Filter state survives a popup→manager switch via `GSettings`.

### US-007: Sensitive content is always redacted in UI
**As** a user
**I want** to see a `🔒 redacted` chip on sensitive items in every UI
**So that** I never accidentally share a secret

**Acceptance**:
- `ui_gtk::widgets::ItemRow` always renders `redacted_preview` (never
  `content` or `plain_text`) for items with `sensitive == true`.
- A toggle `Ctrl+Shift+R` reveals the redacted content for 5 seconds,
  with a countdown chip, only in the manager window.
- Sensitive items in the external picker row show the same `🔒` prefix.

### US-008: Cute, branded visual identity
**As** a user
**I want** the app to feel hand-crafted
**So that** I enjoy using it

**Acceptance**:
- Custom CSS at `crates/ui-gtk/assets/style.css` defines:
  - 14 design tokens (`--accent`, `--surface-0/1/2`, `--text-0/1/2`,
    `--radius-sm/md/lg/pill`, `--shadow-sm/md`, `--motion-fast/base`).
  - Soft 12-16px border radii on cards.
  - 150ms ease-out transitions on hover/select.
  - Spring-easing bounce on pin/star toggle.
  - Custom scrollbar (8px wide, transparent track).
- Custom icon set at `crates/ui-gtk/assets/icons/` (24×24 symbolic
  SVGs) for: `clipboard`, `pin`, `star`, `lock`, `search`, `trash`,
  `image`, `code`, `files`, `link`, `emoji`, `kaomoji`, `symbol`,
  `snippet`, `gear`, `chevron-down`, `x`, `plus`, `copy`.
- A custom font fallback: `Inter` → `Cantarell` → system.
- Light + dark theme via `AdwStyleManager::default()`.

### US-009: Empty states are beautiful
**As** a user
**I want** the empty clipboard view to be charming
**So that** the app feels alive

**Acceptance**:
- No-history empty state shows a hand-drawn clipboard SVG, the
  message "Your clipboard is empty", and a "Copy something to get
  started" subtitle.
- No-results empty state shows a magnifying-glass SVG, the message
  "No matches for `foo`", and a "Clear search" button.
- All empty states use the same `AdwStatusPage` + custom illustration
  layout.

### US-010: Settings page is real
**As** a user
**I want** settings to be in a real sidebar page, not a tab
**So that** they don't get lost

**Acceptance**:
- The manager window has a sidebar with: `Clipboard`, `Emoji`,
  `Symbols`, `Kaomoji`, `Snippets`, `Settings`.
- The Settings page uses `AdwPreferencesWindow` patterns: groups
  (`General`, `Privacy`, `Storage`, `Data`, `About`), each row
  using `AdwActionRow` / `AdwSwitchRow` / `AdwComboRow`.

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|---|---|---|---|
| FR-001 | `ui-gtk` crate exists, exports `run_popup()` + `run_manager()` | Must | |
| FR-002 | New `author-clipboard` binary replaces old `applet` binary | Must | |
| FR-003 | `author-clipboard-hypr-picker` calls `ui_gtk::run_popup()` | Must | |
| FR-004 | `ctl picker` uses upgraded `shared::picker` filter enum | Must | |
| FR-005 | Pop-up size = 720×520, manager size = 1100×720 | Must | |
| FR-006 | Layer-shell anchors = top + left + right (popup) | Must | |
| FR-007 | All keyboard shortcuts from US-005 work in both modes | Must | |
| FR-008 | Custom CSS + icon set + design tokens | Must | |
| FR-009 | `GSettings` schema for filter + sort state | Must | |
| FR-010 | Sensitive redaction with timed reveal (manager only) | Must | |
| FR-011 | Empty states with custom illustrations | Must | |
| FR-012 | `AdwNavigationView` for the manager sidebar | Must | |
| FR-013 | `AdwPreferencesWindow` for the settings page | Must | |
| FR-014 | `AdwToast` for transient feedback (copied, deleted) | Must | |
| FR-015 | IPC `Status` command is unchanged | Must | |
| FR-016 | `just verify` passes; no clippy warnings | Must | |

## Non-Functional Requirements

| ID | Target | Notes |
|---|---|---|
| NFR-001 | Popup opens in < 150ms (cold) | measured with `time` |
| NFR-002 | List scrolls at 60fps with 1000 items | GTK4 recycles rows |
| NFR-003 | Manager first paint < 300ms (cold) | |
| NFR-004 | Memory < 80MB (manager, 1000 items) | |
| NFR-005 | A11y: every interactive widget is focusable, labelled | |

## Edge Cases

| Case | Handling |
|---|---|
| Very long text content | Truncate with "..." in row, full in preview |
| Very large image | Downscale thumbnail for list; full in preview |
| Corrupt HTML | Show raw HTML in preview |
| Unknown content type | Show generic icon and filename |
| 1000+ items | `gio::ListStore` + `SingleSelection` recycles rows |
| Daemon not running | Manager shows "Daemon Offline" chip, IPC calls fail gracefully |
| Encrypted content | Decrypt only at preview/copy boundary; `redacted_preview` always safe |
| Sensitive redaction | Always show `redacted_preview` in list; reveal needs explicit user action |
| Empty search query | Esc clears, returns to list with full data |
| GSettings unavailable | Fall back to in-memory state |

## Out of Scope

- New clipboard features.
- Search syntax (FTS5/LIKE) changes.
- Encryption at rest changes.
- Tray icon, notifications.
- Internationalization (English-only).
- Mobile / touchscreen layouts.
- Custom theme engine.

## Dependencies

- `gtk4` >= 4.10
- `libadwaita` >= 1.4
- `gtk4-layer-shell` >= 0.4
- `glib`, `gio`, `gdk-pixbuf`
- `sourceview5` (text preview)
- `webkit6` (HTML preview)
- `glib-build-tools` (build-time GResource)

---

**Last Updated**: 2026-06-12
