# Release Candidate Test Plan

No row is marked passing without recording the date, build/commit, environment,
and evidence. “Not tested” is distinct from “unsupported”.

| Environment | Fresh install | Existing DB migration | Picker/copy | Quick paste | Doctor/setup | Result/evidence |
|---|---:|---:|---:|---:|---:|---|
| Hyprland | [ ] | [ ] | [ ] | [ ] | [ ] | Not run |
| COSMIC | [ ] | [ ] | [ ] | [ ] | [ ] | Not run |
| Sway | [ ] | [ ] | [ ] | [ ] | [ ] | Not run |
| Daemon unavailable | n/a | n/a | [ ] degraded | n/a | [ ] actionable | Not run |

## Data and Privacy Matrix

- [ ] Empty database renders actionable empty state.
- [ ] Existing and migrated databases retain history and organization metadata.
- [ ] 1,000- and 5,000-item fixtures remain searchable; timings are recorded.
- [ ] Sensitive rows, status JSON, logs, export, and MCP output remain redacted.
- [ ] Reveal is explicit and returns to redacted state after its timeout.
- [ ] MCP sensitive access requires confirmation and remains local-only.

## Workflow Matrix

- [ ] Collections lifecycle and multi-membership.
- [ ] Expression search/category/copy across emoji, symbols, and kaomoji.
- [ ] HTML, image, and file previews including missing-file behavior.
- [ ] Snippet variables and transforms.
- [ ] `status --json` remains compatible with the bundled Waybar script.
- [ ] Doctor failures name the affected feature and a corrective action.
- [ ] Hyprland generated config is idempotent and preserves user-owned text.
- [ ] Install, upgrade, rollback, and uninstall instructions match artifacts.

## Automated Gate

```bash
just verify
just perf-seed
just perf-picker
just ui-check
```

## Foundation Evidence — T002–T005

Recorded 2026-07-12 in the development workspace:

- 43 reducer/state tests pass, including unknown-ID, copy, quick-paste,
  navigation, delete, and snapshot replacement behavior.
- 2 reconciliation tests pass, including a synthetic 1,000-row one-item insert
  retaining all existing keyed rows.
- 4 clipboard adapter tests pass, including versioned IPC item mapping.
- Daemon and GTK packages pass `cargo check`; daemon unit target passes.
- `timeout_add_local_once` and `Database::open` are absent from the clipboard
  result page.
- Package clippy is blocked by pre-existing/concurrent controller and collections
  warnings; the foundation-specific cast/pass-by-value findings were fixed.
