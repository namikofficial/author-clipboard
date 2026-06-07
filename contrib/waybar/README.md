# Waybar Module for author-clipboard

A drop-in Waybar module that shows the clipboard daemon's running state and a
summary of the last captured item.  Designed for Hyprland but works on any
Wayland bar that speaks JSON over `exec`.

---

## Requirements

- `jq` (Waybar dependency — present on every Hyprland install)
- `author-clipboard` installed (any of the distribution packages)
- `author-clipboard-daemon` running (the module degrades gracefully when it is not)

---

## Installation

### 1. Copy the script

```bash
mkdir -p ~/.local/share/author-clipboard
cp clipboard.sh ~/.local/share/author-clipboard/clipboard.sh
chmod +x ~/.local/share/author-clipboard/clipboard.sh
```

### 2. Add to your Waybar config

In `~/.config/waybar/config` (or wherever your config lives), add the
`custom/clipboard` block to the `"modules-right"` (or left) array and
merge the example `config.example.json` values:

```json
"modules-right": ["custom/clipboard"],
"custom/clipboard": {
    "exec": "~/.local/share/author-clipboard/clipboard.sh",
    "exec-on-event": true,
    "interval": 30,
    "signal": 7,
    "on-click": "author-clipboard-hypr-picker",
    "on-click-right": "author-clipboard-ctl toggle",
    "return-type": "json"
}
```

### 3. Add CSS classes (optional)

Copy `style.css` into your Waybar stylesheet or append its contents to your
existing `style.css`.  The module emits `class` values of `running`,
`down`, `text`, `image`, `html`, `files`, and `sensitive` so you can
style each state independently.

### 4. Restart Waybar

```bash
waybarctl restart   # or kill -SIGUSR1 your waybar instance
```

---

## Signal-based refresh

Waybar's `signal: 7` field means it re-runs the `exec` chain whenever
Waybar receives `SIGRTMIN+7`.  You can send this from anywhere to force an
immediate refresh without waiting for the next 30-second poll:

```bash
pkill -SIGUSR1 waybar   # force refresh
```

Or from a `systemd` timer or a script that hooks into clipboard changes.

---

## Customisation

| Environment variable | Default | Effect |
|--------------------|---------|--------|
| `CTL` | `author-clipboard-ctl` | Path or name of the ctl binary |
| `DATA_DIR` | `~/.local/share/author-clipboard` | Data directory (informational only) |

Example with custom binary path:

```bash
CTL=/usr/local/bin/author-clipboard-ctl ~/.local/share/author-clipboard/clipboard.sh
```

---

## Behaviour

| Daemon state | `text` | `class` | `tooltip` |
|-------------|--------|---------|-----------|
| Running, items present | `N items` (capped at `99+`) | `running text` / `running image` / … | First 60 chars of last item + counts |
| Running, empty | `clipboard: empty` | `running` | `clipboard: empty` |
| Down | `clipboard: empty` | `down` | `clipboard: down` |
| Last item sensitive | Same as above | `running sensitive` | `Sensitive item` (masked) |

---

## Alternative bars

Wayle, ags, and polybar all accept the same `exec` + JSON model.  The
`author-clipboard-ctl status --json` command emits the same payload
regardless of which bar calls it, so the script is reusable with minor
config changes.  See `docs/HYPRLAND.md` for the full JSON schema.

---

## Troubleshooting

### Module shows `clipboard: down` even when the daemon is running

The module polls every 30 seconds.  Try clicking the module or send
`pkill -SIGUSR1 waybar` to force a refresh.

### `jq` errors in the log

Verify `jq` is installed:

```bash
which jq
jq --version
```

### JSON parse errors

Run the script manually and inspect the output:

```bash
author-clipboard-ctl status --json | jq .
~/.local/share/author-clipboard/clipboard.sh
```

If `status --json` exits non-zero, the module still renders `class: down`.
If the daemon is truly running but the module shows `down`, check the IPC
socket path: `systemctl --user status author-clipboard-daemon`.
