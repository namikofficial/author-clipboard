# Packaging Guide

Instructions for packaging author-clipboard for Linux distributions.

Current workspace version: `0.5.0`.

author-clipboard is a native COSMIC clipboard manager with wlroots compositor support, including Hyprland and Sway. The UI is built with `libcosmic`; Hyprland support is runtime/compositor support, not a Hyprland-native UI.

## Install Paths

| Distro / Format | Path | Notes |
|-----------------|------|-------|
| Debian / Ubuntu / Pop!_OS | [`.deb`](#debianubuntu-deb) | Built via `cargo-deb` from CI on every tag. |
| Arch / Manjaro / Endeavour | [AUR / PKGBUILD](#arch-linux--aur) | AUR publication is manual; PKGBUILD template ships in this repo. |
| Fedora / Bazzite / COSMIC Atomic | [Flatpak](#flatpak) | Manifest at `packaging/flatpak/`. |
| Locked-down / portable | [AppImage](#appimage) | Built locally; not sandboxed. |
| NixOS | [Nix flake](#nixos) | `flake.nix` and `default.nix` at the repo root. |
| COSMIC App Store | [Submission checklist](COSMIC_STORE.md) | Metadata, screenshots, build artifacts. |
| Source | [Building from source](#building-from-source) | Cargo workspace. |

## Debian/Ubuntu `.deb`

`.deb` packaging support exists through [cargo-deb](https://github.com/kornelski/cargo-deb). Published release artifacts are attached to each GitHub release; download the latest package matching your architecture from [releases/latest](https://github.com/namikofficial/author-clipboard/releases/latest).

### Prerequisites

```bash
cargo install cargo-deb
```

### Build

```bash
just deb

# Or manually:
cargo build --release --workspace
cargo deb -p author-clipboard-applet --no-build
```

The package is written to `target/debian/author-clipboard_<version>-1_<arch>.deb`.

### Test Locally

```bash
just deb-install

# Inspect the contents without installing:
just deb-inspect
```

### Package Contents

| File | Destination |
|------|-------------|
| `author-clipboard` | `/usr/bin/author-clipboard` |
| `author-clipboard-daemon` | `/usr/bin/author-clipboard-daemon` |
| `author-clipboard-ctl` | `/usr/bin/author-clipboard-ctl` |
| `author-clipboard-hypr-picker` | `/usr/bin/author-clipboard-hypr-picker` |
| Systemd service | `/usr/lib/systemd/user/author-clipboard-daemon.service` |
| Desktop file | `/usr/share/applications/` |
| AppStream metainfo | `/usr/share/metainfo/` |
| Icon | `/usr/share/icons/hicolor/scalable/apps/` |
| LICENSE | `/usr/share/doc/author-clipboard/LICENSE` |
| CHANGELOG | `/usr/share/doc/author-clipboard/CHANGELOG.md` |

## Arch Linux / AUR

Runtime dependencies:

- `wayland`
- `wl-clipboard`
- `sqlite`
- `xkbcommon`

Optional dependencies:

- `wtype` for preferred Wayland quick paste
- `ydotool` for advanced input automation, requiring daemon/permissions

Make dependencies:

- `rust`
- `cargo`
- `pkg-config`
- `wayland-protocols`

Arch/AUR templates live in:

- [`packaging/arch/PKGBUILD`](../packaging/arch/PKGBUILD)
- [`packaging/arch/.SRCINFO`](../packaging/arch/.SRCINFO)

The GitHub release workflow uploads these files as release assets alongside the `.deb`. AUR publication is still a manual follow-up unless an AUR deploy key/secret is configured. See [`docs/AUR.md`](AUR.md) for the runbook.

### Local Build (Arch)

```bash
just arch-build      # runs makepkg --nocheck --nodeps
just aur-check       # verifies .SRCINFO is in sync with PKGBUILD
```

## Flatpak

The Flatpak manifest is at
[`packaging/flatpak/com.namikofficial.author-clipboard.yml`](../packaging/flatpak/com.namikofficial.author-clipboard.yml).
It builds against the Freedesktop 23.08 SDK and grants Wayland socket access.

```bash
just flatpak-validate   # YAML check (no build)
just flatpak-build      # full build (requires flatpak-builder + runtimes)
```

See [`docs/FLATPAK.md`](FLATPAK.md) for portal setup and build caveats.

## AppImage

The AppImage build script is at
[`packaging/appimage/build.sh`](../packaging/appimage/build.sh). It downloads
`appimagetool` on first run, stages an `AppDir/` from the release binaries,
and produces `dist/author-clipboard-<version>-x86_64.AppImage`.

```bash
just appimage-check   # shellcheck syntax
just appimage-build   # full build
```

AppImages are **not sandboxed**; for that, use the Flatpak form. See
[`packaging/appimage/README.md`](../packaging/appimage/README.md).

## NixOS

A Nix flake is provided at the repo root.

```bash
# Run the applet directly
nix run github:namikofficial/author-clipboard

# Add to your profile
nix profile install github:namikofficial/author-clipboard

# Build (no install)
nix build github:namikofficial/author-clipboard

# Development shell
nix develop github:namikofficial/author-clipboard
```

For non-flakes users, `default.nix` provides a fallback:

```bash
nix-build -E '(import <nixpkgs> {}).callPackage ./default.nix {}'
```

The flake pins the upstream `v<version>` tag. When cutting a release, update
both `flake.nix` and the workspace `version` in lockstep.

## COSMIC App Store / Flathub

See [`docs/COSMIC_STORE.md`](COSMIC_STORE.md) for the submission checklist.
The required artifacts are the AppStream metainfo at
`data/com.namikofficial.author-clipboard.metainfo.xml`, the icon, the
desktop file, and 3-5 screenshots (place under `resources/screenshots/`).

## Building from Source

### Prerequisites

- Rust toolchain, stable 1.79+
- Wayland development headers
- xkbcommon
- SQLite
- `pkg-config`

Debian/Ubuntu:

```bash
sudo apt install libwayland-dev libxkbcommon-dev libssl-dev libsqlite3-dev pkg-config
```

Arch:

```bash
sudo pacman -S wayland wl-clipboard sqlite xkbcommon rust cargo pkgconf wayland-protocols
```

### Build

```bash
cargo build --release --workspace
```

Binaries:

- `target/release/author-clipboard`
- `target/release/author-clipboard-daemon`
- `target/release/author-clipboard-ctl`
- `target/release/author-clipboard-hypr-picker`

## Installing from Source

### Full Install with systemd

```bash
just install
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
```

### Manual Install

```bash
sudo install -Dm755 target/release/author-clipboard /usr/local/bin/author-clipboard
sudo install -Dm755 target/release/author-clipboard-daemon /usr/local/bin/author-clipboard-daemon
sudo install -Dm755 target/release/author-clipboard-ctl /usr/local/bin/author-clipboard-ctl
sudo install -Dm755 target/release/author-clipboard-hypr-picker /usr/local/bin/author-clipboard-hypr-picker

install -Dm644 data/author-clipboard-daemon.service ~/.config/systemd/user/author-clipboard-daemon.service
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
```

## Releasing

See [`docs/RELEASING.md`](RELEASING.md) for the maintainer runbook: tag
format, GPG signing, verification, store/AUR publication.

## Uninstalling

### From `.deb`

```bash
sudo dpkg -r author-clipboard
```

### From Source

```bash
just uninstall
rm -rf ~/.local/share/author-clipboard
rm -rf ~/.config/author-clipboard
```

### From AUR

```bash
sudo pacman -Rns author-clipboard
```

### From Flatpak

```bash
flatpak uninstall com.namikofficial.author-clipboard
```

### From Nix

```bash
nix profile remove '.*author-clipboard.*'  # adjust the selector
# or, if built with `nix build`:
rm -rf result
```
