# Packaging Guide

Instructions for packaging author-clipboard for Linux distributions.

Current workspace version: `0.5.0`.

author-clipboard is a native COSMIC clipboard manager with wlroots compositor support, including Hyprland and Sway. The UI is built with `libcosmic`; Hyprland support is runtime/compositor support, not a Hyprland-native UI.

## Debian/Ubuntu `.deb`

`.deb` packaging support exists through [cargo-deb](https://github.com/kornelski/cargo-deb). Published release artifacts depend on the GitHub release workflow; users should download the latest package matching their architecture from [releases/latest](https://github.com/namikofficial/author-clipboard/releases/latest) when artifacts are available.

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
```

### Package Contents

| File | Destination |
|------|-------------|
| `author-clipboard` | `/usr/bin/author-clipboard` |
| `author-clipboard-daemon` | `/usr/bin/author-clipboard-daemon` |
| `author-clipboard-ctl` | `/usr/bin/author-clipboard-ctl` |
| Systemd service | `/usr/lib/systemd/user/author-clipboard-daemon.service` |
| Desktop file | `/usr/share/applications/` |
| AppStream metainfo | `/usr/share/metainfo/` |
| Icon | `/usr/share/icons/hicolor/scalable/apps/` |

## Arch Linux / Hyprland

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

An Arch template lives at [`packaging/arch/PKGBUILD`](../packaging/arch/PKGBUILD). AUR publication is planned.

## Building from Source

### Prerequisites

- Rust toolchain, stable 1.75+
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

## Installing

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

install -Dm644 data/author-clipboard-daemon.service ~/.config/systemd/user/author-clipboard-daemon.service
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
```

## NixOS

A Nix flake is planned. For now, build from source with Cargo. COSMIC users should set `COSMIC_DATA_CONTROL_ENABLED=1` in the session environment; Hyprland users do not need that COSMIC-specific variable.

## Flatpak

Flatpak packaging remains planned. Clipboard managers may be limited by sandboxing and portal behavior, so any Flatpak package needs explicit Wayland clipboard testing and clear caveats.

## Uninstalling

```bash
just uninstall-service
rm -f ~/.cargo/bin/author-clipboard
rm -f ~/.cargo/bin/author-clipboard-daemon
rm -f ~/.cargo/bin/author-clipboard-ctl
rm -rf ~/.local/share/author-clipboard
rm -rf ~/.config/author-clipboard
```
