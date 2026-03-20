# Panchang — development commands
# Run `just` to see all available recipes

# Default: list all recipes
default:
    @just --list

# --- Build ---

# Build the Rust extension for development
build:
    maturin develop --uv

# Build release wheels
build-release:
    maturin build --release --out dist

# --- Test ---

# Run all tests
test: build
    uv run pytest tests/ -v

# Run Rust tests only
test-rust:
    cargo test --manifest-path crates/panchang-core/Cargo.toml -- --test-threads=1

# Run Python tests only
test-python:
    uv run pytest tests/ -v

# --- Lint ---

# Run all linters
lint:
    cargo clippy --manifest-path crates/panchang-core/Cargo.toml -- -D warnings
    cargo fmt --manifest-path crates/panchang-core/Cargo.toml -- --check
    uv run ruff check python/ tests/
    uv run ruff format --check python/ tests/

# Auto-fix lint issues
fix:
    cargo fmt --manifest-path crates/panchang-core/Cargo.toml
    uv run ruff check --fix python/ tests/
    uv run ruff format python/ tests/

# --- Version ---

# Show current version
version:
    @echo "pyproject.toml: $(grep '^version' pyproject.toml | head -1 | sed 's/.*= *"//' | sed 's/"//')"
    @echo "Cargo.toml:     $(grep '^version' crates/panchang-core/Cargo.toml | head -1 | sed 's/.*= *"//' | sed 's/"//')"

# Bump version in pyproject.toml and Cargo.toml. Usage: just bump 0.2.0
bump new_version:
    #!/usr/bin/env bash
    set -euo pipefail

    # Validate semver format
    if ! echo "{{new_version}}" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
        echo "Error: '{{new_version}}' is not valid semver (expected: X.Y.Z or X.Y.Z-pre)"
        exit 1
    fi

    old_version=$(grep '^version' pyproject.toml | head -1 | sed 's/.*= *"//' | sed 's/"//')
    echo "Bumping version: $old_version → {{new_version}}"

    # Update pyproject.toml
    sed -i 's/^version = ".*"/version = "{{new_version}}"/' pyproject.toml

    # Update Cargo.toml
    sed -i '0,/^version = ".*"/{s/^version = ".*"/version = "{{new_version}}"/}' crates/panchang-core/Cargo.toml

    # Update Cargo.lock
    cargo generate-lockfile --manifest-path crates/panchang-core/Cargo.toml 2>/dev/null || true

    echo ""
    echo "Done! Version bumped to {{new_version}}"
    echo ""
    echo "Next steps:"
    echo "  1. git add pyproject.toml crates/panchang-core/Cargo.toml Cargo.lock"
    echo "  2. git commit -m 'release: v{{new_version}}'"
    echo "  3. git push origin main"
    echo "  4. Create a GitHub Release with tag v{{new_version}} → triggers PyPI publish"

# --- Benchmarks ---

# Run Rust benchmarks
bench:
    cargo bench --manifest-path crates/panchang-core/Cargo.toml
