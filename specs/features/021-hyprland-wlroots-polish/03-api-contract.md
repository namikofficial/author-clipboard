# API Contract: Hyprland-native UX & wlroots Polish

> CLI surface for `status --json`, Waybar module script interface, and
> distribution artifact contracts.

---

## CLI: `author-clipboard-ctl status --json`

```bash
author-clipboard-ctl status [--json] [--pretty]
```

The existing `Status` command becomes the structured-payload source for
the Waybar / Wayle module. When `--json` is passed:

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

When the daemon is down:

```json
{
  "running": false,
  "daemon_pid": null,
  "total": 142,
  "pinned": 7,
  "last_type": "text",
  "last_preview": "echo 'hello from the picker'",
  "last_timestamp": 1717861832,
  "sensitive_last": false
}
```

Exit code is always `0`. The `total` / `pinned` / `last_*` fields are read
directly from the local SQLite database so the module still works when the
daemon is down (graceful degradation, useful for "is my history still
there?" UX).

The legacy `status` output (human-readable) is preserved when neither
`--json` nor `--pretty` is passed.

---

## Waybar module script: `contrib/waybar/clipboard.sh`

Inputs (Waybar passes them as positional args):

| Arg | Meaning |
|-----|---------|
| `$1` | Mode: `update` (default) or `signal` |
| `$2` (optional) | Signal number (when `mode=signal`) |

Output (single line of JSON for Waybar's `custom/...` module):

```json
{"text": "12 items", "tooltip": "echo 'hello'\n[42] text · 2 pinned", "class": "running", "alt": "text"}
```

The script does not poll. Waybar drives polling via
`interval: 30` and signal-based refresh via `signal: 7`.

The script must:
- Be POSIX `sh`-compatible (no bashisms).
- Be ≤ 80 lines (audit-friendly).
- Exit `0` even when `ctl` returns a non-zero exit (Waybar expects a
  payload; failure is encoded in the payload as `class: down`).

---

## Waybar config snippet

```json
{
  "custom/clipboard": {
    "exec": "~/.local/share/author-clipboard/clipboard.sh update",
    "exec-on-event": true,
    "interval": 30,
    "signal": 7,
    "format": "{text}",
    "format-tooltip": "{tooltip}",
    "on-click": "author-clipboard-hypr-picker",
    "on-click-right": "author-clipboard-ctl toggle",
    "tooltip": true
  }
}
```

CSS classes emitted by the script:

| Class | When |
|-------|------|
| `running` | `running == true` |
| `down` | `running == false` |
| `image` | `last_type == "image"` |
| `text` | `last_type == "text"` |
| `sensitive` | `sensitive_last == true` |

---

## Signal-based refresh

The daemon already has graceful shutdown on `SIGTERM` / `SIGINT`. We
add `SIGUSR1` as a "refresh bar" signal that the daemon handles by
re-writing its `signal_pipe` (or, for Phase 19, by just no-op'ing; the
Waybar `signal` mechanism is polled so a no-op handler is fine).

This is the same Waybar pattern used by `network`, `battery`, etc.: an
external process sends `pkill -SIGRTMIN+7 waybar` and Waybar re-runs the
`exec` chain.

---

## AUR package contract

AUR releases follow the [AUR submission
guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines):

- `PKGBUILD` lives in `packaging/arch/PKGBUILD` and is mirrored verbatim
  to the AUR git repo on push.
- `.SRCINFO` is regenerated via `makepkg --printsrcinfo` and committed
  alongside the PKGBUILD.
- Commit messages use the `upgpkg:` prefix for version bumps.
- The AUR does **not** store release binaries; it builds from the source
  tarball at `archive/v<version>.tar.gz`. The `release.yml` workflow
  uploads `packaging/arch/PKGBUILD` and `.SRCINFO` to the GitHub Release
  as `aur/PKGBUILD`, `aur/.SRCINFO`, and
  `aur/author-clipboard-aur-files.tar.gz` so manual AUR maintainers
  have everything in one place.

---

## Nix flake contract

`flake.nix` must:

- Pin `nixpkgs` to a stable channel (`nixos-23.11`).
- Pin `rust-overlay` to track the same `nixpkgs` (via `inputs.nixpkgs.follows`).
- Use `rustPlatform.buildRustPackage` with `cargoLock = { lockFile = ./Cargo.lock; }`
  for reproducibility.
- Expose `default`, `applet`, `daemon`, `ctl`, `hypr-picker`,
  `apps.default`, and `devShells.default`.
- Use `RUSTFLAGS = "-D warnings"` so the build fails on clippy warnings
  (matches the CI policy).

`default.nix` is a strict subset of the flake for non-flake users.

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
