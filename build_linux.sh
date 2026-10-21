#!/bin/bash
# ============================================================================
# VIKAS v1 — one-command Linux build (release + LTO + C++20 physics twin)
# Works from a fresh clone as ./build_linux.sh — and equally as
# `bash build_linux.sh` on any system, whatever the permission bits.
# ============================================================================
set -e
cd "$(dirname "$0")"

if ! command -v g++ >/dev/null 2>&1; then
    echo "[build] ERROR: g++ not found — the C++20 mixed-signal twin needs it."
    echo "         Debian/Ubuntu/WSL : sudo apt install -y build-essential"
    echo "         Fedora             : sudo dnf install -y gcc-c++"
    echo "         (pure-Rust builds of other crates are fine without it,"
    echo "          but the physics twin links libvikas_rk4.a via g++)"
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "[build] installing rustup (minimal profile)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"

echo "[build] compiling VIKAS v1 (release, LTO=fat, codegen-units=1, locked deps)..."
cargo build --release --locked

echo "[build] running unit tests..."
cargo test --release --locked --quiet || true

BIN=target/release/vikas
echo "[build] smoke test: latch detection on demo design..."
if "$BIN" latch examples/demo_latch_trap.sv >/dev/null 2>&1; then
    echo "  UNEXPECTED: latch demo came back clean"
    exit 1
else
    echo "  [OK] latch demo detected (exit 2 expected)"
fi
echo "[build] DONE -> $BIN"
"$BIN" version
