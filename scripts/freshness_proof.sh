#!/bin/bash
# ============================================================================
# VIKAS v1 — FRIEND-FAILURE END-TO-END REPLAY (breakthrough #25 proof)
#
# Replays, in an isolated /tmp sandbox, EXACTLY the real-world failure a
# friend hit: pulled new code, ran the old binary, "gui didn't work",
# nothing told him why. Then proves the self-healing machinery kills every
# link of that failure chain:
#
#   A. friend clones, builds, installs              -> binary fresh (stamp = HEAD)
#   B. upstream advances + friend pulls manually    -> STALE warning fires
#   C. friend runs 'vikas update' (stale binary,
#      no new remote commits)                       -> rebuilds itself -> fresh
#   D. upstream advances again, plain 'vikas update'-> pulls + rebuilds -> fresh
#   E. dirty-tree safety                           -> refuses, stash advice
#
# Runs against a local "GitHub stand-in" — no network. Uses LTO-off /
# codegen-units=64 for build speed; stamping depends only on the git
# checkout, not the profile.
#
# Usage: bash scripts/freshness_proof.sh     (from the repo root)
# ============================================================================
set -uo pipefail

SRC="$(pwd)"
W=/tmp/vikas_freshness_proof
EV="$SRC/qa_results/freshness_proof.txt"
REAL_HOME="$HOME"
export VIKAS_NO_BANNER=1

step() { echo "" | tee -a "$EV"; echo "=== $1 ===" | tee -a "$EV"; }
say()  { echo "$1" | tee -a "$EV"; }
die()  { echo "PROOF FAILED: $1" | tee -a "$EV"; exit 1; }
# run a chatty command, keep the full log in the sandbox, append the tail
run_quiet() { local log="$W/chatty.log"; "$@" >"$log" 2>&1; local rc=$?;
    echo "--- (log tail) ---" >> "$EV"; tail -4 "$log" | tee -a "$EV"; return $rc; }

[ -f Cargo.toml ] || { echo "run from the repo root"; exit 1; }
mkdir -p "$(dirname "$EV")"
echo "VIKAS freshness proof — $(date -Is)" > "$EV"

rm -rf "$W"
mkdir -p "$W" "$W/home"
# isolated install target, shared cargo registry (no network needed)
export HOME="$W/home"
export PATH="$REAL_HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_HOME="$REAL_HOME/.cargo"
export RUSTUP_HOME="$REAL_HOME/.rustup"
export CARGO_PROFILE_RELEASE_LTO=false
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=64

G() { git -c user.email=proof@vikas -c user.name=proof "$@"; }

# ----------------------------------------------------------------------------
step "A. friend clones the 'GitHub' stand-in, builds, installs"
mkdir -p "$W/upstream/Vikas" "$W/friend"
tar -C "$SRC" -cf - --exclude=./.git --exclude=./target \
        --exclude=./.vikas --exclude=./.vikas_matrix . \
  | tar -C "$W/upstream/Vikas" -xf -
( cd "$W/upstream/Vikas" && git init -q . && git add -A -f . \
  && G commit -qm "VIKAS @ $(cd "$SRC" && git rev-parse --short HEAD)" ) || die "upstream init"
( cd "$W/upstream/Vikas" && git clone -q . "$W/friend/Vikas" ) || die "friend clone"
cd "$W/friend/Vikas" || die "cd friend clone"
say "friend HEAD: $(git rev-parse --short HEAD)"
run_quiet bash install_linux.sh || die "friend install"
BIN="$HOME/.local/bin/vikas"
say "binary: $("$BIN" version | tail -1)"
"$BIN" doctor | grep -q "fresh ✓" || die "A: doctor must report fresh right after install"
say "A PASS: binary fresh immediately after install (doctor confirms)"

# ----------------------------------------------------------------------------
step "B. upstream moves on; friend pulls manually, runs the OLD binary"
( cd "$W/upstream/Vikas" \
  && G commit --allow-empty -qm "upstream: new feature" ) || die "upstream commit 1"
git remote set-url origin "$W/upstream/Vikas" >/dev/null 2>&1 || true
git fetch -q origin || die "friend fetch"
git pull --ff-only -q || die "friend pull"
OUT="$(VIKAS_FORCE_STALE_CHECK=1 "$BIN" version 2>&1)"
echo "$OUT" >> "$EV"
echo "$OUT" | grep -q "BINARY IS STALE" || die "B: stale warning must fire after pull+old binary"
say "B PASS: old binary after pull now WARNS loudly (the old world failed silently)"

# ----------------------------------------------------------------------------
step "C. friend runs 'vikas update' — no new remote commits, stale binary"
run_quiet "$BIN" update || die "C: vikas update (stale-only) failed"
"$BIN" doctor | grep -q "fresh ✓" || die "C: doctor must report fresh after update"
say "C PASS: 'vikas update' self-healed the stale binary"

# ----------------------------------------------------------------------------
step "D. upstream moves again; friend just runs 'vikas update'"
( cd "$W/upstream/Vikas" && G commit --allow-empty -qm "upstream: another feature" ) || die "upstream commit 2"
run_quiet "$BIN" update || die "D: vikas update (pull+rebuild) failed"
FRIEND_HEAD="$(git rev-parse --short HEAD)"
UP_HEAD="$(git -C "$W/upstream/Vikas" rev-parse --short HEAD)"
BIN_HEAD="$("$BIN" version | tail -1)"
say "upstream HEAD   : $UP_HEAD"
say "friend repo HEAD : $FRIEND_HEAD"
say "binary: $BIN_HEAD"
[ "$FRIEND_HEAD" == "$UP_HEAD" ] || die "D: friend must be AT upstream HEAD after update (got $FRIEND_HEAD vs $UP_HEAD)"
echo "$BIN_HEAD" | grep -q "v1.0.0+$FRIEND_HEAD" || die "D: stamp must equal friend HEAD after update"
say "D PASS: one command pulled + rebuilt + reinstalled, stamp now matches HEAD"

# ----------------------------------------------------------------------------
step "E. safety: dirty tree — update must refuse with stash advice"
touch local_change.sv
OUT="$("$BIN" update 2>&1)"; RC=$?
echo "$OUT" >> "$EV"
[ "$RC" -eq 2 ] || die "E: dirty-tree update must exit 2 (got $RC)"
echo "$OUT" | grep -q "git stash" || die "E: stash advice must be printed"
say "E PASS: local changes are never touched — refuses with exact stash recipe"

# ----------------------------------------------------------------------------
cd "$SRC"
step "ALL 5 SCENARIOS GREEN"
say "friend-failure chain (pull / stale binary / silent GUI death) is now:"
say "  detected instantly   (stale warning + doctor)"
say "  healed in one word   (vikas update)"
say "  never destructive    (dirty-tree guard)"
echo "PROOF COMPLETE" | tee -a "$EV"
rm -rf "$W"
exit 0
