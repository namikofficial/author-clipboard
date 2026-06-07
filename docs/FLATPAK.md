# Flatpak Notes

The Flatpak manifest lives at
`packaging/flatpak/com.namikofficial.author-clipboard.yml`. It builds
author-clipboard against the Freedesktop 23.08 SDK and the `rust-stable`
extension.

## Building

```bash
# Install Flatpak + the required runtimes (one-time)
flatpak install flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08

# Build
flatpak-builder --user --force-clean build-dir \
  packaging/flatpak/com.namikofficial.author-clipboard.yml

# Install the resulting bundle
flatpak-builder --user --install build-dir \
  packaging/flatpak/com.namikofficial.author-clipboard.yml
```

Then launch:

```bash
flatpak run com.namikofficial.author-clipboard
```

## Wayland Clipboard Caveats

Clipboard access on Flatpak is mediated by `xdg-desktop-portal`. The
manifest declares the required portals and a writable `xdg-data` for the
clipboard database:

```yaml
finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --filesystem=xdg-data/author-clipboard:create
  - --filesystem=xdg-config/author-clipboard:ro
  - --socket=pulseaudio
  - --talk-name=org.freedesktop.Notifications
  - --talk-name=org.kde.StatusNotifierWatcher
```

The portal needs to be running on the host:

```bash
systemctl --user status xdg-desktop-portal
```

If you are on a wlroots compositor (Hyprland, Sway), the portal needs
`xdg-desktop-portal-wlr`. See https://github.com/emersion/xdg-desktop-portal-wlr
for setup.

## Build Time

Expect a first build to take 10-15 minutes (libcosmic is a heavy
dependency). `flatpak-builder` caches intermediates; subsequent builds
are typically 1-3 minutes.

## When Not to Use Flatpak

- If you need the systemd user service for autostart, prefer the
  `.deb` / AUR / source install. Flatpak sandboxes make the
  `~/.local/bin/author-clipboard-daemon` path unusable.
- If you want a single binary tarball, use the AppImage from
  `packaging/appimage/build.sh`.

## See Also

- [`docs/PACKAGING.md`](PACKAGING.md) — all install paths.
- [`docs/RELEASING.md`](RELEASING.md) — release flow.
