# Domain Model: Hyprland-native UX & wlroots Polish

> Data structures and architecture for the Waybar module, `status --json`
> payload, and packaging polish.

---

## Architecture

```
┌──────────────┐  exec / exec-on-event  ┌────────────────────────┐
│  Waybar /    │ ─────────────────────▶ │ author-clipboard-ctl   │
│  Wayle /     │                        │ status --json          │
│  ags / etc.  │ ◀──── JSON payload ─── │  (reuses IPC client)   │
└──────────────┘                        └──────────┬─────────────┘
                                                    │ Unix socket
                                                    ▼
                                          ┌─────────────────────┐
                                          │  clipboard-daemon   │
                                          │  (IpcServer)        │
                                          └─────────────────────┘
```

The Waybar module is a thin shell wrapper that:
1. Calls `author-clipboard-ctl status --json` on a fixed interval (30s) and
   on `signal` (Waybar's exec-on-event mechanism — daemon `pkill -SIGUSR1`
   will trigger an immediate refresh).
2. Maps the JSON payload to Waybar's `text`, `tooltip`, `class`, and
   `alt` fields.

The `status --json` command runs against the daemon via `IpcClient`. If the
daemon is down, it falls back to direct DB access for the stats and reports
`running: false`. The plain-text preview is read from the database, masked
if the item is sensitive.

---

## Status JSON Payload

```json
{
  "running": true,
  "daemon_pid": 12345,
  "total": 142,
  "pinned": 7,
  "last_type": "text",
  "last_preview": "echo 'hello from the picker'",
  "last_timestamp": 1717861832,
  "sensitive_last": false
}
```

Field semantics:

| Field | Type | Source |
|-------|------|--------|
| `running` | bool | `IpcClient::send_command(Ping)` succeeds |
| `daemon_pid` | u32 \| null | Reported by daemon in Ping response |
| `total` | u64 | `db.stats().total` |
| `pinned` | u64 | `db.stats().pinned` |
| `last_type` | `"text"\|"image"\|"html"\|"files"\|"other"` | `db.get_recent(1)[0].content_type` |
| `last_preview` | string (truncated to 60 chars) | `plain_text` or filename or thumbnail marker |
| `last_timestamp` | u64 (unix seconds) | `timestamp` of most recent item |
| `sensitive_last` | bool | `sensitive` flag of most recent item |

If `sensitive_last == true`, `last_preview` is replaced with
`"Sensitive item"` regardless of `show_sensitive_previews`.

---

## Waybar Module Mapping

| Waybar field | Source |
|--------------|--------|
| `text` | `N items` (or `99+` when `N > 99`); falls back to `clipboard: down` when `!running` |
| `tooltip` | `last_preview` (truncated, sensitive-masked) + `\n` + `<icon> <total> · <pinned> pinned` |
| `class` | `running` (default) or `down` (when `!running`) or `image` (when `last_type == "image"`) |
| `alt` | `last_type` |
| `percentage` | omitted (binary up/down only) |

The shell script is deliberately simple — it only translates the JSON,
re-emits on `signal:7`, and updates Waybar's `class` for CSS styling.

---

## AUR Package Layout

The AUR PKGBUILD in `packaging/arch/PKGBUILD` builds and ships:

| Path | Mode | Source |
|------|------|--------|
| `/usr/bin/author-clipboard` | 0755 | `target/release/author-clipboard` |
| `/usr/bin/author-clipboard-daemon` | 0755 | `target/release/author-clipboard-daemon` |
| `/usr/bin/author-clipboard-ctl` | 0755 | `target/release/author-clipboard-ctl` |
| `/usr/bin/author-clipboard-hypr-picker` | 0755 | `target/release/author-clipboard-hypr-picker` |
| `/usr/lib/systemd/user/author-clipboard-daemon.service` | 0644 | `data/...service` |
| `/usr/share/applications/...desktop` | 0644 | `data/...desktop` |
| `/usr/share/metainfo/...metainfo.xml` | 0644 | `data/...metainfo.xml` |
| `/usr/share/icons/hicolor/scalable/apps/...svg` | 0644 | `resources/icons/...svg` |
| `/usr/share/licenses/$pkgname/LICENSE` | 0644 | `LICENSE` |
| `/usr/share/doc/$pkgname/README.md` | 0644 | `README.md` |

AUR submission is a manual step (AUR is read-only from CI). The
`docs/AUR.md` guide documents the one-time setup, the version bump flow,
and the `upgpkg:` commit message convention.

---

## Nix Flake Outputs

| Output | Type | Description |
|--------|------|-------------|
| `packages.<system>.default` | package | All-in-one workspace build (applet + daemon + ctl + hypr-picker) |
| `packages.<system>.applet` | alias of `default` | Convenience alias |
| `packages.<system>.daemon` | runCommand | `author-clipboard-daemon` only |
| `packages.<system>.ctl` | runCommand | `author-clipboard-ctl` only |
| `packages.<system>.hypr-picker` | runCommand | `author-clipboard-hypr-picker` only |
| `apps.<system>.default` | app | Launches the applet |
| `devShells.<system>.default` | shell | Rust + libcosmic + GTK4 + layer-shell |

`default.nix` is the non-flake fallback and mirrors the same derivation.

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
