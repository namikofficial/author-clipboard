# Test Plan: Hyprland-native UX & wlroots Polish

> Test strategy for Phase 19 polish.

---

## Automated

| Test | Layer | What it covers |
|------|-------|----------------|
| `cargo test -p author-clipboard-shared` | Unit | DB stats accessor, sensitive masking (already in place) |
| `cargo test -p author-clipboard-ctl` | Unit | `Status` JSON shape, `hyprland-config` text content (existing) |
| `cargo test -p author-clipboard-hypr-picker` | Compile | GTK4 + layer-shell dependency wiring (compile-only — GTK4 init needs a display) |
| `shellcheck contrib/waybar/clipboard.sh` | Shell | POSIX sh compliance (manual / `just waybar-check`) |
| `bash -n contrib/waybar/clipboard.sh` | Shell | Syntax check (manual / `just waybar-check`) |
| `just aur-check` | AUR | `.SRCINFO` parity with PKGBUILD (CI `arch-pkg` job) |
| `cargo clippy -- -D warnings` | Lint | All new code matches the workspace's strict clippy policy |

## Manual

| Check | Steps |
|-------|-------|
| Waybar module renders | Add module to Waybar config, restart Waybar, observe icon + tooltip |
| Waybar `class: down` styling | Stop daemon (`pkill author-clipboard-daemon`), wait 30 s, observe class changes to `down` and tooltip reflects `clipboard: down` |
| `on-click` opens picker | Click the module, confirm `author-clipboard-hypr-picker` launches |
| `on-click-right` toggles applet | Right-click, confirm `author-clipboard-ctl toggle` runs |
| `status --json` valid JSON | `author-clipboard-ctl status --json \| jq .` parses without errors |
| `status --json` while daemon down | Stop daemon, run `status --json`, confirm `running: false` and other fields still present |
| AUR PKGBUILD builds | `cd packaging/arch && makepkg --nocheck --nodeps` in Arch container |
| Nix flake metadata | `nix flake check --no-build` in a Nix-enabled environment |
| Demo transcript is reproducible | Follow `docs/HYPRLAND.md` Demo section commands in a fresh Hyprland session |

## Test Data

A `tempfile`-backed test database in `crates/ctl/src/main.rs` `tests`
module exercises the `Status --json` path with zero items, one pinned
item, one sensitive item, and one normal text item.

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
