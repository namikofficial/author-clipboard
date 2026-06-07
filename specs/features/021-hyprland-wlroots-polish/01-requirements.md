# Requirements: Hyprland-native UX & wlroots Polish

---

## User Stories

### US-001: Waybar status module

**As a** Hyprland user running Waybar
**I want to** see a clipboard status indicator in my bar
**So that** I know the daemon is running and roughly what's in history

**Acceptance Criteria**:

- Given the daemon is running, when I add the module to my Waybar config,
  the module shows the clipboard icon with `N items` (N = total rows in
  the database, capped display to "99+").
- Given the daemon is not running, the module shows `clipboard: down` and
  the `class` includes `down` for styling.
- Given the last captured item is text, the `tooltip` shows the first 60
  characters of the plain-text preview.
- Given the last captured item is an image, the `tooltip` says `Last:
  image` and the icon name is `image`.
- The module's `exec` is `author-clipboard-ctl status --json` (single
  command) and the script in `contrib/waybar/clipboard.sh` only re-renders
  on `signal` events.
- Module uses `interval: 30` plus `signal: 7` so it doesn't poll more than
  every 30 s when nothing changes.

### US-002: Wayle / ags / generic bar support

**As a** user with a different bar (Wayle, ags, polybar)
**I want to** consume the same status payload
**So that** I don't have to fork the module

**Acceptance Criteria**:

- `author-clipboard-ctl status --json` prints a single JSON object on
  stdout with at minimum: `running` (bool), `total` (u64), `pinned` (u64),
  `last_type` (`text|image|html|files|other`), `last_preview` (truncated
  string, masked if sensitive).
- Exit code is `0` whether the daemon is up or down (status reflects state,
  not failure).

### US-003: AUR package

**As an** Arch user
**I want to** install author-clipboard from the AUR
**So that** I get all four binaries + the systemd user service + desktop
file in one `paru -S` / `yay -S`

**Acceptance Criteria**:

- `packaging/arch/PKGBUILD` builds all four binaries (daemon, applet, ctl,
  hypr-picker) and installs the systemd user service, .desktop file,
  metainfo, and icon.
- `.SRCINFO` is byte-identical to `makepkg --printsrcinfo` against the
  PKGBUILD; CI fails the PR otherwise (`arch-pkg` job).
- `docs/AUR.md` describes the one-time setup and the version-bump flow.

### US-004: Nix flake

**As a** NixOS user
**I want to** install author-clipboard via `nix profile install` or
include it in a system flake
**So that** the package is reproducible and pins the right rust toolchain

**Acceptance Criteria**:

- `flake.nix` exposes `packages.<system>.default`,
  `packages.<system>.{applet,daemon,ctl,hypr-picker}`,
  `apps.<system>.default`, and `devShells.<system>.default`.
- `nix flake check` exits `0` (or, if hashes haven't been filled in,
  fails with a clear "sha256 not yet pinned" error — see `09-decisions.md`).
- `default.nix` (non-flake) is kept in lockstep with `flake.nix`.

### US-005: Hyprland demo

**As a** prospective user
**I want to** see what the picker looks like in Hyprland
**So that** I can decide whether to install it

**Acceptance Criteria**:

- `docs/HYPRLAND.md` has a `## Demo` section with:
  - A reproducible shell transcript (commands + expected output) that
    demonstrates: launching the daemon, copying some text, opening the
    native picker, and selecting an item.
  - A short ASCII layout sketch of the layer-shell popup.
  - A link to the upstream repo where maintainers can attach a real GIF
    in a future release.

## Out of Scope

- Animated GIF / video capture (requires a graphical session that isn't
  available in CI).
- Native Hyprland IPC bridge to the daemon (the picker already uses the
  shared IPC over the Unix socket; no extra Hyprland-specific protocol is
  introduced).
- Bar-specific module repos (no separate `waybar-author-clipboard` repo).

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
