# Hyprland Integration Guide

> Setup guide for using author-clipboard on Hyprland with external menu pickers and the first-party native picker.

**Feature Spec**: See `/specs/features/011-hyprland-integration/` for requirements, design, and tasks.

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

`author-clipboard-hypr-picker` is a standalone GTK4 layer-shell popup.
It uses the Wayland layer-shell protocol by default, so it appears as an
overlay on the currently focused monitor without tiling or reserving
screen space. No window rules are needed.

```bash
# Open with defaults (layer-shell overlay)
author-clipboard-hypr-picker

# Specify source
author-clipboard-hypr-picker --source emoji
author-clipboard-hypr-picker --source history --count 30

# Force XDG window mode (for debugging on non-layer-shell compositors)
author-clipboard-hypr-picker --xdg-window
```

### CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | history | Data source |
| `--count` | 50 | Maximum items |
| `--include-sensitive` | off | Show sensitive items |
| `--action` | copy | `copy` or `quick-paste` |
| `--xdg-window` | off | Force XDG window mode (debugging fallback) |

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

# First-party native picker (layer-shell overlay by default)
bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker

# COSMIC applet toggle (requires libcosmic runtime)
bind = SUPER ALT, V, exec, author-clipboard-ctl toggle
```

> **No window rules needed.** The native picker uses layer-shell by default,
> so it does not appear in `hyprctl clients` as a regular window and does not
> participate in tiling layout. To force XDG window mode for debugging, pass
> `--xdg-window`.

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
author-clipboard-ctl doctor --json
hyprctl version
echo $WAYLAND_DISPLAY
```

`doctor --fix` is deliberately narrow: it may create Author Clipboard's own
data directory, but it does not install packages, start services, or edit your
Hyprland configuration.

Generate the recommended managed block without changing files, or explicitly
opt in to an idempotent file update:

```bash
author-clipboard-ctl hyprland-config
author-clipboard-ctl hyprland-config --write ~/.config/hypr/hyprland.conf
```

Only text between the Author Clipboard managed-block markers is replaced.
Existing binds and comments outside that block are preserved. A malformed
half-block is rejected instead of being overwritten.

### Daemon not running

```bash
systemctl --user status author-clipboard-daemon
journalctl --user -u author-clipboard-daemon -f
```

---

## Demo

> **Note**: A real animated GIF screencast is a future release artifact.
> The section below is the canonical reproducible demo. Run these commands
> in a fresh Hyprland session to evaluate the picker without installing
> additional software.

### Reproducible shell transcript

```bash
# 1. Install the AUR package (Arch) or use Nix / binary release.
#    Here we assume the binaries are in PATH after install.

# 2. Enable and start the daemon.
systemctl --user enable --now author-clipboard-daemon
systemctl --user status author-clipboard-daemon   # confirm "active (running)"

# 3. Copy some text from any application (e.g., firefox, alacritty).
#    The daemon captures it automatically.

# 4. Open the native picker with Super+Shift+V.
author-clipboard-hypr-picker          # or use the keybind

# 5. Type to filter, use ↑/↓ to navigate, Enter to copy.
#    The picker closes after copying (if close_after_copy is true in config).

# 6. Paste in any application with Ctrl+V.
#    The pasted text matches what you selected in step 5.

# 7. Check daemon status and item count.
author-clipboard-ctl status
author-clipboard-ctl status --json    # structured output for bar modules

# 8. Use the external menu picker instead (no GTK4 required).
author-clipboard-ctl picker --menu wofi --source history
```

### Layer-shell popup layout

```
┌──────────────────────────────────────────────────────────────────┐
│  🔍  Search clipboard…                                           │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  echo 'hello from the picker'              12:34:05  📌    │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │  https://github.com/...                   12:30:22         │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │  Last copied text here...                    12:28:01      │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │  🖼️  image-preview.png                         11:55:40     │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  3/7  ·  ↑↓ navigate  ·  Enter copy  ·  Esc close            │
└──────────────────────────────────────────────────────────────────┘
          ↑─────────────────────────────────────────────────↑
          │                Layer-shell overlay                │
          │           (640×480, top-anchored, full-width)    │
          └──────────────────────────────────────────────────┘
```

### Waybar / Wayle status module

Once the daemon is running, add the clipboard module to your bar:

```bash
# Copy the module script into your data directory
mkdir -p ~/.local/share/author-clipboard
cp /path/to/contrib/waybar/clipboard.sh \
   ~/.local/share/author-clipboard/clipboard.sh
chmod +x ~/.local/share/author-clipboard/clipboard.sh
```

The module script is not included in the binary release packages.  It lives
in the source tree under `contrib/waybar/`.  If you installed from the
AUR, clone the repo to get the script:

```bash
git clone https://github.com/namikofficial/author-clipboard
cp author-clipboard/contrib/waybar/clipboard.sh ~/.local/share/author-clipboard/
```

See `contrib/waybar/README.md` for the full Waybar config snippet,
CSS classes, and signal-based refresh instructions.

---

## Known Limitations

- The COSMIC applet uses `libcosmic` and may not visually match Hyprland themes
- Quick paste requires `wtype` or a working `ydotool` setup
- The first-party picker requires GTK4 and gtk4-layer-shell at runtime
- Layer-shell popups do not appear in `hyprctl clients` output
