# justfile for author-clipboard development
# Usage: just <command>
# Install just: https://github.com/casey/just

# Default task - show available commands
default:
    @just --list

# Build all crates
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run quick checks without full compilation
check:
    cargo check

# Format all code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run clippy with fixes
lint-fix:
    cargo clippy --fix

# Run all tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Generate and open documentation
docs:
    cargo doc --open

# Development mode - watch for changes and rebuild
dev:
    cargo watch -x check

# Run the clipboard daemon
daemon:
    cargo run -p author-clipboard-daemon

# Run the applet
applet:
    cargo run -p author-clipboard-applet

# Run daemon in background for development
daemon-bg:
    cargo run -p author-clipboard-daemon &

# Full development check (format, lint, test, build)
verify: fmt lint test build
    @echo "✅ All checks passed!"

# Setup development environment
setup:
    @echo "🔧 Setting up author-clipboard development environment..."
    rustup component add rustfmt clippy rust-analyzer
    @echo "📋 Installing additional tools..."
    cargo install cargo-watch
    @echo "✅ Development environment ready!"

# Clean slate - remove all build artifacts and lock files
reset: clean
    rm -f Cargo.lock

# Quick development cycle
quick: check test
    @echo "⚡ Quick check complete!"

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
    @echo "Rust version:"
    @rustc --version
    @echo "Cargo version:" 
    @cargo --version
    @echo "Checking Wayland support..."
    @which wl-copy || echo "❌ wl-clipboard not found - install with: sudo apt install wl-clipboard"
    @echo "Checking workspace structure..."
    @ls -la crates/
    @echo "✅ Health check complete!"