# Release Review Checklist

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

