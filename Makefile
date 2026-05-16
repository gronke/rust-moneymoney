.PHONY: help test lint fmt fmt-check version-check check doc audit clean all

# Default target
help:
	@echo "Available targets:"
	@echo "  make test          - Run all tests (lib + CLI)"
	@echo "  make lint          - Run clippy linter"
	@echo "  make fmt           - Format code with rustfmt"
	@echo "  make version-check - Verify lib/CLI version alignment"
	@echo "  make check         - Run all checks (fmt + lint + version + test + doc)"
	@echo "  make doc           - Build documentation"
	@echo "  make audit         - Run security audit"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make all           - Run fmt + check"

# Run tests
test:
	@echo "Running library unit tests..."
	@cargo test --lib
	@echo "\nRunning library doc tests..."
	@cargo test --doc
	@echo "\nRunning CLI tests..."
	@cargo test -p moneymoney-cli --all-features

# Run clippy linter
lint:
	@echo "Running clippy..."
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all

# Check formatting
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --all -- --check

# Build documentation
doc:
	@echo "Building documentation..."
	@cargo doc --no-deps --all-features

# Security audit
audit:
	@echo "Running security audit..."
	@cargo audit

# Workspace version alignment (lib vs cli)
version-check:
	@echo "Checking workspace version alignment..."
	@./scripts/check_version_alignment.py

# Run all checks
check: fmt-check lint version-check test doc
	@echo "\n✅ All checks passed!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Run all (format + check)
all: fmt check
