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

## MCP T016–T017

- [x] Search, resources, safe get, and prompt payloads are centrally redacted.
- [x] MCP redaction does not trust permissive UI preview configuration.
- [x] Full sensitive get/copy requires confirmation on each request.
- [x] Destructive item and snippet deletion requires per-request confirmation.
- [x] Tool failures include stable machine-readable error codes.
- [x] Tests prove raw secrets are absent after redaction and from error payloads.
- [x] Local stdio setup and privacy behavior are documented for two clients.

## Implemented Command-Center Increments

- [x] Shared transforms are pure, privacy-gated, and used by IPC/CLI adapters.
- [x] Snippet variables validate safely and support canonical and compatibility syntax.
- [x] Default versioned history export redacts sensitive/encrypted payloads.
- [x] Full export and mutating import require explicit confirmation.
- [x] Import preview re-runs sensitive detection before storage.
- [x] Ordered capture rules enforce ignore and force-sensitive actions before storage.
- [x] Ignore-next-copy is armed through IPC/CLI and consumed once by capture.
- [x] MCP default resources redact sensitive values and confirmation gates sensitive/destructive requests.
- [ ] Capture-rule tag actions persist tags (schema support is not yet available).

## Evidence

- [x] `just verify` is green in the final review working tree (2026-07-12).
- [x] Integrated headless UI tests pass: 95 passed, 14 GTK-display tests ignored.
- [x] Shared tests pass except two Unix-socket tests blocked by sandbox `EPERM`
  (274 passed, 2 environment-blocked).
- [x] MCP tests pass: 6 passed, including raw-secret non-leakage.
- [ ] Supported-compositor results are recorded in `07-test-plan.md`.
- [ ] Performance measurements include machine/build context.
- [ ] Screenshots reflect the shipped binary and current theme.
- [ ] Manual Hyprland, COSMIC, and Sway command-center smoke results are recorded.

## Safety and Compatibility

- [ ] Sensitive content does not enter status, logs, screenshots, or default exports.
- [x] MCP redaction and per-request confirmation helpers are tested.
- [ ] Database/config migrations are forward-safe and rollback limitations documented.
- [ ] `status --json` keeps existing keys and compatible types.
- [ ] Setup generators preserve user-owned configuration and are idempotent.

## Release Truthfulness

- [ ] README features exist in the release artifacts.
- [ ] Hyprland, COSMIC, and Sway limitations are explicit.
- [ ] Optional dependencies are labelled with their affected behavior.
- [ ] Packaging includes binaries, schemas, service, desktop file, and icons.
- [ ] Known gaps remain unchecked and are not described as complete.
