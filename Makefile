.PHONY: build release test lint fmt check clean install reinstall uninstall

# Default target
all: check test build

# Debug build
build:
	cargo build

# Optimized release build
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Run tests with verbose output
test-verbose:
	cargo test --verbose

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Check formatting
fmt:
	cargo fmt -- --check

# Auto-fix formatting
fmt-fix:
	cargo fmt

# Full CI check (format + lint + test)
check: fmt lint test

# Install binary to ~/.cargo/bin
install:
	cargo install --path .

# Uninstall binary from wherever cargo installed it
uninstall:
	cargo uninstall differ_helper 2>/dev/null || true

# Remove old version and install fresh from current source
reinstall: uninstall install

# Clean build artifacts
clean:
	cargo clean

# Build release binaries for multiple platforms (requires cross)
# Install cross: cargo install cross
cross-linux:
	cross build --release --target x86_64-unknown-linux-gnu

cross-linux-arm:
	cross build --release --target aarch64-unknown-linux-gnu

cross-windows:
	cross build --release --target x86_64-pc-windows-gnu

cross-all: cross-linux cross-linux-arm cross-windows release
	@echo "Binaries:"
	@echo "  macOS:       target/release/differ_helper"
	@echo "  Linux x86:   target/x86_64-unknown-linux-gnu/release/differ_helper"
	@echo "  Linux ARM:   target/aarch64-unknown-linux-gnu/release/differ_helper"
	@echo "  Windows:     target/x86_64-pc-windows-gnu/release/differ_helper.exe"
