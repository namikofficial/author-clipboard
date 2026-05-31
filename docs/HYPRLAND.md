# Hyprland Integration Guide

> Setup guide for using author-clipboard on Hyprland with external menu pickers and the first-party native picker.

---

## Overview

author-clipboard supports Hyprland through two picker modes:

1. **External menu picker** (`author-clipboard-ctl picker`) — pipes items to `wofi`, `fuzzel`, or `rofi`
2. **First-party native picker** (`author-clipboard-hypr-picker`) — standalone GTK4 layer-shell popup

Both share the same database, search logic, and clipboard restore path.

---

## Prerequisites

```bash
# Required
sudo pacman -S wayland wl-clipboard sqlite xkbcommon

# Recommended
sudo pacman -S wtype        # quick paste support

# External menu picker (pick one)
sudo pacman -S wofi         # most common
sudo pacman -S fuzzel       # fast and minimal
sudo pacman -S rofi         # feature-rich

# First-party picker dependencies (for building from source)
sudo pacman -S gtk4 gtk4-layer-shell
```

---

## Daemon Setup

```bash
# Build and install
git clone https://github.com/namikofficial/author-clipboard
cd author-clipboard
cargo build --release --workspace
just install

# Enable daemon
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
systemctl --user status author-clipboard-daemon
```

---

## External Menu Picker

The external picker pipes clipboard items to a dmenu-style menu:

```bash
# Auto-detect backend
author-clipboard-ctl picker

# Explicit backend
author-clipboard-ctl picker --menu wofi
author-clipboard-ctl picker --menu fuzzel
author-clipboard-ctl picker --menu rofi
```

### CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--menu` | auto | Menu backend: `wofi`, `fuzzel`, `rofi`, or `auto` |
| `--source` | history | Data source: `history`, `snippets`, `emoji`, `symbols`, `kaomoji`, `all` |
| `--count` | 50 | Number of items to show |
| `--prompt` | Clipboard | Prompt text shown in the menu |
| `--include-sensitive` | off | Show masked sensitive items |
| `--action` | copy | Action: `copy` or `quick-paste` |

### Examples

```bash
# Browse emoji
author-clipboard-ctl picker --source emoji --prompt "Emoji"

# Browse snippets
author-clipboard-ctl picker --source snippets --prompt "Snippets"

# Quick paste text items
author-clipboard-ctl picker --action quick-paste
```

---

## First-Party Native Picker

`author-clipboard-hypr-picker` is a standalone GTK4 layer-shell popup:

```bash
# Open with defaults
author-clipboard-hypr-picker

# Specify source
author-clipboard-hypr-picker --source emoji
author-clipboard-hypr-picker --source history --count 30
```

### CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | history | Data source |
| `--count` | 50 | Maximum items |
| `--include-sensitive` | off | Show sensitive items |
| `--action` | copy | `copy` or `quick-paste` |

### Keyboard Controls

| Key | Action |
|-----|--------|
| Type | Filter/search |
| ↑ / ↓ | Navigate selection |
| Enter | Copy selected and close |
| Ctrl+Enter | Quick-paste selected text |
| Delete | Delete selected item |
| Ctrl+P | Pin/unpin selected history item |
| Ctrl+1..9 | Quick select by position |
| Esc | Close picker |

---

## Hyprland Keybinds

Add to `~/.config/hypr/hyprland.conf`:

```ini
# External menu picker (fast, uses wofi/fuzzel/rofi)
bind = SUPER, V, exec, author-clipboard-ctl picker --menu auto

# First-party native picker (standalone GTK4 popup)
bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker

# COSMIC applet toggle (requires libcosmic runtime)
bind = SUPER ALT, V, exec, author-clipboard-ctl toggle
```

### Clipboard History Shortcuts

```ini
# Quick paste most recent item
bind = SUPER SHIFT, C, exec, author-clipboard-ctl copy 1
```

---

## Window Rules

For the COSMIC applet (libcosmic-based):

```ini
windowrulev2 = float,class:^(author-clipboard)$
windowrulev2 = center,class:^(author-clipboard)$
```

The first-party picker (`author-clipboard-hypr-picker`) uses layer-shell, so it does not appear in `hyprctl clients` as a regular window and does not need window rules.

### Inspecting the COSMIC applet class

```bash
hyprctl clients | grep -i author
```

---

## Quick Paste Behavior

- `wtype` is preferred for typing selected text into the active app
- `ydotool` works but may require daemon and input permissions
- `wl-copy` only updates the clipboard; it does not type or paste

```bash
# Test wtype
echo "hello" | wtype -

# Test quick paste
author-clipboard-ctl picker --action quick-paste --menu wofi
```

---

## Sensitive Item Behavior

- Sensitive items (passwords, API keys, tokens) are masked by default
- The picker shows "Sensitive item" instead of raw content
- Use `--include-sensitive` to allow selecting sensitive entries from CLI
- Set `picker.confirm_sensitive_copy: true` to require confirmation in the native picker

---

## Configuration

Config path: `~/.config/author-clipboard/config.json`

Picker-specific settings:

```json
{
  "picker": {
    "default_mode": "external",
    "default_source": "history",
    "max_results": 50,
    "show_sensitive_previews": false,
    "confirm_sensitive_copy": true,
    "close_after_copy": true,
    "prefer_quick_paste": false,
    "width": 720,
    "height": 520
  }
}
```

---

## Troubleshooting

### External picker does not appear

```bash
# Check if a backend is installed
which wofi || which fuzzel || which rofi

# Test the picker manually
author-clipboard-ctl picker --menu wofi
```

### Native picker does not appear

```bash
# Check GTK4 and layer-shell
author-clipboard-hypr-picker --help

# Check if GTK4 is installed
pkg-config --modversion gtk4

# Check layer-shell support
pkg-config --modversion gtk4-layer-shell
```

### Clipboard not captured

```bash
author-clipboard-ctl doctor
hyprctl version
echo $WAYLAND_DISPLAY
```

### Daemon not running

```bash
systemctl --user status author-clipboard-daemon
journalctl --user -u author-clipboard-daemon -f
```

---

## Known Limitations

- The COSMIC applet uses `libcosmic` and may not visually match Hyprland themes
- Quick paste requires `wtype` or a working `ydotool` setup
- The first-party picker requires GTK4 and gtk4-layer-shell at runtime
- Layer-shell popups do not appear in `hyprctl clients` output
