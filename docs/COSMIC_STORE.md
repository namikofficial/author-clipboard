# COSMIC App Store Submission

This document is a checklist for submitting author-clipboard to the COSMIC
App Store (or a third-party Flatpak distributor like Flathub). Submission
is a **manual** process; this repo provides the metadata, manifest, and
artifacts needed.

## Required Artifacts

| Item | Path / Filename | Notes |
|------|-----------------|-------|
| AppStream metainfo | `data/com.namikofficial.author-clipboard.metainfo.xml` | Validated by `appstreamcli validate`. |
| Icon (scalable SVG) | `resources/icons/com.namikofficial.author-clipboard.svg` | At least 128x128 logical; we ship scalable. |
| Desktop file | `data/com.namikofficial.author-clipboard.desktop` | Includes `Exec=author-clipboard`, `Icon=...`, `Categories=Utility;`. |
| .deb | `target/debian/author-clipboard_*.deb` | Built by `cargo deb`. |
| Flatpak manifest | `packaging/flatpak/com.namikofficial.author-clipboard.yml` | Build a Flatpak separately for Flathub. |
| Screenshots | (see below) | 3-5 PNG/JPG, 1600x900 or similar. |

## AppStream Metadata Checklist

Run `appstreamcli validate data/com.namikofficial.author-clipboard.metainfo.xml`
before each submission. The metainfo in this repo:

- [x] `<id>` matches the `.desktop` file basename.
- [x] `<metadata_license>` is set (MIT in our case).
- [x] `<project_license>` is set (GPL-3.0-or-later).
- [x] `<developer>` block identifies the maintainer.
- [x] `<launchable type="desktop-id">` points at the desktop file.
- [x] `<categories>` includes `Utility`.
- [x] `<content_rating type="oars-1.1"/>` is present.
- [x] `<supports><internet>offline-only</internet></supports>` is present.
- [x] `<releases>` block lists the current version with date and a one-line
      description (CI bumps this on tag).

## Screenshots

Author and place 3-5 screenshots in `resources/screenshots/`:

- `01-main-window.png` — applet open with the clipboard history visible.
- `02-search.png` — search results in the applet.
- `03-emoji-picker.png` — emoji picker overlay.
- `04-symbol-picker.png` — symbol picker overlay.
- `05-hyprland-picker.png` (optional) — Hyprland GTK4 layer-shell picker.

Recommended size: 1600x900, 24-bit color, no transparency. Reference
them from the metainfo's `<screenshots>` block.

## Privacy & Permissions

The app is **offline-only**. The metainfo declares this. The Flatpak manifest
declares only the minimum required permissions (Wayland socket, XDG data
read-write, Notifications). If a future feature needs network access, the
metainfo and the manifest must both be updated to declare it.

## Submission Steps (Flathub-style)

1. Fork https://github.com/flathub/flathub.
2. Add a new repo: `author-clipboard` with manifest at
   `com.namikofficial.author-clipboard.yml`. The file content is the same
   as `packaging/flatpak/com.namikofficial.author-clipboard.yml`, but the
   source tag is updated to the current release.
3. Open a PR; Flathub bots will run `flatpak-builder` to verify the build.
4. Address bot feedback (typically: missing icons, missing metainfo
   translations, oversized build).
5. After merge, the build is published to `flathub-beta` for human QA.
6. After QA approval, it's promoted to `flathub`.

## Submission Steps (COSMIC Store)

The COSMIC Store is currently in active development. Once the submission
portal is open:

1. Sign in with a Pop!_OS account.
2. Create a new app entry pointing at the `target/debian/author-clipboard_*.deb`
   (or the Flatpak from Flathub).
3. Fill in the metadata, copy-paste from the metainfo.
4. Upload screenshots from `resources/screenshots/`.
5. Submit for review.

## See Also

- [`docs/RELEASING.md`](RELEASING.md) — overall release flow.
- [`docs/PACKAGING.md`](PACKAGING.md) — all install paths.
- [`docs/FLATPAK.md`](FLATPAK.md) — Flatpak caveats.
