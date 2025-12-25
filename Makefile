.PHONY: help test lint fmt check doc audit clean all

# Default target
help:
	@echo "Available targets:"
	@echo "  make test    - Run all tests (unit + doc)"
	@echo "  make lint    - Run clippy linter"
	@echo "  make fmt     - Format code with rustfmt"
	@echo "  make check   - Run all checks (fmt + lint + test + doc)"
	@echo "  make doc     - Build documentation"
	@echo "  make audit   - Run security audit"
	@echo "  make clean   - Clean build artifacts"
	@echo "  make all     - Run fmt + check"

# Run tests
test:
	@echo "Running unit tests..."
	@cargo test --lib
	@echo "\nRunning doc tests..."
	@cargo test --doc

# Run clippy linter
lint:
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

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

# Run all checks
check: fmt-check lint test doc
	@echo "\n✅ All checks passed!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Run all (format + check)
all: fmt check
