#!/bin/bash
# ============================================================================
# VIKAS v1 — one-command Linux installer (sudo-less by design)
# Installs to ~/.local/bin — no root, no system pollution.
# Works as ./install_linux.sh after a fresh clone, or `bash install_linux.sh`
# on any system, whatever the permission bits.
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
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
    echo "[install] building VIKAS v1 from source (release + LTO, locked deps)..."
    (cd "$V_DIR" && cargo build --release --locked)
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
echo "[install] DONE."
# current-session usability check — the #1 fresh-install confusion:
# the shell that ran the installer does not re-read .bashrc by itself.
case ":$PATH:" in
    *":$DEST:"*)
        echo "[install] $DEST is on PATH in this shell — 'vikas' works right now."
        ;;
    *)
        echo "[install] NOTE: '$DEST' is not on PATH in THIS shell yet."
        echo "         run    : export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo "         or just: open a new terminal (already wired into .bashrc)"
        ;;
esac
echo ""
echo "[install] sanity check: $($DEST/vikas version | tail -1)"
echo ""
echo "Try:"
echo "  vikas help"
echo "  vikas run  $V_DIR/examples/multifile_proj/project.f   # multi-file manifest"
echo "  vikas latch $V_DIR/examples/demo_latch_trap.sv        # breakthrough 13"
echo "  vikas run  $V_DIR/examples/demo_riscv_alu_hazard.sv    # golden demo"
echo "  vikas gui                                            # browser window pops open (WSL-aware)"
echo "  vikas tui .vikas/vikas_wave.vwf                       # terminal UI"
