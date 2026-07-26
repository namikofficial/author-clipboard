# Author Clipboard — Hyprland Picker: Layer-Shell by Default

**Date:** 2026-07-27
**Status:** ✅ Complete

---

## Problem

`author-clipboard-hypr-picker` behaved like a normal resizable XDG window unless the user explicitly passed `--layer-shell`. The documentation presented it as a native layer-shell popup, but the default was misleading.

## Changes Made

### 1. CLI Argument Reversal (`crates/hypr-picker/src/main.rs`)

| Before | After |
|--------|-------|
| `--layer-shell` opt-in (hidden, default off) | `--layer-shell` hidden/deprecated (accepted, ignored) |
| No `--xdg-window` flag | `--xdg-window` opt-out for debugging |

- `PopupConfig.layer_shell` now defaults to `true`
- The `--layer-shell` flag is still accepted for backward compatibility with existing keybinds but is hidden from `--help` and ignored
- `--xdg-window` forces XDG window mode for debugging

### 2. Layer-Surface Configuration (`crates/ui-gtk/src/window/popup.rs`)

Added to the layer-shell initialization:

- **`set_namespace(Some("author-clipboard-picker"))`** — stable namespace for compositor matching and Hyprland window rules
- **`set_exclusive_zone(0)`** — explicit no-reserve to prevent pushing other windows
- Layer: `Overlay` (unchanged)
- Anchors: Top + Left + Right (unchanged, spans focused monitor)
- Keyboard mode: `OnDemand` (unchanged, reliable focus)

### 3. `PopupConfig` Default (`crates/ui-gtk/src/lib.rs`)

```rust
// Before
layer_shell: false,

// After
layer_shell: true,
```

All callers using `PopupConfig::default()` or `..Default::default()` now get layer-shell enabled automatically.

### 4. Hyprland Config Generator (`crates/ctl/src/main.rs`)

Updated the managed block to reflect that no window rules are needed:

```ini
# First-party Hyprland-native picker (layer-shell by default)
bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker

# No window rules needed — the picker uses layer-shell overlay by default.
# To force XDG window mode for debugging: author-clipboard-hypr-picker --xdg-window
```

### 5. Documentation Updates

| File | Change |
|------|--------|
| `docs/HYPRLAND.md` | Updated CLI options table, added `--xdg-window`, noted "no window rules needed" |
| `specs/features/011-hyprland-integration/03-api-contract.md` | Updated CLI reference with `--xdg-window` and layer-shell description |
| `docs/rebuild/02-hyprland-picker.md` | This document |

### 6. Tests (`crates/hypr-picker/src/main.rs`)

10 new unit tests:

| Test | Verifies |
|------|----------|
| `default_args_enable_layer_shell` | No flags → layer_shell = true |
| `xdg_window_flag_disables_layer_shell` | `--xdg-window` → layer_shell = false |
| `deprecated_layer_shell_flag_is_ignored` | `--layer-shell` accepted, doesn't change behavior |
| `source_defaults_to_history` | `--source` defaults to history |
| `action_defaults_to_copy` | `--action` defaults to copy |
| `filter_defaults_to_all` | `--filter` defaults to all |
| `count_defaults_to_50` | `--count` defaults to 50 |
| `popup_config_reflects_xdg_window_flag` | PopupConfig.layer_shell = false when --xdg-window |
| `popup_config_layer_shell_by_default` | PopupConfig.layer_shell = true by default |
| `include_sensitive_defaults_to_false` | `--include-sensitive` defaults to false |

### 7. Smoke Test Script (`crates/hypr-picker/tests/shellcheck`)

`crates/hypr-picker/tests/smoke.sh` verifies:
- `--xdg-window` appears in `--help` output
- `--layer-shell` is hidden from `--help` output
- Deprecated `--layer-shell` flag is still accepted
- Binary builds and runs with both flag modes

---

## Runtime Behavior Summary

### Default (no flags)
```
author-clipboard-hypr-picker
```
- Opens as a layer-shell overlay on the focused monitor
- Namespace: `author-clipboard-picker`
- Layer: Overlay
- Exclusive zone: 0 (no screen reservation)
- Keyboard: OnDemand (receives focus reliably)
- Does not appear in `hyprctl clients` as a regular window
- Does not participate in tiling layout
- Closes cleanly on Esc or close button

### Debugging mode
```
author-clipboard-hypr-picker --xdg-window
```
- Opens as a normal resizable XDG window
- Can be tiled, moved, and resized
- Useful for non-layer-shell compositors or debugging

### Backward compatibility
```
author-clipboard-hypr-picker --layer-shell  # accepted, ignored (layer-shell is already default)
```

---

## Validation

```
cargo test -p author-clipboard-hypr-picker    ✅ 10 passed
cargo test -p author-clipboard-ui-gtk         ✅ 97 passed, 14 ignored (GTK display required)
cargo clippy --workspace --all-targets -- -D warnings  ✅ Pass
cargo fmt --all -- --check                    ✅ Pass
```

---

## Files Changed

```
crates/hypr-picker/src/main.rs          — CLI args, tests
crates/hypr-picker/Cargo.toml           — dev-dependencies (assert_cmd)
crates/ui-gtk/src/lib.rs                — PopupConfig default
crates/ui-gtk/src/window/popup.rs       — namespace, exclusive_zone
crates/ctl/src/main.rs                  — hyprland config text
docs/HYPRLAND.md                        — CLI options, keybinds, troubleshooting
specs/features/011-hyprland-integration/03-api-contract.md — CLI reference
crates/hypr-picker/tests/smoke.sh       — new smoke test script
```
