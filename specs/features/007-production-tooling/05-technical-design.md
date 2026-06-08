# Technical Design: Production Tooling

> Implementation approach for production tooling.

---

## Overview

Production tooling consists of:
1. CLI tool (`author-clipboard-ctl`)
2. Systemd service file
3. Doctor command for diagnostics

---

## Affected Files

| File | Change |
|------|--------|
| `crates/ctl/src/main.rs` | CLI implementation |
| `packaging/systemd/author-clipboard-daemon.service` | Systemd unit file |
| `packaging/systemd/author-clipboard-daemon.env` | Environment file |

---

## Implementation Details

### CLI Structure

```rust
// In crates/ctl/src/main.rs

#[derive(Subcommand)]
enum Command {
    Toggle,
    Show,
    Hide,
    ShowAt { x: i32, y: i32 },
    Ping,
    Status,
    History { count: usize },
    Clear,
    Export { output: Option<String> },
    Config,
    Doctor,
    Copy { id: i64 },
    Picker {
        menu: MenuBackend,
        source: SourceArg,
        count: usize,
        prompt: String,
        include_sensitive: bool,
        action: ActionArg,
    },
    HyprlandConfig,
}
```

### Doctor Implementation

```rust
fn run_doctor() {
    let server = detect_display_server();
    let protocols = probe_wayland_protocols();
    let daemon_running = check_daemon();

    println!("Display: {server:?}");
    println!("Wayland: {}", if protocols.wayland { "connected" } else { "unavailable" });
    println!("wlr-data-control: {}", if protocols.wlr_data_control { "available" } else { "missing" });
    println!("wl_seat: {}", if protocols.seat { "available" } else { "missing" });
    println!("Clipboard capture: {}", if protocols.can_capture() { "supported" } else { "unsupported" });
    println!("Daemon: {}", if daemon_running { "running" } else { "not running" });
}
```

---

**Last Updated**: Phase 15 (Updated from draft)