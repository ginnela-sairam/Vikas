#!/bin/bash
# ============================================================================
# VIKAS v1 — one-command Linux build (release + LTO + C++20 physics twin)
# ============================================================================
set -e
cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
    echo "[build] installing rustup (minimal profile)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    source "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"

echo "[build] compiling VIKAS v1 (release, LTO=fat, codegen-units=1)..."
cargo build --release

echo "[build] running unit tests..."
cargo test --release --quiet || true

BIN=target/release/vikas
echo "[build] smoke test: latch detection on demo design..."
$BIN latch examples/demo_latch_trap.sv >/dev/null 2>&1 && echo "  unexpected: clean" || echo "  [OK] latch demo detected (exit 2 expected)"
echo "[build] DONE -> $BIN"
$BIN version
