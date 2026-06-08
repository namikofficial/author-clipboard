# API Contract: Hyprland Integration

> IPC protocol and CLI commands for Hyprland picker.

---

## CLI Commands

### Picker (external menu)

```bash
author-clipboard-ctl picker [OPTIONS]

Options:
  --menu <backend>   auto, wofi, fuzzel, rofi (default: auto)
  --source <source>  history, snippets, emoji, symbols, kaomoji, all
  --count <n>        Number of items (default: 50)
  --prompt <text>   Prompt shown by menu (default: Clipboard)
  --action <action>  copy, quick-paste (default: copy)
```

### Hyprland Config Generator

```bash
author-clipboard-ctl hyprland-config
```

**Output**:
```bash
# Author Clipboard - Hyprland configuration
# Add these to your hyprland.conf

# External menu picker (rofi/wofi/fuzzel)
bind = SUPER, V, exec, author-clipboard-ctl picker --menu auto

# First-party Hyprland-native picker
bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker

# Optional COSMIC applet toggle
bind = SUPER ALT, V, exec, author-clipboard-ctl toggle

# Verify app class and choose rules (if not using layer-shell):
# hyprctl clients | grep -i author

# Make sure the daemon is running:
# systemctl --user enable --now author-clipboard-daemon
```

---

## IPC Commands

### Show/Hide (via daemon IPC)

```json
{"cmd": "Show", "args": {}}
{"cmd": "Hide", "args": {}}
{"cmd": "Toggle", "args": {}}
```

---

## Hypr-Picker Binary

```bash
author-clipboard-hypr-picker [OPTIONS]

Options:
  --source <source>   history, snippets, emoji, symbols, kaomoji, all (default: history)
  --count <n>         Number of items (default: 50)
  --action <action>   copy, quick-paste (default: copy)
  --include-sensitive  Show sensitive items (masked)
```

---

**Last Updated**: Phase 15 (Updated from draft)