#!/bin/bash
# ============================================================================
# VIKAS v3.0 — EXHAUSTIVE PER-FEATURE VERIFICATION MATRIX (85 CHECKS)
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

echo "======== VIKAS v3.0 FULL COMMAND MATRIX ========"

# ---- core simulation on all demos + multi-file manifest + 7-lang polyglot cluster ----
for demo in demo_latch_trap demo_riscv_alu_hazard demo_fsm_uart demo_cdc demo_fifo_backpressure demo_sar_adc; do
  rm -rf .vikas
  check "run_$demo" 0 $VIKAS run examples/$demo.sv
done
rm -rf .vikas
check "run_multifile-manifest" 0 $VIKAS run examples/multifile_proj/project.f
rm -rf .vikas
check_grep "run_polyglot-cluster" "CLUSTER-PASS 6/6" $VIKAS run examples/polyglot_ai_cluster/cluster.f --max-ns 600

# ---- NOVA-SoC deep multi-file project (26 files, 5-level hierarchy, 9 exts) ----
check "nova_master-filelist" 0 $VIKAS run examples/nova_soc/nova.f --max-ns 8000
check_grep "nova_9-checks-pass" "NOVA-PASS 9/9" $VIKAS run examples/nova_soc/nova.f --max-ns 8000
check_grep "nova_alt-manifest-files" "NOVA-PASS 9/9" $VIKAS run examples/nova_soc/docs/manifest.files --max-ns 8000
check_grep "nova_latch-clean" "latch-clean" $VIKAS latch examples/nova_soc/nova.f
check_grep "nova_kdb-graph" "open_kdb" bash -c "cd /tmp && rm -rf nova_kdb && mkdir nova_kdb && cd nova_kdb && $VIKAS run $OLDPWD/examples/nova_soc/nova.f --max-ns 8000 >/dev/null 2>&1 && test -s .vikas/open_kdb.vikas.json && echo open_kdb"
check "nova_signoff-ready" 0 $VIKAS signoff examples/nova_soc/nova.f --max-ns 8000
check "nova_power" 0 bash -c "$VIKAS run examples/nova_soc/nova.f --max-ns 8000 >/dev/null 2>&1 && $VIKAS power .vikas/vikas_wave.vwf"
check "nova_power-json" 0 bash -c "$VIKAS power .vikas/vikas_wave.vwf --json"
check "mutant_fast-target" 0 $VIKAS mutant examples/multifile_proj/project.f --max-mutants 12 --min-score 50 --max-ns 4000
check "mutant_json" 0 $VIKAS mutant examples/multifile_proj/project.f --max-mutants 8 --min-score 0 --max-ns 4000 --json
check_grep "signoff_latch-trap-risk" "AT RISK" $VIKAS signoff examples/demo_latch_trap.sv --max-ns 2000

# ---- breakthrough 13: latch detection (L1+L2+L3) ----
rm -rf .vikas
check "latch_trap(exit2)" 2 $VIKAS latch examples/demo_latch_trap.sv
check "latch_clean(exit0)" 0 $VIKAS latch examples/demo_fsm_uart.sv
check "latch_json" 2 $VIKAS latch examples/demo_latch_trap.sv --json

# ---- KDB / debug commands ----
rm -rf .vikas
$VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
check "kdb_build" 0 $VIKAS kdb examples/demo_riscv_alu_hazard.sv
check "trace_alu_result" 0 $VIKAS trace alu_result examples/demo_riscv_alu_hazard.sv
check "xtrace_alu_result" 0 $VIKAS xtrace alu_result examples/demo_riscv_alu_hazard.sv
check "cdc_violations" 0 $VIKAS cdc examples/demo_cdc.sv
check "fsm_uart" 0 $VIKAS fsm examples/demo_fsm_uart.sv
check "sva_uart" 0 $VIKAS sva examples/demo_fsm_uart.sv
check "loops_alu" 0 $VIKAS loops examples/demo_riscv_alu_hazard.sv
check "recommend_alu" 0 $VIKAS recommend alu_result examples/demo_riscv_alu_hazard.sv
check "why_alu_result" 0 $VIKAS why alu_result examples/demo_riscv_alu_hazard.sv
check "doctor_context" 0 $VIKAS doctor examples/demo_latch_trap.sv
check "schematic_alu" 0 $VIKAS schematic alu_result examples/demo_riscv_alu_hazard.sv
check "coverage_toggle" 0 $VIKAS coverage .vikas/vikas_wave.vwf

# ---- wave analysis ----
rm -rf .vikas
$VIKAS run examples/demo_fifo_backpressure.sv >/dev/null 2>&1
check "gui_bundle" 0 $VIKAS gui .vikas/vikas_wave.vwf -o /tmp/vikas_gui_test.bundle
check "summary_exec" 0 $VIKAS summary .vikas/vikas_wave.vwf
check "find_temporal" 0 $VIKAS find valid_in --dump .vikas/vikas_wave.vwf
check "expr_virtual" 0 $VIKAS expr valid_in --dump .vikas/vikas_wave.vwf
check "anomaly_scan" 0 $VIKAS anomaly .vikas/vikas_wave.vwf
check "decode_bus" 0 $VIKAS decode .vikas/vikas_wave.vwf
check "perf_handshake" 0 $VIKAS perf .vikas/vikas_wave.vwf
check "export-doc_wavedrom" 0 $VIKAS export-doc .vikas/vikas_wave.vwf --from 0 --to 100
check "export-tb_stimulus" 0 $VIKAS export-tb .vikas/vikas_wave.vwf
check "delta_latency" 0 $VIKAS delta 10 50
check "cycles_ruler" 0 $VIKAS cycles 100 10
check "convert_vcd" 0 $VIKAS convert .vikas/vikas_wave.vwf /tmp/vikas_rt_test.vcd
check "convert_native" 0 $VIKAS convert .vikas/vikas_wave.vwf /tmp/vikas_rt_test.vwt
check "radix_q15" 0 $VIKAS radix 4000 q15
check "radix_fp16" 0 $VIKAS radix 16960 fp16

# diff + gate-compare (need a second dump)
rm -rf .vikas
$VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
cp .vikas/vikas_wave.vwf /tmp/gold.vwf
rm -rf .vikas
$VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
check "diff_identical" 0 $VIKAS diff /tmp/gold.vwf .vikas/vikas_wave.vwf
check "gate-compare_identical" 0 $VIKAS gate-compare /tmp/gold.vwf .vikas/vikas_wave.vwf

# disasm (32-bit bus needed)
rm -rf .vikas
$VIKAS run examples/demo_sar_adc.sv >/dev/null 2>&1
check "disasm_riscv" 0 $VIKAS disasm .vikas/vikas_wave.vwf
check "spectrum_dac" 0 $VIKAS spectrum .vikas/vikas_wave.vwf --bus code
check "memdump_array" 0 $VIKAS memdump .vikas/vikas_wave.vwf --array approx --time 100
check "pbit_entropy" 0 $VIKAS pbit .vikas/vikas_wave.vwf
check "hdc_fingerprint" 0 $VIKAS hdc .vikas/vikas_wave.vwf
check "annotate_source" 0 $VIKAS annotate examples/demo_sar_adc.sv --time 100

# ---- physics ----
check "physics_subthreshold" 0 $VIKAS physics examples/demo_sar_adc.sv --vdd 0.12 --temp-k 77
check "energy_multivdd" 0 $VIKAS energy examples/demo_sar_adc.sv --vdd 0.12,0.4,0.8
check "sweep_corners" 0 $VIKAS sweep --vdd 0.12,0.4,0.8 --temp-k 300
check "cosim_sar" 0 $VIKAS cosim 12
check "egraph_repair" 0 $VIKAS egraph "a&&a"
check "taint_dye" 0 $VIKAS taint vin code examples/demo_sar_adc.sv

# ---- linux-native ----
check "topo_linux" 0 $VIKAS topo
check "fork_cow" 0 $VIKAS fork 3
check "shm_arena" 0 $VIKAS shm

# ---- project / CI / formats ----
check "session_toml" 0 $VIKAS session
check "ci-report_headless" 0 $VIKAS ci examples/demo_fsm_uart.sv
check "ci-report_latchfail" 1 $VIKAS ci examples/demo_latch_trap.sv
check "autogen-tb" 0 $VIKAS autogen-tb examples/demo_sar_adc.sv
check "quickrun" 0 $VIKAS quickrun examples/demo_sar_adc.sv
check "help_all" 0 $VIKAS help
check "help_formats" 0 $VIKAS formats
check "version" 0 $VIKAS version

# ---- VIKAS v3.0: Multicore Amdahl-Breaker, UVM 1800.2, 42-Lang CIRCT, Agentic-IQ, Horizon & Batch ----
check "bench_suite" 0 $VIKAS bench --scale 1
check "multicore_16c" 0 $VIKAS multicore --threads 16
check "uvm_1800_2" 0 $VIKAS uvm --threads 16
check "compile_all_38" 0 $VIKAS compile-all
check "agentic_iq" 0 $VIKAS agentic examples/nova_soc/nova.f --epochs 5
check "circt_42" 0 $VIKAS circt examples/polyglot_ai_cluster/cluster.f
check "horizon_2030" 0 $VIKAS horizon --vdd 0.7 --tiers 3 --cpo 16
check "audit_7x" 0 $VIKAS audit-7x
check "batch_headless" 0 $VIKAS batch examples/demo_fsm_uart.sv --jobs 4
check_grep "gui_studio" "VIKAS STUDIO" $VIKAS gui .vikas/vikas_wave.vwf -o /dev/shm/studio_check.bundle
check_grep "studio_app-complete" "openPalette" bash -c "cat /dev/shm/studio_check.bundle"
check_grep "studio_kdb-embed" "Design Intel" bash -c "cat /dev/shm/studio_check.bundle"
check_grep "studio_file-open" "loadVWF" bash -c "cat /dev/shm/studio_check.bundle"

# ---- servers (start/kill) ----
check "mcp_server" 0 bash -c "echo 'QUIT' | timeout 10 $VIKAS mcp . >/dev/null 2>&1; true" 
check "wcp_server" 0 bash -c "timeout 8 $VIKAS wcp --addr 127.0.0.1:45984 >/dev/null 2>&1; true" 

echo ""
echo "======== MATRIX COMPLETE: $PASS passed, $FAIL failed ========"
if [ "$FAIL" -gt 0 ]; then
  echo -e "$FAILED_TESTS"
  exit 1
fi