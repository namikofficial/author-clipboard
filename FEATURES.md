## author-clipboard - Features Overview

> Native COSMIC clipboard manager with wlroots compositor support, including Hyprland and Sway.

author-clipboard is free and open source under GPL-3.0. COSMIC is the primary UI target through `libcosmic`; Hyprland, Sway, and other wlroots compositors are supported through Wayland `wlr-data-control` when the compositor exposes it.

---

### Core Clipboard

- **Persistent clipboard history** - Stored in SQLite with WAL mode.
- **Full-text search** - SQLite FTS5 with LIKE fallback.
- **Content deduplication** - Hash-based dedup with configurable `dedup_window_seconds`.
- **Pin/favorite items** - Pinned items are preserved during cleanup.
- **Auto-cleanup** - Configurable max items, TTL expiry, and size limits.
- **Per-item TTL** - Database support for custom retention per item.
- **Image thumbnails** - 128px thumbnails generated via the `image` crate.
- **HTML + plain text indexing** - Captures HTML and stores plain text for search when offered.
- **File URI list capture** - Captures `text/uri-list` file copy events and parses file metadata.

### Content Type Support

| Content type | Current support |
|--------------|-----------------|
| Text | Capture, search, display, and restore as `text/plain` |
| HTML | Capture and search indexing; restore as `text/html` |
| Images | Capture, thumbnail display, and restore with image MIME type |
| File URI lists | Capture, display metadata, and restore as `text/uri-list` |

### User Interface

- **COSMIC native applet** - Built with `libcosmic`.
- **Tabs** - Clipboard, Emoji, Symbols, Kaomoji, Snippets, Settings.
- **Expression pickers** - Emoji, symbol, and kaomoji search.
- **Recently used tracking** - Across picker tabs.
- **Keyboard workflow** - Arrows, Home/End, PgUp/PgDn, Ctrl+1-9, Ctrl+D, Enter, Escape.
- **Auto-scroll** - Follows keyboard selection.
- **Smart refresh** - Diff-based refresh preserves scroll position.
- **Daemon status indicator** - Real-time connection status in settings/status UI.

### Security & Privacy

- **Sensitive content detection** - Passwords, OTPs, JWTs, API keys, SSH keys, AWS credentials, URI credentials, and high-entropy secrets.
- **Encryption at rest** - AES-256-GCM for sensitive items when `encrypt_sensitive` is enabled.
- **Incognito mode** - Temporarily pause clipboard capture.
- **Clear on screen lock** - Configurable via `clear_on_lock`.
- **Screen lock detection** - `loginctl` and D-Bus `org.freedesktop.ScreenSaver`.
- **IPC socket security** - Socket in `$XDG_RUNTIME_DIR` or a private cache directory, never `/tmp`.
- **Audit logging** - Security events recorded without raw sensitive clipboard previews.
- **Threat model** - Documented in `SECURITY.md`.

### Tools & CLI

- **CLI control tool** - `toggle`, `show`, `hide`, `ping`, `history`, `status`, `clear`, `export`, `config`, `picker`, `hyprland-config`.
- **Global shortcut command** - `author-clipboard-ctl toggle` for compositor-managed keybindings.
- **Quick paste** - `wtype` preferred, `ydotool` optional, `wl-copy` copy-only fallback.
- **External menu picker** - `author-clipboard-ctl picker` integrates with `wofi`, `rofi`, or `fuzzel`.
- **First-party native picker** - `author-clipboard-hypr-picker` provides a standalone GTK4 layer-shell popup for Hyprland/wlroots.
- **Shared picker module** - Reusable picker logic in `shared::picker` for history, snippets, emoji, symbols, and kaomoji.
- **Hyprland config helper** - `author-clipboard-ctl hyprland-config` prints recommended keybinds and window rules.
- **Data export/import** - JSON format.
- **File manager integration** - `xdg-open` for file paths.

### Configuration

- **JSON config file** - `~/.config/author-clipboard/config.json`.
- **Configurable options** - `max_items`, `max_item_size`, `data_dir`, `ttl_seconds`, `cleanup_interval_seconds`, `keyboard_shortcut`, `dedup_window_seconds`, `encrypt_sensitive`, `clear_on_lock`, `mime_denylist`, `content_regex_denylist`.
- **Denylist rules** - MIME denylist and simple content patterns are implemented. `content_regex_denylist` is a legacy field name and does not implement full regex.
- **Settings tab** - In-app configuration display with stats and privacy controls.

### Infrastructure

- **Systemd user service** - `author-clipboard-daemon.service`.
- **`just` commands** - `install`, `enable`, `disable`, `status`, `logs`, `verify`, packaging helpers.
- **GitHub Actions CI** - Format, clippy, test, build.
- **Crash-safe database** - SQLite WAL mode.
- **Database migrations** - Automatic schema versioning.
- **Debian packaging support** - `cargo-deb` metadata and `just deb`.
- **Arch packaging template** - `packaging/arch/PKGBUILD` and `.SRCINFO`.
- **GitHub release assets** - Release workflow uploads `.deb` plus Arch/AUR packaging files.

### Planned Features

- [x] External menu picker via wofi/rofi/fuzzel.
- [x] First-party Hyprland/wlroots native picker (`author-clipboard-hypr-picker`).
- [ ] Waybar/Wayle status module.
- [ ] Verified `windowrulev2` examples for the native picker.
- [ ] AUR package.
- [ ] Nix flake.
- [ ] Flatpak/AppImage packaging with sandbox caveats.
- [ ] X11 fallback clipboard monitoring.
- [ ] OCR for images.
- [ ] Self-hosted E2EE sync.
- [ ] Demo GIFs for COSMIC and Hyprland.
- [ ] Shortcut configuration UI where compositor APIs allow it.

---

See `PROJECT_PLAN.md` for the full roadmap and `docs/DEVELOPMENT.md` for build instructions.
