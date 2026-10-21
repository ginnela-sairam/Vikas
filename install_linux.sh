#!/bin/bash
# ============================================================================
# VIKAS v1 — one-command Linux installer
# ============================================================================
set -e
V_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -f "$V_DIR/target/release/vikas" ]; then
    BIN="$V_DIR/target/release/vikas"
    echo "[install] using pre-built binary: $BIN"
else
    if ! command -v cargo >/dev/null 2>&1; then
        echo "[install] cargo not found — installing rustup (minimal profile)"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        source "$HOME/.cargo/env"
    fi
    echo "[install] building VIKAS v1 from source (release + LTO)..."
    (cd "$V_DIR" && cargo build --release)
    BIN="$V_DIR/target/release/vikas"
fi

DEST="$HOME/.local/bin"
mkdir -p "$DEST"
install -m 755 "$BIN" "$DEST/vikas"

SHM_DIR="/dev/shm/vikas_$(id -u)"
if mkdir -p "$SHM_DIR" 2>/dev/null; then
    echo "[install] /dev/shm arena ready: $SHM_DIR (zero-disk-wear, RAM speed)"
fi

SHELL_RC="$HOME/.bashrc"
[ -n "$ZSH_VERSION" ] && SHELL_RC="$HOME/.zshrc"
if ! grep -q "vikas alias quick" "$SHELL_RC" 2>/dev/null; then
    cat >> "$SHELL_RC" << 'RC'

# VIKAS v1 EDA platform
export PATH="$HOME/.local/bin:$PATH"
alias vwatch='vikas watch'
RC
fi

echo ""
echo "[install] DONE. Try:"
echo "  vikas help"
echo "  vikas run $V_DIR/examples/multifile_proj/project.f   # multi-file manifest"
echo "  vikas latch $V_DIR/examples/demo_latch_trap.sv       # breakthrough 13"
echo "  vikas tui .vikas/vikas_wave.vwf                       # terminal UI"
echo "  vikas gui .vikas/vikas_wave.vwf                       # HTML5 GUI (offline)"
