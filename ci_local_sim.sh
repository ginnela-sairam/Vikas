#!/usr/bin/env bash
# Local CI simulation — mirrors .github/workflows/ci.yml exactly.
# Usage: bash ci_local_sim.sh  (run from repo root)
set -u
export PATH="$HOME/.cargo/bin:$PATH"
VIKAS=./target/release/vikas
PASS=0; FAIL=0
declare -a FAILED

step() { # name, expected_rc, command...
  local name="$1" want="$2"; shift 2
  "$@" > /tmp/ci_step.log 2>&1; local got=$?
  if [ "$got" -eq "$want" ]; then
    echo "PASS  $name"; PASS=$((PASS+1))
  else
    echo "FAIL  $name (rc=$got want=$want)"; FAIL=$((FAIL+1)); FAILED+=("$name")
    tail -3 /tmp/ci_step.log | sed 's/^/        /'
  fi
}
step_grep() { # name, pattern, command...
  local name="$1" pat="$2"; shift 2
  local st
  "$@" 2>&1 | grep -q "$pat"
  st=("${PIPESTATUS[@]}")
  if [ "${st[1]}" -eq 0 ]; then echo "PASS  $name"; PASS=$((PASS+1))
  else echo "FAIL  $name (grep miss, cmd rc=${st[0]})"; FAIL=$((FAIL+1)); FAILED+=("$name"); fi
}

echo "══ JOB 1: exec-bits ══"
step "exec-bits" 0 bash -c 'for f in *.sh; do [ -f "$f" ] || continue; test -x "$f" || exit 1; done'

echo "══ JOB 2: kernel (build + unit tests) ══"
step "cargo-build" 0 cargo build --release
step "cargo-test"  0 cargo test --release

echo "══ JOB 3: matrix (85-check sweep) ══"
step "test-matrix" 0 bash test_matrix.sh

echo "══ JOB 4: proof (NOVA-SoC + goldens + signoff) ══"
rm -f /tmp/nova_studio.html
step "proof:version"  0 "$VIKAS" version
step "proof:doctor"   0 "$VIKAS" doctor
step_grep "proof:multifile-result200" "result=200" "$VIKAS" run examples/multifile_proj/project.f
step_grep "proof:nova-9of9" "NOVA-PASS 9/9" "$VIKAS" run examples/nova_soc/nova.f --max-ns 8000
step "proof:signoff"  0 "$VIKAS" signoff examples/nova_soc/nova.f --max-ns 8000
step "proof:latch-trap-rc2" 2 "$VIKAS" latch examples/demo_latch_trap.sv
step "proof:latch-uart-rc0" 0 "$VIKAS" latch examples/demo_fsm_uart.sv
step "proof:studio"  0 "$VIKAS" studio examples/nova_soc/nova.f -o /tmp/nova_studio.html --no-open
step "proof:studio-html-nonempty" 0 test -s /tmp/nova_studio.html

echo
echo "══════════ LOCAL CI SIMULATION: $PASS passed, $FAIL failed ══════════"
[ $FAIL -gt 0 ] && { printf 'failed: %s\n' "${FAILED[@]}"; exit 1; }
exit 0
