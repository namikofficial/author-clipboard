# Test Plan: Native Power-User Revamp

> Verification matrix for implementation slices. Each task should add tests at
> the lowest useful layer before UI smoke checks.

---

## Unit Tests

| Area | Command | Coverage |
|------|---------|----------|
| Query parser | `cargo test -p author-clipboard-shared -- query` | token parsing, quoted phrases, malformed filters, Unicode |
| Classification | `cargo test -p author-clipboard-shared -- classify` | URL/path/code/JSON/SQL/command/secrets |
| Collections | `cargo test -p author-clipboard-shared -- collection` | create, rename, delete, membership, cascade |
| Saved filters | `cargo test -p author-clipboard-shared -- saved_filter` | CRUD and apply |
| Snippets | `cargo test -p author-clipboard-shared -- template` | render variables, cursor, unknown vars |
| UI reducer | `cargo test -p author-clipboard-ui-gtk -- app::tests` | action effects, selected item, reveal state |
| Key handling | `cargo test -p author-clipboard-ui-gtk -- controller::key` | Esc, actions, navigation |
| CLI | `cargo test -p author-clipboard-ctl` | command shape and JSON helpers |

## Integration Tests

| Flow | Verification |
|------|--------------|
| Daemon capture | Copy text with `wl-copy`, assert daemon stores row. |
| Unicode safety | Copy multi-byte text, assert no daemon panic. |
| IPC actions | Pin/star/delete through IPC and verify DB state. |
| Collection lifecycle | Create collection, add item, list items, delete collection. |
| Export/import v2 | Export redacted payload, dry-run import, import into temp DB. |
| Health | Stop daemon, run health/status, restart daemon, compare output. |

## Runtime Smoke Tests

### Native Picker

```bash
just install
setsid -f author-clipboard-hypr-picker
hyprctl clients -j | jq '.[] | select(.class == "com.namikofficial.author-clipboard.popup")'
wtype -k Escape
pgrep -af '^author-clipboard-hypr-picker' && exit 1 || true
```

Expected:

- opens as XDG utility by default
- Hyprland rule floats and centers it when configured
- `Esc` closes the window and process

### Layer-Shell Compatibility

```bash
timeout 3s author-clipboard-hypr-picker --layer-shell --count 5
```

Expected:

- opens without schema/display errors
- remains available for users who prefer overlay mode

### UI Screenshots

```bash
just ui-smoke
```

Expected:

- popup, manager, clipboard page, and search screenshots updated
- screenshots use real GTK app, not mock HTML

## Performance Tests

Create a seeded DB with:

- 5,000 text items
- 500 code/command-like items
- 100 URL/path items
- 50 image/file entries
- 50 sensitive entries
- 20 collections
- 20 saved filters

Measurements:

| Metric | Target |
|--------|--------|
| Picker cold launch | < 250 ms target |
| Search query update | < 100 ms target |
| Result render after filter | < 150 ms target |
| Memory after launch | < 150 MB target |
| Scroll/list interaction | no visible jank |

## Security Tests

- Sensitive item rows never include raw content.
- `status --json` masks sensitive preview.
- Export redacts sensitive/encrypted payloads by default.
- Reveal countdown hides sensitive preview again.
- Logs do not contain raw sensitive sample strings.

## Manual Review Checklist

- Verify mouse and keyboard parity for every primary action.
- Verify narrow and wide window layouts.
- Verify light and dark theme readability.
- Verify no stale process remains after close/copy/Esc.
- Verify install from clean user state: no existing schema, no desktop file.
- Verify docs describe Wayland/source-app limitations honestly.

---

**Last Updated**: 2026-06-19
