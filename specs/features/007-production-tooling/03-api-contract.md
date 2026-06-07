# API Contract: Production Tooling

> CLI commands and IPC protocol for production tooling.

---

## CLI Commands

### Toggle

Toggle picker visibility.

```bash
author-clipboard-ctl toggle
```

### Show

Show picker.

```bash
author-clipboard-ctl show
```

### Hide

Hide picker.

```bash
author-clipboard-ctl hide
```

### ShowAt

Show picker at coordinates.

```bash
author-clipboard-ctl show-at --x 100 --y 200
```

### Ping

Check daemon is running.

```bash
author-clipboard-ctl ping
# Output: "Daemon is running" or exit 1
```

### Status

Get daemon and database status.

```bash
author-clipboard-ctl status
# Output:
# Items: 150
# Pinned: 12
# Size: 1024.5 KB
# Database: /path/to/clipboard.db
# Daemon: running
```

### History

List recent clipboard items.

```bash
author-clipboard-ctl history [OPTIONS]

Options:
  --limit <count>     Number of items (default: 10)
  --json              Output JSON
```

### Clear

Clear all unpinned items.

```bash
author-clipboard-ctl clear
```

### Export

Export clipboard history.

```bash
author-clipboard-ctl export [OPTIONS]

Options:
  --output <path>     Output file
  --json              Output JSON (default)
```

### Config

Show current configuration.

```bash
author-clipboard-ctl config
# Output: human-readable config
```

### Doctor

Probe compositor support.

```bash
author-clipboard-ctl doctor
# Output: detailed status report
```

### Copy

Copy a history item.

```bash
author-clipboard-ctl copy <id>
```

### Picker

Open external menu picker.

```bash
author-clipboard-ctl picker [OPTIONS]

Options:
  --menu <backend>    auto, wofi, fuzzel, rofi
  --source <source>   history, snippets, emoji, etc.
  --count <n>         Number of items
  --action <action>  copy, quick-paste
```

### HyprlandConfig

Print Hyprland configuration.

```bash
author-clipboard-ctl hyprland-config
# Output: lua config snippet
```

---

**Last Updated**: Phase 15 (Updated from draft)