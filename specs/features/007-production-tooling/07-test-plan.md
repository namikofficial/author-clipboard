# Test Plan: Production Tooling

> Test strategy for production tooling.

---

## Unit Tests

```rust
#[test]
fn test_hyprland_config_includes_binds() {
    let output = hyprland_config_text();
    assert!(output.contains("bind = SUPER, V"));
    assert!(output.contains("author-clipboard-ctl"));
}
```

---

## Integration Tests

```bash
# Test daemon start/stop
systemctl --user start author-clipboard-daemon
sleep 1
author-clipboard-ctl ping
systemctl --user stop author-clipboard-daemon
```

---

## Manual Test Checklist

- [ ] `author-clipboard-ctl toggle` works
- [ ] `author-clipboard-ctl ping` returns success
- [ ] `author-clipboard-ctl status` shows correct info
- [ ] `author-clipboard-ctl history` shows items
- [ ] `author-clipboard-ctl doctor` shows detailed report
- [ ] `author-clipboard-ctl hyprland-config` outputs valid lua
- [ ] Systemd service starts correctly

---

**Last Updated**: Phase 15 (Updated from draft)