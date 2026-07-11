# Release Review Checklist

## Foundation T002–T005

- [x] Clipboard result widgets request snapshots through IPC, not SQLite.
- [x] Popup and manager populate the shared authoritative `AppState.items`.
- [x] Selection and actions are driven by stable database ID.
- [x] Unknown IDs never select row zero.
- [x] Refresh preserves selection when the ID remains visible.
- [x] Delete chooses next, previous, or no selection deterministically.
- [x] GTK rows are rebound/reordered by stable ID instead of rebuilt wholesale.
- [x] A 1,000-row keyed reconciliation fixture is covered.
- [x] Arbitrary delayed refresh is removed.
- [x] Daemon capture emits an explicit revision signal and IPC snapshots expose it.

## Evidence

- [ ] `just verify` is green on the release commit.
- [ ] Supported-compositor results are recorded in `07-test-plan.md`.
- [ ] Performance measurements include machine/build context.
- [ ] Screenshots reflect the shipped binary and current theme.

## Safety and Compatibility

- [ ] Sensitive content does not enter status, logs, screenshots, or default exports.
- [ ] MCP permissions and confirmation behavior are tested.
- [ ] Database/config migrations are forward-safe and rollback limitations documented.
- [ ] `status --json` keeps existing keys and compatible types.
- [ ] Setup generators preserve user-owned configuration and are idempotent.

## Release Truthfulness

- [ ] README features exist in the release artifacts.
- [ ] Hyprland, COSMIC, and Sway limitations are explicit.
- [ ] Optional dependencies are labelled with their affected behavior.
- [ ] Packaging includes binaries, schemas, service, desktop file, and icons.
- [ ] Known gaps remain unchecked and are not described as complete.
