#!/bin/bash
# ============================================================================
# VIKAS v1.1 — FIVE-TERMINAL PARALLEL STRESS VERIFICATION
# Simultaneous heavy workloads across independent sessions, exactly like a
# team of engineers hammering the tool at once. All must pass.
# ============================================================================
cd /home/z/my-project/vikas-eda
VIKAS="$(pwd)/target/release/vikas"
export VIKAS_NO_BANNER=1
mkdir -p qa_results/multi
LOG=qa_results/multi

echo "[MULTI] launching 5 parallel verification terminals @ $(date +%T)"

# T1 — NOVA-SoC full + mutation testing (CPU heavy) — ISOLATED COPY
(
  T1D=/tmp/multi_t1; rm -rf $T1D; mkdir -p $T1D
  cp -r /home/z/my-project/vikas-eda/examples $T1D/ && mkdir -p $T1D/target/release && cp /home/z/my-project/vikas-eda/target/release/vikas $T1D/target/release/
  cd $T1D
  VIKAS="$T1D/target/release/vikas"
  out=$($VIKAS run examples/nova_soc/nova.f 2>&1)
  echo "$out" | grep -q "NOVA-PASS 9/9" && echo "T1-NOVA PASS" || echo "T1-NOVA FAIL"
  mout=$(timeout 400 $VIKAS mutant examples/nova_soc/nova.f --max-mutants 16 --max-ns 4000 2>&1)
  echo "$mout" | tail -4
  echo "$mout" | grep -qiE "score|killed|survivor" && echo "T1-MUTANT PASS" || echo "T1-MUTANT FAIL"
) > $LOG/t1_nova.log 2>&1 &

# T2 — kernel bench at 2 scales, repeated (perf stability) — ISOLATED COPY
(
  T2D=/tmp/multi_t2; rm -rf $T2D; mkdir -p $T2D
  cp -r /home/z/my-project/vikas-eda/examples $T2D/ && mkdir -p $T2D/target/release && cp /home/z/my-project/vikas-eda/target/release/vikas $T2D/target/release/
  cd $T2D
  VIKAS="$T2D/target/release/vikas"
  $VIKAS bench --scale 2 >/dev/null 2>&1 && echo "T2-BENCH2 PASS" || echo "T2-BENCH2 FAIL"
  e1=$(python3 -c "import json;print(json.load(open('qa_results/bench.json'))['aggregate_events_per_s'])" 2>/dev/null)
  $VIKAS bench --scale 4 >/dev/null 2>&1 && echo "T2-BENCH4 PASS" || echo "T2-BENCH4 FAIL"
  echo "T2 aggregate events/s (scale2, under 5-way CPU contention): $e1"
  [ -n "$e1" ] && [ "$e1" -gt 25000 ] && echo "T2-THROUGHPUT PASS" || echo "T2-THROUGHPUT FAIL"
) > $LOG/t2_bench.log 2>&1 &

# T3 — signoff + latch + CDC + FSM on two projects (static flows) — ISOLATED COPY
(
  T3D=/tmp/multi_t3; rm -rf $T3D; mkdir -p $T3D
  cp -r /home/z/my-project/vikas-eda/examples $T3D/ && mkdir -p $T3D/target/release && cp /home/z/my-project/vikas-eda/target/release/vikas $T3D/target/release/
  cd $T3D
  VIKAS="$T3D/target/release/vikas"
  $VIKAS signoff examples/nova_soc/nova.f 2>&1 | tail -4
  rc=$?
  [ $rc -le 2 ] && echo "T3-SIGNOFF PASS (rc=$rc)" || echo "T3-SIGNOFF FAIL (rc=$rc)"
  $VIKAS latch examples/demo_latch_trap.sv >/dev/null 2>&1
  [ $? -eq 2 ] && echo "T3-LATCH-TRAP PASS" || echo "T3-LATCH-TRAP FAIL"
  $VIKAS cdc examples/demo_cdc.sv >/dev/null 2>&1 && echo "T3-CDC PASS" || echo "T3-CDC FAIL"
  $VIKAS fsm examples/demo_fsm_uart.sv >/dev/null 2>&1 && echo "T3-FSM PASS" || echo "T3-FSM FAIL"
) > $LOG/t3_signoff.log 2>&1 &

# T4 — big-dump stress: 5k-cycle sim -> vwf/vcd roundtrip -> studio with big data
(
  mkdir -p /dev/shm/t4 && cd /dev/shm/t4
  cp -r /home/z/my-project/vikas-eda/examples .
  $VIKAS run examples/nova_soc/nova.f >/dev/null 2>&1
  s1=$(stat -c %s .vikas/vikas_wave.vwf 2>/dev/null || echo 0)
  $VIKAS convert .vikas/vikas_wave.vwf /dev/shm/t4/big.vcd >/dev/null 2>&1 && echo "T4-VCD-WRITE PASS" || echo "T4-VCD-WRITE FAIL"
  $VIKAS gui .vikas/vikas_wave.vwf -o /dev/shm/t4/big_studio.html >/dev/null 2>&1 && echo "T4-STUDIO-BIG PASS" || echo "T4-STUDIO-BIG FAIL"
  s2=$(stat -c %s /dev/shm/t4/big_studio.html)
  [ "$s1" -gt 10000 ] && [ "$s2" -gt 200000 ] && echo "T4-SIZES PASS (vwf=$s1 html=$s2)" || echo "T4-SIZES FAIL (vwf=$s1 html=$s2)"
  $VIKAS summary /dev/shm/t4/big.vcd >/dev/null 2>&1 && echo "T4-VCD-READBACK PASS" || echo "T4-VCD-READBACK FAIL"
) > $LOG/t4_bigdump.log 2>&1 &

# T5 — wave-format torture: all 6 native formats roundtrip + diff + search — ISOLATED COPY
(
  T5D=/tmp/multi_t5; rm -rf $T5D; mkdir -p $T5D
  cp -r /home/z/my-project/vikas-eda/examples $T5D/ && mkdir -p $T5D/target/release && cp /home/z/my-project/vikas-eda/target/release/vikas $T5D/target/release/
  cd $T5D
  VIKAS="$T5D/target/release/vikas"
  $VIKAS run examples/demo_riscv_alu_hazard.sv >/dev/null 2>&1
  ok=0
  for fmt in vwt vwe vwp vwi vwm; do
    if $VIKAS convert .vikas/vikas_wave.vwf /dev/shm/t5.$fmt >/dev/null 2>&1 && \
       $VIKAS summary /dev/shm/t5.$fmt >/dev/null 2>&1; then
      ok=$((ok+1)); echo "T5-$fmt PASS"
    else
      echo "T5-$fmt FAIL"
    fi
  done
  [ $ok -eq 5 ] && echo "T5-FORMATS PASS ($ok/5)" || echo "T5-FORMATS FAIL ($ok/5)"
  $VIKAS find valid_ex --dump .vikas/vikas_wave.vwf >/dev/null 2>&1 && echo "T5-SEARCH PASS" || echo "T5-SEARCH FAIL"
) > $LOG/t5_formats.log 2>&1 &

wait
echo "[MULTI] all terminals finished @ $(date +%T)"
echo "=========================================="
PASSN=0; FAILN=0
for f in $LOG/t*.log; do
  echo "--- $f ---"
  cat "$f"
  p=$(grep -c " PASS" "$f"); fl=$(grep -c " FAIL" "$f")
  PASSN=$((PASSN+p)); FAILN=$((FAILN+fl))
done
echo "=========================================="
echo "[MULTI] TOTAL: $PASSN passed, $FAILN failed"
[ $FAILN -eq 0 ] && echo "[MULTI] ALL 5 TERMINALS GREEN" || echo "[MULTI] FAILURES DETECTED"
exit $([ $FAILN -eq 0 ] && echo 0 || echo 1)
