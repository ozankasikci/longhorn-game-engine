# Longhorn Game Engine Makefile

# Default target
.PHONY: all
all: check

# Run all checks (clippy, format check, and tests)
.PHONY: check
check: clippy fmt-check test

# Run core checks (format, clippy on core crates, and tests)
.PHONY: check-core
check-core: fmt-check clippy-core test

# Run clippy on all targets
.PHONY: clippy
clippy:
	@echo "🔍 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Run clippy on core crates only (excludes problematic graphics crates)
.PHONY: clippy-core
clippy-core:
	@echo "🔍 Running clippy on core crates..."
	cargo clippy -p engine-renderer-core -p engine-renderer-ecs-bridge -p engine-ecs-core \
		-p engine-components-3d -p engine-renderer-3d -p structure-validator \
		--all-targets -- -D warnings

# Run rustfmt check
.PHONY: fmt-check
fmt-check:
	@echo "📐 Checking formatting..."
	cargo fmt --all -- --check

# Format code
.PHONY: fmt
fmt:
	@echo "✨ Formatting code..."
	cargo fmt --all

# Run all tests
.PHONY: test
test:
	@echo "🧪 Running tests..."
	cargo test --all

# Run tests with output
.PHONY: test-verbose
test-verbose:
	@echo "🧪 Running tests (verbose)..."
	cargo test --all -- --nocapture

# Quick check - format and clippy only
.PHONY: quick
quick: fmt clippy

# Clean build artifacts
.PHONY: clean
clean:
	@echo "🧹 Cleaning..."
	cargo clean

# Build all crates
.PHONY: build
build:
	@echo "🔨 Building..."
	cargo build --all

# Build release
.PHONY: release
release:
	@echo "🚀 Building release..."
	cargo build --all --release

# Run the editor
.PHONY: run
run:
	@echo "🎮 Running editor..."
	cargo run --bin longhorn-editor

# Check structure
.PHONY: check-structure
check-structure:
	@echo "🏗️ Checking crate structure..."
	cargo run --bin validate-structure

# Fix structure issues
.PHONY: fix-structure
fix-structure:
	@echo "🔧 Fixing crate structure..."
	cargo run --bin migrate-structure

# Update dependencies
.PHONY: update
update:
	@echo "📦 Updating dependencies..."
	cargo update

# Run benchmarks
.PHONY: bench
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench --all

# Check for outdated dependencies
.PHONY: outdated
outdated:
	@echo "🔍 Checking for outdated dependencies..."
	cargo outdated

# Run security audit
.PHONY: audit
audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Full CI pipeline
.PHONY: ci
ci: fmt-check clippy test check-structure

# Help
.PHONY: help
help:
	@echo "Longhorn Game Engine - Available commands:"
	@echo ""
	@echo "  make check          - Run clippy, format check, and tests"
	@echo "  make clippy         - Run clippy linter"
	@echo "  make fmt            - Format code with rustfmt"
	@echo "  make fmt-check      - Check code formatting"
	@echo "  make test           - Run all tests"
	@echo "  make test-verbose   - Run tests with output"
	@echo "  make quick          - Quick check (format + clippy)"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make build          - Build all crates"
	@echo "  make release        - Build release version"
	@echo "  make run            - Run the editor"
	@echo "  make check-structure - Validate crate structure"
	@echo "  make fix-structure  - Fix crate structure issues"
	@echo "  make update         - Update dependencies"
	@echo "  make bench          - Run benchmarks"
	@echo "  make outdated       - Check for outdated dependencies"
	@echo "  make audit          - Run security audit"
	@echo "  make ci             - Run full CI pipeline"
	@echo "  make help           - Show this help message"