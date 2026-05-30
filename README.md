# author-clipboard

> Native COSMIC clipboard manager with wlroots compositor support, including Hyprland and Sway.

[![CI](https://github.com/namikofficial/author-clipboard/actions/workflows/ci.yml/badge.svg)](https://github.com/namikofficial/author-clipboard/actions)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/namikofficial/author-clipboard)](https://github.com/namikofficial/author-clipboard/releases)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**author-clipboard** is a privacy-focused clipboard manager for COSMIC and wlroots Wayland compositors. It stores clipboard history in a local SQLite database with FTS5 search, detects sensitive content, supports optional encryption for sensitive items, and provides a libcosmic popup UI with emoji, symbol, and kaomoji pickers.

The default GUI is COSMIC-native through `libcosmic`. Hyprland users can use the native external picker mode with `wofi`, `fuzzel`, or `rofi`.

---

## Features

### Clipboard & Storage
- Persistent history with SQLite and WAL mode
- FTS5 full-text search with LIKE fallback
- Pin and unpin items
- Per-item TTL and automatic cleanup
- Configurable duplicate suppression window
- JSON export and import

### Supported Content Types

| Content type | Capture | Display | Restore / copy behavior |
|--------------|---------|---------|--------------------------|
| Text | Yes | Yes | Restored as plain text with `wl-copy` |
| HTML / rich text | Yes | Plain-text search/display fallback | Restored as `text/html` with `wl-copy --type text/html` |
| Images | Yes for supported image MIME types | Thumbnail/file metadata | Restored with `wl-copy --type <mime>` |
| File URI lists | Yes for `text/uri-list` | File names/metadata | Restored as `text/uri-list` |

### Security & Privacy
- Sensitive content detection for passwords, API keys, tokens, SSH keys, URI credentials, and high-entropy secrets
- AES-256-GCM encryption at rest for sensitive items when `encrypt_sensitive` is enabled
- Incognito mode to pause recording
- Optional sensitive-item clearing on screen lock
- Local-only denylist matching for MIME types and simple content patterns
- IPC over a Unix socket in `$XDG_RUNTIME_DIR` or a private cache directory

### UI & Integration
- libcosmic popup UI with light/dark theme support
- Emoji, symbol, kaomoji, snippet, and settings tabs
- Keyboard navigation: arrows, Home/End, PgUp/PgDn, Ctrl+1-9, Delete, Enter, Escape
- CLI tool: `author-clipboard-ctl`
- Systemd user service
- Quick paste with `wtype` or `ydotool`; `wl-copy` is a copy-only fallback
- Hyprland-native external picker mode through `wofi`, `rofi`, or `fuzzel`

### Planned
- Hyprland-native UX options such as Waybar module and layer-shell popup mode
- AUR package and Nix flake
- Flatpak/AppImage packaging, subject to clipboard sandbox limitations
- X11 fallback monitoring
- OCR for images and richer image handling
- Self-hosted encrypted sync

---

## Installation

### Download `.deb` Package

`.deb` packaging support exists through `cargo-deb`. If release artifacts are published, download the latest package matching your architecture from [GitHub Releases](https://github.com/namikofficial/author-clipboard/releases/latest).

```bash
# Example only; choose the current file from releases/latest.
sudo dpkg -i author-clipboard_*_amd64.deb

systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
```

### Build from Source

Requirements: Rust 1.75+, Wayland development libraries, SQLite, xkbcommon, and `pkg-config`.

```bash
git clone https://github.com/namikofficial/author-clipboard
cd author-clipboard
cargo build --release --workspace

just install
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
```

Ubuntu/Debian dependencies:

```bash
sudo apt install libwayland-dev libxkbcommon-dev libssl-dev libsqlite3-dev pkg-config
```

Arch dependencies:

```bash
sudo pacman -S wayland wl-clipboard sqlite xkbcommon
```

Optional quick-paste tools:

```bash
sudo pacman -S wtype
# ydotool is optional and may require daemon/permission setup.
```

---

## Quick Start

```bash
author-clipboard-ctl toggle          # Open or close picker
author-clipboard-ctl show            # Open picker
author-clipboard-ctl hide            # Close picker
author-clipboard-ctl ping            # Check daemon IPC
author-clipboard-ctl history         # List recent items
author-clipboard-ctl status          # Show database stats
author-clipboard-ctl clear           # Clear unpinned items
author-clipboard-ctl export out.json # Export history
author-clipboard-ctl config          # Show current config
author-clipboard-ctl doctor          # Probe display/protocol support
author-clipboard-ctl copy 42         # Restore item id 42
author-clipboard-ctl picker          # Open wofi/fuzzel/rofi picker
```

### COSMIC Shortcut

Add a custom shortcut in **COSMIC Settings -> Keyboard -> Custom Shortcuts**:

1. Command: `author-clipboard-ctl toggle`
2. Binding: `Super+V`

### Hyprland Setup

Install runtime packages:

```bash
sudo pacman -S wayland wl-clipboard sqlite xkbcommon
sudo pacman -S wtype        # optional, preferred for quick paste
sudo pacman -S ydotool      # optional, requires daemon/permissions
```

Build and install:

```bash
git clone https://github.com/namikofficial/author-clipboard
cd author-clipboard
cargo build --release --workspace
just install
```

Enable the daemon:

```bash
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
systemctl --user status author-clipboard-daemon
```

Add a Hyprland bind:

```ini
bind = SUPER, V, exec, author-clipboard-ctl toggle
```

For a Hyprland-native menu instead of the libcosmic app UI, bind the external picker:

```ini
bind = SUPER, V, exec, author-clipboard-ctl picker --menu wofi
```

`author-clipboard-ctl picker` auto-detects `wofi`, `fuzzel`, then `rofi` if `--menu` is omitted. The picker restores text, HTML, images, and file URI lists using the same clipboard restore path as the applet.

Optional window rules depend on the actual app class. Inspect it first:

```bash
hyprctl clients
```

If the class is `author-clipboard`, these rules may be useful:

```ini
windowrulev2 = float,class:^(author-clipboard)$
windowrulev2 = center,class:^(author-clipboard)$
```

Hyprland does not need `COSMIC_DATA_CONTROL_ENABLED`. Clipboard capture depends on Hyprland exposing wlroots `wlr-data-control`. The UI is still libcosmic-based and may not visually match Hyprland themes.

Quick paste on Hyprland:

- `wtype` is preferred for typing selected text into the active app.
- `ydotool` can work but may require daemon and input permissions.
- `wl-copy` only copies the selected item to the clipboard; it does not type or paste into the active app.

---

## Configuration

Config path: `~/.config/author-clipboard/config.json`

Default data path: usually `~/.local/share/author-clipboard`

Database path: `<data_dir>/clipboard.db`

Image storage: `<data_dir>/images` and `<data_dir>/thumbnails`

Incognito mode flag: `<data_dir>/.incognito`; when present, daemon capture is skipped.

| Key | Default | Description |
|-----|---------|-------------|
| `max_items` | `100` | Maximum clipboard items to retain |
| `max_item_size` | `1048576` | Maximum size per item in bytes |
| `data_dir` | Platform data dir | Database, images, thumbnails, and runtime flags |
| `ttl_seconds` | `604800` | Auto-expire unpinned items. `0` means never expire |
| `cleanup_interval_seconds` | `300` | How often cleanup runs |
| `keyboard_shortcut` | `"Super+V"` | Display/reference value; compositor binding is configured separately |
| `encrypt_sensitive` | `false` | Encrypt sensitive items at rest |
| `clear_on_lock` | `true` | Clear sensitive items when the screen locks |
| `dedup_window_seconds` | `2` | Skip identical content copied within this window |
| `mime_denylist` | `["application/x-kde-cutselection"]` | MIME prefixes or exact MIME types to skip |
| `content_regex_denylist` | `[]` | Legacy name for simple local patterns, not full regex |

Default example:

```json
{
  "max_items": 100,
  "max_item_size": 1048576,
  "data_dir": "/home/you/.local/share/author-clipboard",
  "ttl_seconds": 604800,
  "cleanup_interval_seconds": 300,
  "keyboard_shortcut": "Super+V",
  "encrypt_sensitive": false,
  "clear_on_lock": true,
  "dedup_window_seconds": 2,
  "mime_denylist": [
    "application/x-kde-cutselection"
  ],
  "content_regex_denylist": []
}
```

`content_regex_denylist` supports simple patterns only:

- `^prefix` matches content that starts with `prefix`
- `suffix$` matches content that ends with `suffix`
- `token` matches content containing `token`

Denylist matching is local-only and best-effort.

---

## Wayland Support Matrix

| Environment | Clipboard capture | UI integration | Status |
|-------------|-------------------|----------------|--------|
| COSMIC Wayland | Yes, with `COSMIC_DATA_CONTROL_ENABLED=1` | Native libcosmic | Primary target |
| Hyprland | Yes, via wlroots/wlr-data-control | Hyprland-native external picker or libcosmic app UI | Supported |
| Sway | Yes, via wlroots/wlr-data-control | libcosmic app UI | Supported |
| Other wlroots compositors | Maybe | libcosmic app UI | Best effort |
| GNOME/Mutter | No unless protocol is available | No native support | Unsupported |
| KDE/KWin | No unless protocol is available | No native support | Unsupported |
| X11 | No fallback implemented | No native support | Unsupported/planned |

Use `author-clipboard-ctl doctor` to verify actual live Wayland registry support. If GNOME or KDE ever exposes `zwlr_data_control_manager_v1` and `wl_seat`, the daemon can attempt capture through the same registry-verified path instead of relying on desktop-name assumptions.

### Enabling on COSMIC Desktop

COSMIC requires `COSMIC_DATA_CONTROL_ENABLED=1` to expose the data-control protocol to clipboard managers.

Temporary session:

```bash
export COSMIC_DATA_CONTROL_ENABLED=1
```

Persist across logins:

```bash
# Add to ~/.config/cosmic-comp/env, creating the file if needed.
COSMIC_DATA_CONTROL_ENABLED=1
```

System-wide NixOS example:

```nix
environment.sessionVariables.COSMIC_DATA_CONTROL_ENABLED = "1";
```

Log out and back in after setting the variable.

Security note: data-control lets clipboard manager apps read clipboard contents. Only run clipboard managers you trust.

---

## Troubleshooting

### Daemon Not Running

```bash
systemctl --user status author-clipboard-daemon
journalctl --user -u author-clipboard-daemon -f
```

### Clipboard Not Captured

```bash
echo $WAYLAND_DISPLAY
author-clipboard-ctl status
```

COSMIC:

```bash
echo $COSMIC_DATA_CONTROL_ENABLED
```

Hyprland:

```bash
hyprctl version
hyprctl clients
author-clipboard-ctl doctor
```

### Shortcut Does Nothing

- Check the compositor keybind.
- Run `author-clipboard-ctl toggle` manually.
- Check daemon IPC with `author-clipboard-ctl ping`.
- Check logs with `journalctl --user -u author-clipboard-daemon -f`.

### App Opens But Does Not Paste

- Normal selection copies the item to the clipboard; use your app's paste shortcut after that.
- Quick paste types text only when `wtype` or a working `ydotool` setup is available.
- If the backend is `wl-copy`, author-clipboard only updates the clipboard.

---

## Development

```bash
just                # Show available commands
just verify        # Format, lint, test, build
just build         # Build all crates
just check         # Quick type check
just test          # Run tests
just fmt           # Format code
just lint          # Clippy with -D warnings
just daemon        # Run clipboard daemon
just applet        # Run GUI applet
```

See [PROJECT_PLAN.md](PROJECT_PLAN.md) for the roadmap.

---

## Documentation

- [FEATURES.md](FEATURES.md) - Feature overview
- [PROJECT_PLAN.md](PROJECT_PLAN.md) - Development roadmap
- [docs/PACKAGING.md](docs/PACKAGING.md) - Packaging notes
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - Contribution guide
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) - Development tooling
- [docs/LOCAL_TESTING.md](docs/LOCAL_TESTING.md) - Local testing guide
- [SECURITY.md](SECURITY.md) - Security policy and threat model

## License

[GPL-3.0](LICENSE)
