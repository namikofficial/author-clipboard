# UI Flow: Production Tooling

> User interaction flows for production tooling commands.

---

## Doctor Flow

```
[author-clipboard-ctl doctor]
        |
        v
[Detect display server]
        |
        v
[Probe Wayland protocols]
        |
        v
[Check IPC socket]
        |
        v
[Query database stats]
        |
        v
[Print formatted report]
```

---

## Systemd Flow

```
[systemctl --user start author-clipboard-daemon]
        |
        v
[Unit file loaded]
        |
        v
[Daemon binary executed]
        |
        v
[Daemon connects to Wayland]
        |
        v
[Daemon starts IPC server]
        |
        v
[Ready to capture clipboard]
```

---

## Config Flow

```
[author-clipboard-ctl config]
        |
        v
[Load config from ~/.config/author-clipboard/config.json]
        |
        v
[Print human-readable output]
```

---

**Last Updated**: Phase 15 (Updated from draft)