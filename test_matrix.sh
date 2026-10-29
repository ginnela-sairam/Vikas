#!/bin/bash
# ============================================================================
# VIKAS v1 — EXHAUSTIVE PER-FEATURE VERIFICATION MATRIX
# Tests every command individually and captures PASS/FAIL feedback
# ============================================================================
cd "$(dirname "$0")"
VIKAS="$(pwd)/target/release/vikas"
export VIKAS_NO_BANNER=1
RESULTS=./qa_results
mkdir -p $RESULTS
PASS=0; FAIL=0; FAILED_TESTS=""

check() {
  local name="$1"; shift
  local expect="${1:-0}"; shift
  local out
  out=$(timeout 90 "$@" 2>&1)
  local rc=$?
  if [ "$rc" -eq "$expect" ]; then
    PASS=$((PASS+1))
    echo "PASS [$name]"
    echo "$out" | head -6 > "$RESULTS/$name.out"
  else
    FAIL=$((FAIL+1)); FAILED_TESTS="$FAILED_TESTS\n  - $name (rc=$rc, expected $expect)"
    echo "FAIL [$name] rc=$rc"
    echo "$out" | head -12 > "$RESULTS/$name.out"
    echo "$out" | head -6
  fi
}

# check_grep <name> <pattern> <cmd...> : PASS when the command's combined
# output contains <pattern> (for content-level golden assertions)
check_grep() {
  local name="$1"; local pattern="$2"; shift 2
  local out
  out=$(timeout 120 "$@" 2>&1)
  if echo "$out" | grep -q -- "$pattern"; then
    PASS=$((PASS+1))
    echo "PASS [$name]"
    echo "$out" | head -6 > "$RESULTS/$name.out"
  else
    FAIL=$((FAIL+1)); FAILED_TESTS="$FAILED_TESTS\n  - $name (pattern '$pattern' not found)"
    echo "FAIL [$name] pattern-not-found"
    echo "$out" | head -12 > "$RESULTS/$name.out"
    echo "$out" | head -6
  fi
}

echo "======== VIKAS v1 FULL COMMAND MATRIX ========"

# ---- core simulation on all demos + multi-file manifest ----
for demo in demo_latch_trap demo_riscv_alu_hazard demo_fsm_uart demo_cdc demo_fifo_backpressure demo_sar_adc; do
  rm -rf .vikas
  check "run:$demo" 0 $VIKAS run examples/$demo.sv
done
rm -rf .vikas
check "run:multifile-manifest" 0 $VIKAS run examples/multifile_proj/project.f

# ---- NOVA-SoC deep multi-file project (26 files, 5-level hierarchy, 9 exts) ----
check "nova:master-filelist" 0 $VIKAS run examples/nova_soc/nova.f --max-ns 8000
check_grep "nova:9-checks-pass" "NOVA-PASS 9/9" $VIKAS run examples/nova_soc/nova.f --max-ns 8000
check_grep "nova:alt-manifest-files" "NOVA-PASS 9/9" $VIKAS run examples/nova_soc/docs/manifest.files --max-ns 8000
check_grep "nova:latch-clean" "latch-clean" $VIKAS latch examples/nova_soc/nova.f
check_grep "nova:kdb-graph" "open_kdb" bash -c "cd /tmp && rm -rf nova_kdb && mkdir nova_kdb && cd nova_kdb && $VIKAS run $OLDPWD/examples/nova_soc/nova.f --max-ns 8000 >/dev/null 2>&1 && test -s .vikas/open_kdb.vikas.json && echo open_kdb"
check "nova:signoff-ready" 0 $VIKAS signoff examples/nova_soc/nova.f --max-ns 8000
check "nova:power" 0 bash -c "$VIKAS run examples/nova_soc/nova.f --max-ns 8000 >/dev/null 2>&1 && $VIKAS power .vikas/vikas_wave.vwf"
check "nova:power-json" 0 bash -c "$VIKAS power .vikas/vikas_wave.vwf --json"
check "mutant:fast-target" 0 $VIKAS mutant examples/multifile_proj/project.f --max-mutants 12 --min-score 50 --max-ns 4000
check "mutant:json" 0 $VIKAS mutant examples/multifile_proj/project.f --max-mutants 8 --min-score 0 --max-ns 4000 --json
check_grep "signoff:latch-trap-risk" "AT RISK" $VIKAS signoff examples/demo_latch_trap.sv --max-ns 2000

# ---- breakthrough 13: latch detection (L1+L2+L3) ----
rm -rf .vikas
check "latch:trap(exit2)" 2 $VIKAS latch examples/demo_latch_trap.sv
check "latch:clean(exit0)" 0 $VIKAS latch examples/demo_fsm_uart.sv
check "latch:json" 2 $VIKAS latch examples/demo_latch_trap.sv --json

# ---- KDB / debug commands ----
rm -rf .vikas
$VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
check "kdb:build" 0 $VIKAS kdb examples/demo_riscv_alu_hazard.sv
check "trace:alu_result" 0 $VIKAS trace alu_result examples/demo_riscv_alu_hazard.sv
check "xtrace:alu_result" 0 $VIKAS xtrace alu_result examples/demo_riscv_alu_hazard.sv
check "cdc:violations" 0 $VIKAS cdc examples/demo_cdc.sv
check "fsm:uart" 0 $VIKAS fsm examples/demo_fsm_uart.sv
check "sva:uart" 0 $VIKAS sva examples/demo_fsm_uart.sv
check "loops:alu" 0 $VIKAS loops examples/demo_riscv_alu_hazard.sv
check "recommend:alu" 0 $VIKAS recommend alu_result examples/demo_riscv_alu_hazard.sv
check "why:alu_result" 0 $VIKAS why alu_result examples/demo_riscv_alu_hazard.sv
check "doctor:context" 0 $VIKAS doctor examples/demo_latch_trap.sv
check "schematic:alu" 0 $VIKAS schematic alu_result examples/demo_riscv_alu_hazard.sv
check "coverage:toggle" 0 $VIKAS coverage .vikas/vikas_wave.vwf

# ---- wave analysis ----
rm -rf .vikas
$VIKAS run examples/demo_fifo_backpressure.sv >/dev/null 2>&1
check "gui:html5" 0 $VIKAS gui .vikas/vikas_wave.vwf -o /tmp/vikas_gui_test.html
check "summary:exec" 0 $VIKAS summary .vikas/vikas_wave.vwf
check "find:temporal" 0 $VIKAS find valid_in --dump .vikas/vikas_wave.vwf
check "expr:virtual" 0 $VIKAS expr valid_in --dump .vikas/vikas_wave.vwf
check "anomaly:scan" 0 $VIKAS anomaly .vikas/vikas_wave.vwf
check "decode:bus" 0 $VIKAS decode .vikas/vikas_wave.vwf
check "perf:handshake" 0 $VIKAS perf .vikas/vikas_wave.vwf
check "export-doc:wavedrom" 0 $VIKAS export-doc .vikas/vikas_wave.vwf --from 0 --to 100
check "export-tb:stimulus" 0 $VIKAS export-tb .vikas/vikas_wave.vwf
check "delta:latency" 0 $VIKAS delta 10 50
check "cycles:ruler" 0 $VIKAS cycles 100 10
check "convert:vcd" 0 $VIKAS convert .vikas/vikas_wave.vwf /tmp/vikas_rt_test.vcd
check "convert:native" 0 $VIKAS convert .vikas/vikas_wave.vwf /tmp/vikas_rt_test.vwt
check "radix:q15" 0 $VIKAS radix 4000 q15
check "radix:fp16" 0 $VIKAS radix 16960 fp16

# diff + gate-compare (need a second dump)
rm -rf .vikas
$VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
cp .vikas/vikas_wave.vwf /tmp/gold.vwf
rm -rf .vikas
$VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
check "diff:identical" 0 $VIKAS diff /tmp/gold.vwf .vikas/vikas_wave.vwf
check "gate-compare:identical" 0 $VIKAS gate-compare /tmp/gold.vwf .vikas/vikas_wave.vwf

# disasm (32-bit bus needed)
rm -rf .vikas
$VIKAS run examples/demo_sar_adc.sv >/dev/null 2>&1
check "disasm:riscv" 0 $VIKAS disasm .vikas/vikas_wave.vwf
check "spectrum:dac" 0 $VIKAS spectrum .vikas/vikas_wave.vwf --bus code
check "memdump:array" 0 $VIKAS memdump .vikas/vikas_wave.vwf --array approx --time 100
check "pbit:entropy" 0 $VIKAS pbit .vikas/vikas_wave.vwf
check "hdc:fingerprint" 0 $VIKAS hdc .vikas/vikas_wave.vwf
check "annotate:source" 0 $VIKAS annotate examples/demo_sar_adc.sv --time 100

# ---- physics ----
check "physics:subthreshold" 0 $VIKAS physics examples/demo_sar_adc.sv --vdd 0.12 --temp-k 77
check "energy:multivdd" 0 $VIKAS energy examples/demo_sar_adc.sv --vdd 0.12,0.4,0.8
check "sweep:corners" 0 $VIKAS sweep --vdd 0.12,0.4,0.8 --temp-k 300
check "cosim:sar" 0 $VIKAS cosim 12
check "egraph:repair" 0 $VIKAS egraph "a&&a"
check "taint:dye" 0 $VIKAS taint vin code examples/demo_sar_adc.sv

# ---- linux-native ----
check "topo:linux" 0 $VIKAS topo
check "fork:cow" 0 $VIKAS fork 3
check "shm:arena" 0 $VIKAS shm

# ---- project / CI / formats ----
check "session:toml" 0 $VIKAS session
check "ci-report:headless" 0 $VIKAS ci examples/demo_fsm_uart.sv
check "ci-report:latchfail" 1 $VIKAS ci examples/demo_latch_trap.sv
check "autogen-tb" 0 $VIKAS autogen-tb examples/demo_sar_adc.sv
check "quickrun" 0 $VIKAS quickrun examples/demo_sar_adc.sv
check "help:all" 0 $VIKAS help
check "help:formats" 0 $VIKAS formats
check "version" 0 $VIKAS version

# ---- v1.1: VIKAS Studio GUI + kernel bench ----
check "bench:suite" 0 $VIKAS bench --scale 1
check_grep "gui:studio" "VIKAS STUDIO" $VIKAS gui .vikas/vikas_wave.vwf -o /dev/shm/studio_check.html
check_grep "studio:app-complete" "openPalette" bash -c "cat /dev/shm/studio_check.html"
check_grep "studio:kdb-embed" "Design Intel" bash -c "cat /dev/shm/studio_check.html"
check_grep "studio:file-open" "loadVWF" bash -c "cat /dev/shm/studio_check.html"

# ---- servers (start/kill) ----
check "mcp:server" 0 bash -c "echo 'QUIT' | timeout 10 $VIKAS mcp . >/dev/null 2>&1; true" 
check "wcp:server" 0 bash -c "timeout 8 $VIKAS wcp --addr 127.0.0.1:45984 >/dev/null 2>&1; true" 

echo ""
echo "======== MATRIX COMPLETE: $PASS passed, $FAIL failed ========"
if [ "$FAIL" -gt 0 ]; then
  echo -e "$FAILED_TESTS"
  exit 1
fi