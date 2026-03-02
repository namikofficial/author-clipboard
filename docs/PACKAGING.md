# Packaging Guide

Instructions for packaging author-clipboard for various Linux distributions.

## Building a .deb Package

Author Clipboard uses [cargo-deb](https://github.com/kornelski/cargo-deb) for Debian packaging.

### Prerequisites

```bash
cargo install cargo-deb
```

### Build

```bash
# Full build (compiles + packages)
just deb

# Or manually:
cargo build --release --workspace
cargo deb -p author-clipboard-applet --no-build
```

The `.deb` will be at `target/debian/author-clipboard_x.x.x-1_amd64.deb`.

### Test installation locally

```bash
just deb-install  # builds + sudo dpkg -i
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

---

## Building from Source

### Prerequisites

- Rust toolchain (stable, 1.75+)
- System packages: `libwayland-dev`, `libxkbcommon-dev`, `pkg-config`, `libsqlite3-dev`
- For COSMIC integration: `libcosmic` (pulled from git during build)

### Build

```bash
# Install system dependencies (Debian/Ubuntu/Pop!_OS)
just install-deps

# Build release binaries
cargo build --release

# Binaries are in target/release/:
#   author-clipboard          (applet/GUI)
#   author-clipboard-daemon   (background service)
#   author-clipboard-ctl      (CLI control)
```

## Installing

### Quick Install (cargo)

```bash
cargo install --path crates/applet
cargo install --path crates/clipboard-daemon
cargo install --path crates/ctl
```

Binaries are installed to `~/.cargo/bin/`.

### Full Install with systemd

```bash
just install   # Build + install binaries + install systemd service
just enable    # Start daemon on login
```

### Manual Install

```bash
# Copy binaries
sudo install -Dm755 target/release/author-clipboard /usr/local/bin/
sudo install -Dm755 target/release/author-clipboard-daemon /usr/local/bin/
sudo install -Dm755 target/release/author-clipboard-ctl /usr/local/bin/

# Install systemd service
install -Dm644 data/author-clipboard-daemon.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
```

## Distribution Packaging

### Debian/Ubuntu (.deb)

A `.deb` package is available via [GitHub Releases](https://github.com/namikofficial/author-clipboard/releases/latest).
See [Building a .deb Package](#building-a-deb-package) above for build instructions.

### Arch Linux (PKGBUILD)

```pkgbuild
# Maintainer: Namik <namikofficial@users.noreply.github.com>
pkgname=author-clipboard
pkgver=0.3.1
pkgrel=1
pkgdesc='Native COSMIC desktop clipboard manager for Wayland'
arch=('x86_64')
url='https://github.com/namikofficial/author-clipboard'
license=('GPL-3.0-or-later')
depends=('wayland' 'sqlite' 'xkbcommon')
makedepends=('rust' 'cargo' 'pkg-config' 'wayland-protocols')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")

build() {
    cd "$pkgname-$pkgver"
    cargo build --release
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 target/release/author-clipboard "$pkgdir/usr/bin/author-clipboard"
    install -Dm755 target/release/author-clipboard-daemon "$pkgdir/usr/bin/author-clipboard-daemon"
    install -Dm755 target/release/author-clipboard-ctl "$pkgdir/usr/bin/author-clipboard-ctl"
    install -Dm644 data/author-clipboard-daemon.service "$pkgdir/usr/lib/systemd/user/author-clipboard-daemon.service"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

### NixOS

A Nix flake is planned for future releases. For now, build from source using `cargo`.

### Flatpak

Flatpak packaging is under consideration. Clipboard managers require special portal permissions on Flatpak, which may limit functionality.

## Uninstalling

```bash
just uninstall-service  # Remove systemd service
# Remove binaries
rm -f ~/.cargo/bin/author-clipboard
rm -f ~/.cargo/bin/author-clipboard-daemon
rm -f ~/.cargo/bin/author-clipboard-ctl
# Remove data (optional)
rm -rf ~/.local/share/author-clipboard
rm -rf ~/.config/author-clipboard
```
