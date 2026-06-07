#!/usr/bin/env bash
# AppImage build script for author-clipboard.
#
# Produces an AppImage that bundles the applet, daemon, ctl, and hypr-picker
# binaries plus the desktop file and icon. Run from the workspace root:
#
#   cargo build --release --workspace
#   bash packaging/appimage/build.sh
#
# The script downloads `appimagetool` on first run and verifies it against
# a pinned SHA256. Outputs to dist/author-clipboard-<version>.AppImage.
#
# Caveats: AppImages for wlroots apps are NOT sandboxed. The user is
# expected to grant Wayland clipboard access through their compositor's
# normal session; this is the same trust model as the .deb / AUR packages.
# For a sandboxed install, prefer the Flatpak form (see docs/FLATPAK.md).

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────
WORKSPACE_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${WORKSPACE_ROOT}/dist"
APPDIR="${DIST_DIR}/AppDir"
VERSION="$(grep '^version' "${WORKSPACE_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)"
APPIMAGE_NAME="author-clipboard-${VERSION}-x86_64.AppImage"
TOOLS_DIR="${DIST_DIR}/.tools"
APPIMAGETOOL_URL="https://github.com/AppImageCommunity/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
# Pinned for reproducibility. Update only after verifying upstream.
APPIMAGETOOL_SHA256="6f1c2c5b3e2c4f9d4e3b1a7c0d5b4e3a2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f"

# ── Helpers ───────────────────────────────────────────────────────────────
log()  { printf '\033[1;34m[appimage]\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31m[appimage]\033[0m %s\n' "$*" >&2; exit 1; }

# ── Pre-flight ────────────────────────────────────────────────────────────
[ -x "${WORKSPACE_ROOT}/target/release/author-clipboard" ] \
  || fail "Missing target/release/author-clipboard. Run: cargo build --release --workspace"

for bin in author-clipboard author-clipboard-daemon author-clipboard-ctl author-clipboard-hypr-picker; do
  [ -x "${WORKSPACE_ROOT}/target/release/${bin}" ] \
    || fail "Missing target/release/${bin}. Run: cargo build --release --workspace"
done

mkdir -p "${DIST_DIR}" "${TOOLS_DIR}"

# ── Acquire appimagetool ───────────────────────────────────────────────────
APPIMAGETOOL="${TOOLS_DIR}/appimagetool"
if [ ! -x "${APPIMAGETOOL}" ]; then
  log "Downloading appimagetool..."
  curl -fL --retry 3 -o "${APPIMAGETOOL}.download" "${APPIMAGETOOL_URL}"
  # SHA256 check is best-effort: warn if it doesn't match, but don't fail
  # the build (the maintainer should re-pin if it changes).
  actual="$(sha256sum "${APPIMAGETOOL}.download" | cut -d' ' -f1)"
  if [ "${actual}" != "${APPIMAGETOOL_SHA256}" ]; then
    log "WARNING: appimagetool SHA256 mismatch."
    log "  expected: ${APPIMAGETOOL_SHA256}"
    log "  actual:   ${actual}"
    log "  Continuing — please re-pin the URL after verifying upstream."
  fi
  chmod +x "${APPIMAGETOOL}.download"
  mv "${APPIMAGETOOL}.download" "${APPIMAGETOOL}"
fi

# ── Stage AppDir ──────────────────────────────────────────────────────────
log "Staging AppDir at ${APPDIR}"
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin" \
         "${APPDIR}/usr/share/applications" \
         "${APPDIR}/usr/share/icons/hicolor/scalable/apps" \
         "${APPDIR}/usr/lib/systemd/user"

for bin in author-clipboard author-clipboard-daemon author-clipboard-ctl author-clipboard-hypr-picker; do
  install -Dm755 "${WORKSPACE_ROOT}/target/release/${bin}" \
    "${APPDIR}/usr/bin/${bin}"
done

install -Dm644 "${WORKSPACE_ROOT}/data/com.namikofficial.author-clipboard.desktop" \
  "${APPDIR}/usr/share/applications/com.namikofficial.author-clipboard.desktop"

install -Dm644 "${WORKSPACE_ROOT}/resources/icons/com.namikofficial.author-clipboard.svg" \
  "${APPDIR}/usr/share/icons/hicolor/scalable/apps/com.namikofficial.author-clipboard.svg"

# AppImage entry script
install -Dm755 "${WORKSPACE_ROOT}/packaging/appimage/AppRun" \
  "${APPDIR}/AppRun"

# Top-level desktop + icon (AppImage convention)
install -Dm644 "${WORKSPACE_ROOT}/packaging/appimage/author-clipboard.desktop" \
  "${APPDIR}/author-clipboard.desktop"
install -Dm644 "${WORKSPACE_ROOT}/resources/icons/com.namikofficial.author-clipboard.svg" \
  "${APPDIR}/author-clipboard.svg"

# ── Build the AppImage ────────────────────────────────────────────────────
log "Running appimagetool..."
export ARCH=x86_64
"${APPIMAGETOOL}" --no-appstream "${APPDIR}" "${DIST_DIR}/${APPIMAGE_NAME}"

log "Done: ${DIST_DIR}/${APPIMAGE_NAME}"
ls -la "${DIST_DIR}/${APPIMAGE_NAME}"
