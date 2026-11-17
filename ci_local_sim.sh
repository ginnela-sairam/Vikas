#!/usr/bin/env bash
# Local CI simulation — mirrors .github/workflows/ci.yml exactly (6 jobs).
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
step "exec-bits" 0 bash -c 'for f in *.sh; do [ -f "$f" ] || continue; sed -i "s/\r$//" "$f" 2>/dev/null || true; chmod +x "$f"; bash -n "$f" && test -x "$f" || exit 1; done'

echo "══ JOB 2: kernel (build + unit tests) ══"
step "cargo-build" 0 cargo build --release
step "cargo-test"  0 cargo test --release

echo "══ JOB 3: matrix (85-check sweep) ══"
step "test-matrix" 0 bash test_matrix.sh

echo "══ JOB 4: proof (NOVA-SoC + Polyglot AI Cluster + goldens + signoff) ══"
rm -f /tmp/nova_studio.bundle
step "proof:version"  0 "$VIKAS" version
step "proof:doctor"   0 "$VIKAS" doctor
step_grep "proof:multifile-result200" "result=200" "$VIKAS" run examples/multifile_proj/project.f
step_grep "proof:polyglot-cluster-6of6" "CLUSTER-PASS 6/6" "$VIKAS" run examples/polyglot_ai_cluster/cluster.f --max-ns 600
step_grep "proof:nova-9of9" "NOVA-PASS 9/9" "$VIKAS" run examples/nova_soc/nova.f --max-ns 8000
step "proof:signoff"  0 "$VIKAS" signoff examples/nova_soc/nova.f --max-ns 8000
step "proof:latch-trap-rc2" 2 "$VIKAS" latch examples/demo_latch_trap.sv
step "proof:latch-uart-rc0" 0 "$VIKAS" latch examples/demo_fsm_uart.sv
step "proof:compile-all-38" 0 "$VIKAS" compile-all
step "proof:multicore-16c"  0 "$VIKAS" multicore --threads 16
step "proof:uvm-1800-2"     0 "$VIKAS" uvm --threads 16
step "proof:agentic-iq"     0 "$VIKAS" agentic examples/nova_soc/nova.f --epochs 5
step "proof:circt-42"       0 "$VIKAS" circt examples/polyglot_ai_cluster/cluster.f
step "proof:horizon-2030"   0 "$VIKAS" horizon --vdd 0.7 --tiers 3 --cpo 16
step "proof:audit-7x"       0 "$VIKAS" audit-7x
step "proof:studio"  0 "$VIKAS" studio examples/nova_soc/nova.f -o /tmp/nova_studio.bundle --no-open
step "proof:studio-bundle-nonempty" 0 test -s /tmp/nova_studio.bundle

echo "══ JOB 5: airgap (offline build — fresh cargo home, zero network) ══"
export CARGO_HOME="$(mktemp -d /tmp/airgap_home.XXXXXX)"
step "airgap:offline-build" 0 cargo build --release --locked --offline
step "airgap:version"   0 "$VIKAS" version
step "airgap:doctor"    0 "$VIKAS" doctor
step_grep "airgap:multifile" "result=200" "$VIKAS" run examples/multifile_proj/project.f
step_grep "airgap:nova" "NOVA-PASS 9/9" "$VIKAS" run examples/nova_soc/nova.f --max-ns 8000
unset CARGO_HOME

echo "══ JOB 6: msrv (Rust 1.70 — oldest supported toolchain) ══"
if rustup toolchain list | grep -q "1.70.0"; then
  export CARGO_HOME="$(mktemp -d /tmp/msrv_home.XXXXXX)"
  step "msrv:build-1.70-offline" 0 cargo +1.70.0 build --release --locked --offline
  step "msrv:version" 0 "$VIKAS" version
  step "msrv:doctor"  0 "$VIKAS" doctor
  step_grep "msrv:multifile" "result=200" "$VIKAS" run examples/multifile_proj/project.f
  unset CARGO_HOME
else
  echo "SKIP  msrv (rustup toolchain 1.70.0 not installed locally)"
fi

echo
echo "══════════ LOCAL CI SIMULATION: $PASS passed, $FAIL failed ══════════"
[ $FAIL -gt 0 ] && { printf 'failed: %s\n' "${FAILED[@]}"; exit 1; }
exit 0
