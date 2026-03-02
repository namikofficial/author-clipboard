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
    # Keep ~/.cargo/bin in sync if it exists and appears before ~/.local/bin in PATH.
    if [ -d "$HOME/.cargo/bin" ]; then install -Dm755 target/release/author-clipboard-daemon ~/.cargo/bin/author-clipboard-daemon; fi
    if [ -d "$HOME/.cargo/bin" ]; then install -Dm755 target/release/author-clipboard ~/.cargo/bin/author-clipboard; fi
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
    rm -f ~/.cargo/bin/author-clipboard-daemon
    rm -f ~/.cargo/bin/author-clipboard
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
