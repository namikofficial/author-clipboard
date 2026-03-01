table of content with completed and pending and wip status

# author-clipboard — COSMIC-Native Clipboard Manager

> **Created by Namik** — An open-source, feature-rich clipboard manager built natively for COSMIC DE and other COSMIC-based desktops.
> Inspired by modern clipboard workflows, designed specifically for the Linux/COSMIC ecosystem.

**License:** GPL-3.0 | **Language:** Rust | **UI:** libcosmic (iced-based)

---

## Vision

The missing productivity tool for COSMIC desktop users. A comprehensive clipboard manager that delivers clipboard history, emoji picker, GIF search, and symbol/kaomoji input — all built natively for COSMIC DE in Rust with libcosmic.

**Not a fork.** This is an original project purpose-built for COSMIC's Wayland-native architecture, with a focus on comprehensive clipboard functionality including history, emoji, GIF search, and symbol input. Built using pure Rust + libcosmic instead of web technologies.

---

## Feature Comparison (vs. Reference Project)

| Feature | Other solutions (web-based) | author-clipboard (Rust+libcosmic) |
|---------|---------------------------|----------------------------------|
| Clipboard history (text) | ✅ | 🎯 Phase 1 |
| Rich text (HTML) | ✅ | 🎯 Phase 3 |
| Image clipboard | ✅ | 🎯 Phase 3 |
| Pin/unpin items | ✅ | 🎯 Phase 1 |
| Search/filter | ✅ | 🎯 Phase 1 |
| Emoji picker | ✅ | 🎯 Phase 4 |
| GIF search (Tenor) | ✅ | 🎯 Phase 4 |
| Kaomoji picker | ✅ | 🎯 Phase 4 |
| Symbol picker | ✅ | 🎯 Phase 4 |
| Smart positioning (follows cursor) | ✅ | 🎯 Phase 2 |
| Global shortcut (Super+V) | ✅ | �� Phase 2 |
| Paste injection (simulated Ctrl+V) | ✅ (uinput) | 🎯 Phase 5 |
| Theme integration | ✅ (custom) | 🎯 Native COSMIC theming |
| Settings UI | ✅ | 🎯 Phase 6 |
| Auto-cleanup / TTL | ✅ | 🎯 Phase 1 |
| Autostart (systemd) | ✅ | 🎯 Phase 2 |
| Setup wizard | ✅ | 🎯 Phase 6 |
| Shortcut conflict detection | ✅ | 🎯 Phase 2 |
| COSMIC DE native | ❌ | ✅ Core goal |
| Wayland-native (no X11 fallback) | Partial | ✅ Pure Wayland |
| Sensitive item detection | ❌ | 🎯 Phase 7 |
| Encryption at rest | ❌ | 🎯 Phase 7 |
| File/path clipboard | ❌ | 🎯 Phase 5 |

---

## Architecture

```
author-clipboard/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── clipboardd/               # Background daemon
│   │   └── src/
│   │       ├── main.rs           # Entry point, systemd service
│   │       ├── watcher.rs        # Wayland data-control clipboard watcher
│   │       ├── dedup.rs          # FNV hashing, deduplication logic
│   │       └── policy.rs         # TTL, size limits, sensitive detection
│   │
│   ├── applet/                   # COSMIC UI applet (the popup)
│   │   └── src/
│   │       ├── main.rs           # libcosmic Application entry
│   │       ├── app.rs            # Application state & messages
│   │       ├── views/
│   │       │   ├── clipboard_tab.rs   # History list view
│   │       │   ├── emoji_tab.rs       # Emoji picker view
│   │       │   ├── gif_tab.rs         # GIF search view (Tenor API)
│   │       │   ├── kaomoji_tab.rs     # Kaomoji picker view
│   │       │   ├── symbol_tab.rs      # Symbol/special char picker
│   │       │   └── settings_view.rs   # Inline settings panel
│   │       ├── widgets/
│   │       │   ├── tab_bar.rs         # Tab navigation
│   │       │   ├── search_bar.rs      # Unified search input
│   │       │   ├── clip_item.rs       # Single clipboard item widget
│   │       │   └── empty_state.rs     # Empty state placeholder
│   │       └── theme.rs              # COSMIC theme integration
│   │
│   └── shared/                   # Shared library
│       └── src/
│           ├── lib.rs
│           ├── types.rs          # ClipboardItem, ClipboardContent enum
│           ├── db.rs             # SQLite storage layer
│           ├── config.rs         # User configuration (COSMIC config)
│           ├── ipc.rs            # Daemon <-> Applet communication
│           └── emoji_data.rs     # Embedded emoji/kaomoji/symbol datasets
│
├── data/
│   ├── author-clipboard.desktop        # .desktop file
│   ├── author-clipboardd.service       # systemd user service
│   └── com.namik.clipboard.gschema.xml  # (optional) GSettings schema
│
├── resources/
│   ├── icons/                    # App icons (scalable SVG)
│   └── emoji/                    # Emoji data files (JSON)
│
└── docs/
    ├── CONTRIBUTING.md
    └── ARCHITECTURE.md
```

### Component Responsibilities

**1. Clipboard Daemon (`author-clipboardd`)**
- Runs as a systemd user service in the background
- Watches clipboard via Wayland `wlr-data-control` protocol (with abstraction layer for future `ext-data-control`)
- Deduplicates entries using stable FNV-1a hashing (not RandomState)
- Enforces policy: max items, max item size, TTL auto-cleanup, sensitive item detection
- Stores entries in SQLite via the shared crate
- Exposes IPC interface (Unix socket or D-Bus) for the applet to query/manipulate history

**2. COSMIC Applet (`author-clipboard`)**
- Tabbed popup UI with 5 tabs: Clipboard | Emoji | GIF | Kaomoji | Symbols
- Opens via global shortcut (Super+V), positions near cursor
- Search bar filters content within the active tab
- Clipboard tab: scrollable history list with pin/delete/clear actions, preview text/images
- Emoji tab: categorized, searchable emoji grid (embedded data, no network)
- GIF tab: Tenor API search with thumbnail grid (network required)
- Kaomoji tab: categorized Japanese emoticons
- Symbol tab: mathematical, currency, arrows, box drawing, Greek, Latin Extended, etc.
- Selecting any item copies to clipboard and optionally simulates paste
- Fully themed via COSMIC's native theming (dark/light mode automatic)

**3. Shared Library (`author-clipboard-shared`)**
- `ClipboardItem` / `ClipboardContent` types (Text, RichText, Image variants)
- SQLite database operations (insert, query, search, pin, delete, enforce limits)
- Configuration management (max items, cleanup interval, paste mode, GIF API key)
- IPC protocol definitions
- Embedded emoji/kaomoji/symbol datasets

---

## Technical Constraints & Solutions

### 1) Clipboard access on Wayland
Wayland clipboard is compositor-managed. Must use `wlr-data-control` protocol to observe and persist clipboard data.
- **Solution:** Implement watcher via `wayland-protocols-wlr` crate. Abstract protocol layer for future `ext-data-control` migration.
- **COSMIC note:** May require `COSMIC_DATA_CONTROL_ENABLED=1` env var.

### 2) Global shortcuts (Super+V)
COSMIC handles global shortcuts at the compositor level. App-level interception is flaky.
- **Solution:** Integrate with COSMIC's shortcut system / portal. Start with `Ctrl+Alt+V` if `Super+V` isn't bindable yet.

### 3) Paste injection
On Wayland, apps can't inject keystrokes into other apps without compositor support.
- **Solution:** Two modes:
  - **Safe mode (default):** Select item -> sets clipboard -> user presses Ctrl+V manually
  - **Quick paste (opt-in):** Virtual keyboard protocol or uinput-based simulation (feature-flagged, requires permissions)

### 4) Smart window positioning
Popup should appear near the text cursor or mouse pointer, respecting multi-monitor setups.
- **Solution:** Query pointer position via compositor, position layer-shell surface accordingly.

---

## Phased Development Plan

### Phase 0 — Prototype: Clipboard Watcher
**Goal:** Prove clipboard monitoring works on COSMIC.

- Connect to Wayland display, bind `wlr-data-control-manager`
- Listen for `selection` events on the data-control device
- Read `text/plain` MIME type from data offers
- Print captured clipboard text to stdout
- Verify on COSMIC (with `COSMIC_DATA_CONTROL_ENABLED=1` if needed)

**Deliverable:** Running daemon that prints clipboard changes to terminal.

---

### Phase 1 — MVP: Text History + Storage
**Goal:** Persistent, searchable text clipboard history with basic UI.

- **Database layer** (shared/db.rs): insert, query, search, pin, delete, enforce max items, dedup
- **Daemon -> DB**: On clipboard change -> hash -> check duplicate -> store
- **Auto-cleanup**: TTL-based expiry for non-pinned items (configurable interval)
- **Basic COSMIC applet**: Single-tab list view with:
  - Search bar (type to filter)
  - Scrollable item list (timestamp, preview text, pin icon)
  - Pin/unpin, delete, clear all actions
  - Keyboard navigation (up/down select, Enter to copy, Esc to close)
- **Paste behavior**: Selecting sets active clipboard; user pastes with Ctrl+V
- **IPC**: Daemon <-> Applet communication (Unix socket or D-Bus)

**Deliverable:** Working clipboard history with search, pin, delete.

---

### Phase 2 — Global Shortcut Experience: Shortcut + Positioning + Autostart
**Goal:** Press a key, popup appears near cursor, feels instant.

- **Global shortcut**: Register with COSMIC's shortcut system (Super+V or Ctrl+Alt+V)
- **Smart positioning**: Popup opens near mouse cursor, respects screen edges/multi-monitor
- **Focus handling**: Opens on top, Esc closes and returns focus to previous window
- **Autostart**: systemd user service for the daemon, auto-launch config
- **Shortcut conflict detection**: Warn if chosen shortcut conflicts with existing bindings
- **.desktop file & icons**: Proper desktop integration

**Deliverable:** Full global shortcut experience — press shortcut anywhere, picker appears, select & paste.

---

### Phase 3 — Rich Content: Images + HTML
**Goal:** Support image and rich text clipboard content.

- **Image support**: Detect `image/png`, `image/jpeg` MIME types
- **Image storage**: Store as base64 or files with DB references, thumbnails for UI
- **Image dedup**: Hash-based deduplication for images
- **Rich text**: Capture `text/html` alongside `text/plain`, store both
- **UI updates**: Image thumbnails in list, preview on hover/expand
- **Size limits**: Configurable max image size, eviction policy for large items

**Deliverable:** Copy images/HTML -> they appear in history -> select to re-copy.

---

### Phase 4 — Expression Pickers: Emoji, GIF, Kaomoji, Symbols
**Goal:** Tabbed UI with pickers for emoji, GIFs, kaomoji, and symbols (comprehensive expression input).

- **Tab bar**: Navigation between Clipboard | Emoji | GIF | Kaomoji | Symbols
- **Emoji picker**:
  - Embedded emoji dataset (Unicode 15.0+, no network needed)
  - Category navigation (Smileys, People, Animals, Food, Travel, etc.)
  - Search by name/keyword
  - Recently used section
  - Click to copy & paste
- **GIF picker**:
  - Tenor API search integration
  - Thumbnail grid with lazy loading
  - Trending GIFs default view
  - Click to download and copy as image/file URI for pasting into chat apps
  - Configurable API key (user provides their own or uses a default)
- **Kaomoji picker**:
  - Categorized list (happy, sad, angry, love, etc.)
  - Search by keyword
  - Click to copy & paste
- **Symbol picker**:
  - Categories: Math, Currency, Arrows, Box Drawing, Greek, Latin Extended, etc.
  - Search by name
  - Recently used
  - Click to copy & paste

**Deliverable:** Full tabbed experience with integrated expression pickers and clipboard history.

---

### Phase 5 — Quick Paste + File/Path Clipboard
**Goal:** Automatic paste injection and file clipboard support.

- **Quick paste mode (opt-in)**:
  - Virtual keyboard protocol or uinput-based keystroke simulation
  - Select item -> automatically pasted into focused app
  - Security warning on first enable, clear opt-in toggle
  - Permission checker for /dev/uinput access
- **File/path clipboard**:
  - Detect file list MIME types from Wayland data offers
  - Store file paths (references, not copies)
  - Display file names with icons
  - Re-copy file URIs on selection

**Deliverable:** One-click paste and file clipboard support.

---

### Phase 6 — Settings & Polish
**Goal:** User-configurable settings, setup wizard, final polish.

- **Settings panel** (within applet or separate window):
  - Max history size
  - Auto-cleanup interval
  - Paste mode (safe vs. quick)
  - Shortcut configuration
  - GIF API key
  - Theme (follows COSMIC / override)
  - Startup behavior
  - Data export/import
- **Setup wizard**: First-run experience — permissions check, shortcut setup, quick tour
- **Smart actions**: URL detection, color code preview, phone number formatting
- **Keyboard shortcuts within app**: Navigate tabs, items, actions without mouse
- **Accessibility**: Screen reader labels, high contrast support

**Deliverable:** Polished, configurable app ready for public release.

---

### Phase 7 — Security & Privacy
**Goal:** Enterprise-grade privacy controls.

- **Sensitive item detection**: Don't store passwords/OTP (detect `CLIPBOARD_STATE=sensitive`, password field hints)
- **Per-item expiration**: OTP auto-expire after configurable timeout (default 60s)
- **Encryption at rest**: SQLite encryption or encrypted file storage via OS keyring / libsodium
- **Clear on lock / logout**: Wipe non-pinned history on screen lock or session end
- **Incognito mode**: Temporarily pause clipboard history recording

**Deliverable:** Privacy-first clipboard manager suitable for security-conscious users.

---

## Implementation Notes

- **Language:** Rust (entire stack)
- **UI toolkit:** libcosmic (iced-based, COSMIC-native)
- **Background service:** Rust + wayland-client + wayland-protocols-wlr
- **Storage:** SQLite via `rusqlite` (bundled)
- **IPC:** Unix domain socket (simple JSON-over-socket) or D-Bus
- **Packaging:** `.deb` package, COSMIC applet registration, systemd user service
- **Future:** Flatpak if COSMIC portal support matures

### Key Dependencies
| Crate | Purpose |
|-------|---------|
| `libcosmic` | COSMIC UI framework |
| `wayland-client` | Wayland protocol client |
| `wayland-protocols-wlr` | wlr-data-control for clipboard |
| `rusqlite` | SQLite database |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `chrono` | Timestamps |
| `reqwest` | HTTP client (GIF API) |
| `image` | Image processing/thumbnails |
| `directories` | XDG directory resolution |

### Reference Projects
- [cosmic-utils/clipboard-manager](https://github.com/cosmic-utils/clipboard-manager) — Community COSMIC clipboard manager (UI patterns)
- [gustavosett/Windows-11-Clipboard-History-For-Linux](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux) — Similar project (Tauri+React)
- [Ringboard](https://github.com/alexanderpaolini/ringboard) — Wayland clipboard + virtual keyboard approach
- [pop-os/cosmic-applets](https://github.com/pop-os/cosmic-applets) — Official COSMIC applet examples
- [cliphist](https://github.com/sentriz/cliphist) — Simple Wayland clipboard history

---

## Definition of Done (v1.0 Release)

- Press Super+V -> picker opens instantly every time
- Type to filter clipboard history instantly
- Enter selects and copies to clipboard
- Tabbed interface: Clipboard / Emoji / GIF / Kaomoji / Symbols all working
- Image clipboard support with thumbnails
- No clipboard loss when source app closes (history persists in SQLite)
- COSMIC theme integration (dark/light automatic)
- Autostart on login
- Settings panel with essential options
- No broken "Super+letter" behavior on COSMIC
- Clean, open-source README with install instructions
