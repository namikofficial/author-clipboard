# Domain Model: Production Tooling

> Data structures and architecture for production tooling (CLI, systemd, doctor command).

---

## CLI Architecture

```
author-clipboard-ctl
    |
    +-- Toggle (IPC: Toggle)
    +-- Show (IPC: Show)
    +-- Hide (IPC: Hide)
    +-- ShowAt (IPC: ShowAt)
    +-- Ping (IPC: Ping)
    +-- Status (IPC: Status + DB query)
    +-- History (IPC: History)
    +-- Clear (IPC: ClearUnpinned)
    +-- Export (IPC: Export)
    +-- Config (IPC: GetConfig)
    +-- Doctor (Probe protocols)
    +-- Copy (IPC: Copy)
    +-- Picker (External menu)
    +-- HyprlandConfig (Print config)
```

---

## Systemd Service

```ini
[Unit]
Description=Author Clipboard Daemon
After=wayland.socket
Requires=wayland.socket

[Service]
ExecStart=/usr/local/bin/author-clipboard-daemon
Restart=on-failure
RestartSec=5
Environment=COSMIC_DATA_CONTROL_ENABLED=1

[Install]
WantedBy=default.target
```

---

## Doctor Command

```bash
author-clipboard-ctl doctor

Output:
Display: Hyprland
Wayland: connected
wlr-data-control: available
wl_seat: available
Clipboard capture: supported
Daemon: running
Items: 150
Pinned: 12
Database: /home/user/.local/share/author-clipboard/clipboard.db
```

---

**Last Updated**: Phase 15 (Updated from draft)