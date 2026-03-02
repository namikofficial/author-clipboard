## author-clipboard — Features Overview

> Native COSMIC desktop clipboard manager for Wayland. Free and open-source (GPL-3.0).

---

### Core Clipboard

- **Persistent clipboard history** — Text, images, files, and HTML survive app closures and reboots
- **Full-text search** — SQLite FTS5 index for instant search at scale, with LIKE fallback
- **Content deduplication** — Hash-based dedup with configurable `dedup_window_seconds`
- **Pin/favorite items** — Pinned items are preserved during cleanup
- **Auto-cleanup** — Configurable max items, TTL-based expiry, and size limits
- **Per-item TTL** — Set custom retention per item (e.g., 2 min for OTPs, 1 hour for tokens)
- **Image thumbnails** — 128px thumbnails generated via `image` crate
- **HTML + plain text dual storage** — Preserves formatting for rich content
- **File list clipboard** — Captures file paths with metadata

### User Interface

- **COSMIC native applet** — Built with `libcosmic`, follows COSMIC design language
- **Tab navigation** — Clipboard, Emoji, Symbols, Kaomoji, Settings tabs
- **Emoji picker** — Unicode 15.0+ with category-based organization and search
- **Symbol picker** — Math, Currency, Arrows, and more categories
- **Kaomoji picker** — Searchable database with compact layout
- **Recently used tracking** — Across all pickers
- **COSMIC symbolic icons** — Native icons for all buttons, content types, status indicators
- **Keyboard-driven workflow** — Full navigation: ↑↓, Home/End, PgUp/PgDn, Ctrl+1-9, Ctrl+D, Enter, Esc
- **Auto-scroll** — Follows keyboard selection
- **Smart auto-refresh** — Diff-based refresh preserves scroll position
- **Daemon status indicator** — Real-time connection status in settings and status bar
- **Empty state UX** — Centered icon with descriptive text

### Security & Privacy

- **Sensitive content detection** — Passwords, OTPs, JWT, API keys, SSH keys, AWS credentials, URI credentials, high-entropy secrets
- **Encryption at rest** — AES-256-GCM for sensitive items (opt-in via `encrypt_sensitive`)
- **Incognito mode** — Temporarily pause clipboard capture
- **Clear on screen lock** — Configurable via `clear_on_lock`
- **Screen lock detection** — Supports `loginctl` (systemd) and D-Bus `org.freedesktop.ScreenSaver`
- **IPC socket security** — Socket in `$XDG_RUNTIME_DIR`, never `/tmp`
- **Audit logging** — Security events recorded and trimmable
- **Threat model** — Documented in `SECURITY.md` with what is and isn't protected

### Tools & CLI

- **CLI control tool** (`author-clipboard-ctl`) — Toggle, show, hide, ping, history, status, clear, export, config
- **Global shortcut** — Super+V to open picker (IPC-based activation)
- **Quick paste mode** — Virtual keyboard integration (wtype/ydotool backends, opt-in)
- **Data export/import** — JSON format with optional encryption
- **File manager integration** — `xdg-open` for file paths

### Configuration

- **JSON config file** — `~/.config/author-clipboard/config.json`
- **Configurable options** — `max_items`, `max_size`, `ttl_seconds`, `cleanup_interval`, `db_path`, `dedup_window_seconds`, `encrypt_sensitive`, `clear_on_lock`
- **Settings tab** — In-app configuration display with stats and privacy toggle
- **COSMIC theming** — Follows system theme automatically

### Infrastructure

- **Systemd service** — `author-clipboard-daemon.service` with auto-restart
- **`just` commands** — `install`, `enable`, `disable`, `status`, `logs`, `uninstall`
- **GitHub Actions CI** — Format, clippy, test, build on every push and PR
- **CODEOWNERS** — Contribution area ownership in `.github/CODEOWNERS`
- **Crash-safe database** — SQLite WAL mode for concurrent reads and crash safety
- **Database migrations** — Automatic schema versioning and migration

### Planned Features

- [ ] "Never store" rules (MIME denylist, regex exclusion) — *Phase 15*
- [ ] Snippets & templates with token replacement — *Phase 15*
- [ ] OCR for images (Tesseract) — *Phase 15*
- [ ] Per-item hotkeys and paste macros — *Phase 15*
- [ ] X11 fallback clipboard monitoring — *Phase 16*
- [ ] Self-hosted E2EE sync — *Phase 17*
- [ ] `.deb`, Flatpak, Nix packaging — *Phase 18*
- [ ] Tenor GIF search (requires API key) — *Deferred*
- [ ] Shortcut configuration UI (requires COSMIC runtime) — *Deferred*

---

See `PROJECT_PLAN.md` for the full development roadmap and `docs/DEVELOPMENT.md` for build instructions.
