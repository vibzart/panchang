#!/bin/bash
set -e

echo "=== Cargo check (core) ==="
cargo check --manifest-path crates/panchang-core/Cargo.toml 2>&1

echo "=== Maturin develop (core) ==="
maturin develop --release -m crates/panchang-core/Cargo.toml 2>&1

echo "=== Smoke test (core) ==="
python -c "from panchang._core import py_datetime_to_jd; print(py_datetime_to_jd(2000, 1, 1, 12, 0, 0.0))"

echo "=== Build complete ==="
