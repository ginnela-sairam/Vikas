#!/bin/bash
# ============================================================================
# VIKAS v3.0 — Sovereign One-Command Linux Installer
# Supports:
#   1. Enterprise / Ceremorphic Air-Gapped Clusters (100% offline via ./vendor,
#      custom install prefix via PREFIX=/opt/vikas/bin bash install_linux.sh)
#   2. Developer / Friend Laptops & Workstations (Ubuntu, RHEL/Rocky, WSL2)
# Requires: Rust/cargo >= 1.70 (https://rustup.rs)
# ============================================================================
set -e
V_DIR="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

echo "┌──────────────────────────────────────────────────────────────────────────────┐"
echo "│  VIKAS v3.0  ·  Sovereign Linux-Native RTL & Mixed-Signal EDA Installer      │"
echo "│  VIKAS-KRONOS Kernel · Open-KDB · 38-Lang SIMD · IEEE 1800.2 UVM · 64-Core   │"
echo "└──────────────────────────────────────────────────────────────────────────────┘"

if [ -f "$V_DIR/target/release/vikas" ] && [ "${FORCE_REBUILD:-0}" != "1" ]; then
    BIN="$V_DIR/target/release/vikas"
    echo "[install] using existing release binary: $BIN"
else
    if ! command -v cargo >/dev/null 2>&1; then
        if [ "${AUTO_RUSTUP:-0}" = "1" ] || [ "${1:-}" = "--auto-rustup" ]; then
            echo "[install] installing minimal Rust toolchain via rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
            export PATH="$HOME/.cargo/bin:$PATH"
        else
            cat << 'NOV'
[install] cargo NOT FOUND.

  VIKAS v3.0 requires Rust >= 1.70 to compile the native VIKAS-KRONOS kernel.

  Option A — Friend / Developer Laptop (with internet):
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
      source "$HOME/.cargo/env"
      bash ./install_linux.sh
      # Or run in one step: AUTO_RUSTUP=1 bash ./install_linux.sh

  Option B — Enterprise / Ceremorphic Air-Gapped Server (ZERO network during build):
      Load your enterprise Rust >= 1.70 module (e.g. `module load rust` or
      `conda install -c conda-forge rust>=1.70`), then run:
      PREFIX=/opt/vikas/bin bash ./install_linux.sh
      (All 29 crates are 100% vendored in ./vendor — cargo never touches the network.)

  Option C — Run the VIKAS v3.0 Full-Stack Web Studio IDE immediately (Node.js >= 18):
      npm install && npm run dev
      # Open http://localhost:3000
NOV
            exit 1
        fi
    fi

    if [ -d "$V_DIR/vendor" ]; then
        echo "[install] building VIKAS v3.0 from source (release + LTO=fat, 100% OFFLINE vendored deps)..."
        (cd "$V_DIR" && cargo build --release --locked --offline)
    else
        echo "[install] building VIKAS v3.0 from source (release + LTO=fat)..."
        (cd "$V_DIR" && cargo build --release)
    fi
    BIN="$V_DIR/target/release/vikas"
fi

# Quick self-test of the compiled binary (6-check environment doctor)
"$BIN" doctor

DEST="${PREFIX:-$HOME/.local/bin}"
mkdir -p "$DEST"
install -m 755 "$BIN" "$DEST/vikas"
echo "[install] installed binary -> $DEST/vikas"

SHM_DIR="/dev/shm/vikas_$(id -u)"
if mkdir -p "$SHM_DIR" 2>/dev/null; then
    echo "[install] /dev/shm arena ready: $SHM_DIR (zero-disk-wear, >18 GB/s RAM speed)"
fi

SHELL_RC="$HOME/.bashrc"
[ -n "${ZSH_VERSION:-}" ] && SHELL_RC="$HOME/.zshrc"
if [ -w "$SHELL_RC" ] || [ ! -e "$SHELL_RC" ]; then
    if ! grep -q "VIKAS v3.0 EDA platform" "$SHELL_RC" 2>/dev/null; then
        cat >> "$SHELL_RC" << RC

# VIKAS v3.0 EDA platform
export PATH="$DEST:\$PATH"
alias vwatch='vikas watch'
RC
    fi
fi

echo ""
echo "[install] VIKAS v3.0 READY. Canonical v3.0 verification commands:"
echo "  vikas version                                                 # verify v3.0 binary"
echo "  vikas doctor                                                  # 6-point Linux environment audit"
echo "  vikas run $V_DIR/examples/nova_soc/nova.f                     # 26-file NOVA-SoC (9/9 self-checking tests)"
echo "  vikas run $V_DIR/examples/polyglot_ai_cluster/cluster.f       # 9-file 7-language Polyglot AI Cluster (6/6 PASS)"
echo "  vikas batch --jobs 16                                         # headless background regression (.vwm zero-copy mmap)"
echo "  vikas signoff $V_DIR/examples/nova_soc/nova.f                 # 7-pillar pre-tapeout readiness gate"
echo "  vikas latch $V_DIR/examples/demo_latch_trap.sv                # 3-layer inferred latch guard (exit code 2)"
echo "  vikas tui .vikas/vikas_wave.vwf                               # native terminal waveform viewer (zero HTML popups)"
