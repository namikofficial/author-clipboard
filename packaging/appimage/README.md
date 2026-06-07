# AppImage Notes

The AppImage produced by `packaging/appimage/build.sh` is a **non-sandboxed** way
to run author-clipboard without installing packages. It is intended for users
who can't install `.deb`/AUR/Flatpak packages (e.g. locked-down corporate
machines, school labs, ephemeral sessions).

## Building

```bash
cargo build --release --workspace
bash packaging/appimage/build.sh
```

The output is `dist/author-clipboard-<version>-x86_64.AppImage`.

## Running

```bash
chmod +x dist/author-clipboard-*.AppImage
./dist/author-clipboard-*.AppImage
```

## Caveats

- **No sandboxing.** Unlike Flatpak, the AppImage runs with the user's full
  permissions. Treat it the same as the `.deb` package.
- **Wayland clipboard** still works through the normal `wlr-data-control`
  protocol; the AppImage just needs to be on a Wayland session.
- **Systemd service** is not auto-installed. The AppImage is intended for
  short-lived sessions. For persistent daemon management, use the `.deb` /
  AUR / source install.
- **Updates** are manual: re-download the AppImage from the GitHub release.
- **No AUR-style signature** beyond what GitHub Releases provide.

## When to prefer Flatpak

If you want a sandboxed install with portal-mediated clipboard access, use
the Flatpak manifest in `../flatpak/`. See `docs/FLATPAK.md`.
