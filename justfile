# justfile for author-clipboard development
# Usage: just <command>
# Install just: https://github.com/casey/just

# Load .env file if it exists (RUST_LOG, COSMIC_DATA_CONTROL_ENABLED, etc.)
set dotenv-load := true

# Default task - show available commands
default:
    @just --list

# ── Build ──────────────────────────────────────────────────────────────

# Build all crates
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run quick checks without full compilation
check:
    cargo check --all-targets

# ── Code Quality ───────────────

# Format all code
fmt:
    cargo fmt --all

# Check formatting without changes
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with auto-fix
lint-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# ── Testing ────────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test --all

# Run tests with output
test-verbose:
    cargo test --all -- --nocapture

# ── Full Verification ──────────────────────────────────────────────────

# Full development check (format, lint, test, build)
verify: fmt-check lint test build
    @echo "✅ All checks passed!"

# Format + fix lints, then verify
fix: fmt lint-fix
    @echo "✅ Auto-fixes applied. Run 'just verify' to confirm."

# ── Run ────────────────────────────────────────────────────────────────

# Build profile: set BUILD_PROFILE=release in .env for release builds
build_flag := if env("BUILD_PROFILE", "debug") == "release" { "--release" } else { "" }

# Run the clipboard daemon
daemon:
    cargo run -p author-clipboard-daemon {{ build_flag }}

# Run the applet
applet:
    cargo run -p author-clipboard-applet {{ build_flag }}

# Run daemon in background for development
daemon-bg:
    cargo run -p author-clipboard-daemon {{ build_flag }} &

# Run both daemon and applet for end-to-end testing
run: build
    @echo "🚀 Starting daemon in background + applet in foreground..."
    @echo "   Copy text anywhere → it appears in the applet window"
    @echo "   Press Ctrl+C to stop both"
    @echo ""
    cargo run -p author-clipboard-daemon {{ build_flag }} &
    @sleep 1
    cargo run -p author-clipboard-applet {{ build_flag }}

# Development mode - watch for changes and rebuild
dev:
    cargo watch -x check

# ── Maintenance ────────────────────────────────────────────────────────

# Generate/update CHANGELOG.md from conventional commits
changelog:
    git-cliff --output CHANGELOG.md
    @echo "📝 CHANGELOG.md updated"

# Preview changelog without writing (dry-run)
changelog-preview:
    git-cliff --unreleased

# Tag a release and generate changelog (usage: just release 0.2.0)
release version:
    @echo "🚀 Releasing v{{version}}..."
    git-cliff --tag "v{{version}}" --output CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore(release): v{{version}}"
    git tag -a "v{{version}}" -m "Release v{{version}}"
    @echo "✅ Release v{{version}} created. Push with: git push && git push --tags"

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Clean slate - remove all build artifacts and lock files
reset: clean
    rm -f Cargo.lock

# Generate and open documentation
docs:
    cargo doc --open

# Quick development cycle
quick: check test
    @echo "⚡ Quick check complete!"

# ── Install / Uninstall ────────────────────────────────────────────────

# Install binaries, .desktop file, icon, and systemd service
install: build-release
    @echo "📦 Installing author-clipboard..."
    install -Dm755 target/release/author-clipboard-daemon ~/.local/bin/author-clipboard-daemon
    install -Dm755 target/release/author-clipboard ~/.local/bin/author-clipboard
    install -Dm755 target/release/author-clipboard-ctl ~/.local/bin/author-clipboard-ctl
    install -Dm755 target/release/author-clipboard-hypr-picker ~/.local/bin/author-clipboard-hypr-picker
    # Keep ~/.cargo/bin in sync if it exists and appears before ~/.local/bin in PATH.
    if [ -d "$HOME/.cargo/bin" ]; then install -Dm755 target/release/author-clipboard-daemon ~/.cargo/bin/author-clipboard-daemon; fi
    if [ -d "$HOME/.cargo/bin" ]; then install -Dm755 target/release/author-clipboard ~/.cargo/bin/author-clipboard; fi
    if [ -d "$HOME/.cargo/bin" ]; then install -Dm755 target/release/author-clipboard-ctl ~/.cargo/bin/author-clipboard-ctl; fi
    if [ -d "$HOME/.cargo/bin" ]; then install -Dm755 target/release/author-clipboard-hypr-picker ~/.cargo/bin/author-clipboard-hypr-picker; fi
    install -Dm644 data/com.namikofficial.author-clipboard.desktop ~/.local/share/applications/com.namikofficial.author-clipboard.desktop
    install -Dm644 resources/icons/com.namikofficial.author-clipboard.svg ~/.local/share/icons/hicolor/scalable/apps/com.namikofficial.author-clipboard.svg
    install -Dm644 data/author-clipboard-daemon.service ~/.config/systemd/user/author-clipboard-daemon.service
    systemctl --user daemon-reload
    @echo "✅ Installed! Enable daemon with: just enable"

# Enable and start the clipboard daemon service
enable:
    systemctl --user enable --now author-clipboard-daemon.service
    @echo "✅ Daemon enabled and started"

# Disable and stop the clipboard daemon service
disable:
    systemctl --user disable --now author-clipboard-daemon.service
    @echo "🛑 Daemon disabled"

# Check daemon service status
status:
    systemctl --user status author-clipboard-daemon.service

# View daemon logs
logs:
    journalctl --user -u author-clipboard-daemon.service -f

# Uninstall everything
uninstall: disable
    rm -f ~/.local/bin/author-clipboard-daemon
    rm -f ~/.local/bin/author-clipboard
    rm -f ~/.local/bin/author-clipboard-ctl
    rm -f ~/.local/bin/author-clipboard-hypr-picker
    rm -f ~/.cargo/bin/author-clipboard-daemon
    rm -f ~/.cargo/bin/author-clipboard
    rm -f ~/.cargo/bin/author-clipboard-ctl
    rm -f ~/.cargo/bin/author-clipboard-hypr-picker
    rm -f ~/.local/share/applications/com.namikofficial.author-clipboard.desktop
    rm -f ~/.local/share/icons/hicolor/scalable/apps/com.namikofficial.author-clipboard.svg
    rm -f ~/.config/systemd/user/author-clipboard-daemon.service
    systemctl --user daemon-reload
    @echo "🗑️  Uninstalled author-clipboard"

# ── Debian Packaging ───────────────────────────────────────────────────

# Build a .deb package (requires cargo-deb: cargo install cargo-deb)
deb:
	@echo "Building release binaries first..."
	cargo build --release --workspace
	@echo "Building .deb package..."
	cargo deb -p author-clipboard-applet --no-build
	@echo ""
	@echo "Package built in target/debian/"
	ls -la target/debian/*.deb 2>/dev/null || echo "No .deb found - check errors above"

# Build .deb and install it locally (for testing)
deb-install: deb
	@echo "Installing .deb locally (requires sudo)..."
	sudo dpkg -i target/debian/author-clipboard_*.deb

# Show what files would be in the .deb (dry run)
deb-check:
	cargo deb -p author-clipboard-applet --no-build --no-strip -- --verbose 2>&1 | head -60 || \
	cargo deb -p author-clipboard-applet --no-build 2>&1 | head -60

# Remove locally installed .deb
deb-remove:
	@echo "Removing author-clipboard package..."
	sudo dpkg -r author-clipboard

# Inspect the contents of a built .deb (binaries, paths, postinst)
deb-inspect: deb
	@DEB=$$(ls target/debian/author-clipboard_*.deb | head -1); \
		echo "── metadata ──"; \
		dpkg-deb -I "$$DEB"; \
		echo ""; \
		echo "── contents ──"; \
		dpkg-deb -c "$$DEB" | grep -E "(usr/bin/author-clipboard|usr/lib/systemd|usr/share/)"

# ── Release Artifacts ──────────────────────────────────────────────────

# Build a release tarball containing the four binaries.
release-archive: build-release
	@mkdir -p dist
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
		tar -C target/release -cJf \
			"dist/author-clipboard-$$VERSION-linux-x86_64.tar.xz" \
			author-clipboard \
			author-clipboard-daemon \
			author-clipboard-ctl \
			author-clipboard-hypr-picker
	@ls -la dist/*.tar.xz

# Generate SHA256SUMS for all artifacts in dist/.
release-checksums:
	@cd dist && sha256sum -- *.deb *.tar.xz *.xml > SHA256SUMS 2>/dev/null || \
		sha256sum -- *.tar.xz > SHA256SUMS
	@cat dist/SHA256SUMS

# Sign SHA256SUMS with GPG. Requires a GPG key on the local keyring.
release-sign:
	@if [ ! -f dist/SHA256SUMS ]; then echo "Run 'just release-checksums' first."; exit 1; fi
	@cd dist && gpg --armor --detach-sign --output SHA256SUMS.asc SHA256SUMS
	@echo "Signed: dist/SHA256SUMS.asc"
	@gpg --verify dist/SHA256SUMS.asc dist/SHA256SUMS

# Verify a signed release locally (downloads artifacts via `gh`).
release-verify version:
	@gh release download "v{{version}}" --dir dist/verify
	@cd dist/verify && sha256sum -c SHA256SUMS
	@if [ -f dist/verify/SHA256SUMS.asc ]; then \
		gpg --verify dist/verify/SHA256SUMS.asc dist/verify/SHA256SUMS; \
	fi

# Bundle the AUR PKGBUILD + .SRCINFO into a single tarball.
aur-bundle:
	@mkdir -p dist
	@tar -C packaging/arch -czf dist/author-clipboard-aur-files.tar.gz PKGBUILD .SRCINFO
	@ls -la dist/author-clipboard-aur-files.tar.gz

# Verify .SRCINFO matches the current PKGBUILD.
aur-check:
	@cd packaging/arch && makepkg --printsrcinfo > .SRCINFO.new
	@if diff -u packaging/arch/.SRCINFO packaging/arch/.SRCINFO.new; then \
		echo "✓ .SRCINFO is in sync with PKGBUILD"; \
		rm packaging/arch/.SRCINFO.new; \
	else \
		echo "✗ .SRCINFO is OUT OF SYNC with PKGBUILD."; \
		echo "  Fix with: cd packaging/arch && makepkg --printsrcinfo > .SRCINFO"; \
		exit 1; \
	fi

# ── Arch PKGBUILD ──────────────────────────────────────────────────────

# Build the Arch package via `makepkg` (Arch-only).
arch-build:
	@if ! command -v makepkg > /dev/null 2>&1; then \
		echo "makepkg not found. Run inside Arch Linux or archlinux:latest container."; \
		exit 1; \
	fi
	@cd packaging/arch && makepkg --nocheck --nodeps

# ── Flatpak ────────────────────────────────────────────────────────────

# Build the Flatpak (requires flatpak-builder + Freedesktop runtime).
flatpak-build:
	@if ! command -v flatpak-builder > /dev/null 2>&1; then \
		echo "flatpak-builder not installed. Install with your distro package manager."; \
		exit 1; \
	fi
	flatpak-builder --user --force-clean build-dir \
		packaging/flatpak/com.namikofficial.author-clipboard.yml
	@echo "✓ Built Flatpak into build-dir/"
	@echo "Install with: flatpak-builder --user --install build-dir packaging/flatpak/com.namikofficial.author-clipboard.yml"

# Validate Flatpak manifest YAML (no full build required).
flatpak-validate:
	@python3 -c "import yaml; yaml.safe_load(open('packaging/flatpak/com.namikofficial.author-clipboard.yml'))" && \
		echo "✓ Flatpak manifest is valid YAML"

# ── AppImage ───────────────────────────────────────────────────────────

# Build an AppImage (requires cargo build --release --workspace to have run).
appimage-build: build-release
	@bash packaging/appimage/build.sh

# Validate AppImage build script syntax.
appimage-check:
	@bash -n packaging/appimage/build.sh && echo "✓ packaging/appimage/build.sh is valid"
	@bash -n packaging/appimage/AppRun && echo "✓ packaging/appimage/AppRun is valid"

# ── Nix ────────────────────────────────────────────────────────────────

# Run `nix flake check` (requires nix with flakes enabled).
nix-check:
	@if ! command -v nix > /dev/null 2>&1; then \
		echo "nix not installed. See https://nixos.org/download.html"; \
		exit 1; \
	fi
	nix flake check --no-build

# Build with nix (default package only).
nix-build:
	@if ! command -v nix > /dev/null 2>&1; then \
		echo "nix not installed. See https://nixos.org/download.html"; \
		exit 1; \
	fi
	nix build
	@ls -la result/bin/

# ── Setup ──────────────────────────────────────────────────────────────

# Setup development environment (first-time)
setup: setup-hooks
    @echo "🔧 Setting up author-clipboard development environment..."
    rustup component add rustfmt clippy rust-analyzer
    @echo "📋 Installing additional tools..."
    cargo install cargo-watch
    @echo "✅ Development environment ready!"

# Install git hooks (conventional commits + pre-commit checks)
setup-hooks:
    @echo "🪝 Installing git hooks..."
    git config core.hooksPath .githooks
    @echo "✅ Git hooks installed (pre-commit: fmt+clippy, commit-msg: conventional commits)"

# Install system dependencies (Ubuntu/Debian)
install-deps:
    @echo "📦 Installing system dependencies..."
    sudo apt update
    sudo apt install -y \
        build-essential pkg-config cmake git curl wget \
        libssl-dev libsqlite3-dev \
        libwayland-dev libwayland-client0 wayland-protocols \
        libxkbcommon-dev libdbus-1-dev \
        libexpat1-dev libfontconfig-dev libfreetype-dev \
        libgtk-4-dev libudev-dev libinput-dev libgbm-dev \
        libseat-dev libxcb-render0-dev libxcb-shape0-dev \
        libxcb-xfixes0-dev wl-clipboard
    @echo "✅ System dependencies installed!"

# Check for potential issues
doctor:
    @echo "🩺 Running project health check..."
    @echo ""
    @echo "── Toolchain ──"
    @rustc --version
    @cargo --version
    @echo ""
    @echo "── Components ──"
    @rustup component list --installed | grep -E "(rustfmt|clippy|rust-analyzer)" || echo "❌ Missing components - run: just setup"
    @echo ""
    @echo "── Git Hooks ──"
    @git config core.hooksPath && echo "✅ Git hooks configured" || echo "❌ Git hooks not installed - run: just setup-hooks"
    @echo ""
    @echo "── Wayland ──"
    @which wl-copy > /dev/null 2>&1 && echo "✅ wl-clipboard found" || echo "❌ wl-clipboard not found - run: just install-deps"
    @echo ""
    @echo "── Workspace ──"
    @ls crates/
    @echo ""
    @echo "✅ Health check complete!"
