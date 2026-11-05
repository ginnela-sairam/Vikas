import React, { useMemo, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  Bug,
  CheckCircle2,
  Cpu,
  Flame,
  Gauge,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Zap,
} from 'lucide-react';
import {
  buildSugiyamaRtlSchematic,
  computeSignoffScore,
  computeSwitchingPower,
  elaborateSystemVerilog2023Constructs,
  OpenKdb,
  partitionWorkStealingDag,
  runFormalBmcAndKInduction,
  runMutationTesting,
  runRtlToGateSynthesisAndLec,
  SignalMeta,
  SignalSteps,
} from '../engine/vikasEngine';
import { ProjectFileItem } from './RtlEditorIdeTab';

interface AnalysisHubAndSignoffTabProps {
  signals: SignalMeta[];
  steps: SignalSteps[];
  kdb: OpenKdb;
  files: ProjectFileItem[];
  onJumpToSignalInWave: (sigIndex: number, timeNs?: number) => void;
  onJumpToSourceFile: (filePath: string) => void;
}

export const AnalysisHubAndSignoffTab: React.FC<AnalysisHubAndSignoffTabProps> = ({
  signals,
  steps,
  kdb,
  files,
  onJumpToSignalInWave,
  onJumpToSourceFile,
}) => {
  const [vdd, setVdd] = useState<number>(0.8);
  const [cnodeFf, setCnodeFf] = useState<number>(1.4);
  const [freqMhz, setFreqMhz] = useState<number>(100);
  const [maxMutants, setMaxMutants] = useState<number>(16);
  const [mutantFilter, setMutantFilter] = useState<'ALL' | 'KILLED' | 'SURVIVED'>('ALL');
  const [bmcBoundK, setBmcBoundK] = useState<number>(24);
  const [workerCount, setWorkerCount] = useState<number>(4);

  // 1. Toggle coverage analysis
  const coverageStats = useMemo(() => {
    let toggled = 0;
    const deadSigs: { index: number; name: string; width: number }[] = [];
    signals.forEach((sig, i) => {
      const st = steps[i];
      const isStaticTie = /tie_lo\b/i.test(sig.n);
      if (st && st.n > 0 && !isStaticTie) {
        toggled++;
      } else {
        deadSigs.push({ index: i, name: sig.n, width: sig.w });
      }
    });
    const pct = signals.length > 0 ? Number(((toggled / signals.length) * 100).toFixed(1)) : 100;
    return { toggled, total: signals.length, pct, deadSigs };
  }, [signals, steps]);

  // 2. Switching power report (Breakthrough 15)
  const powerReport = useMemo(
    () => computeSwitchingPower(signals, steps, vdd, cnodeFf, freqMhz),
    [signals, steps, vdd, cnodeFf, freqMhz]
  );

  // 3. RTL Mutation Testing report (Breakthrough 14)
  const mutationReport = useMemo(
    () => runMutationTesting(files, kdb.top || 'nova_soc_tb', maxMutants, 75.0),
    [files, kdb.top, maxMutants]
  );

  // 4. 7-Pillar One-Command Tapeout Signoff report (Breakthrough 16)
  const signoffReport = useMemo(
    () => computeSignoffScore(kdb, signals, steps, mutationReport),
    [kdb, signals, steps, mutationReport]
  );

  // 5. X-Propagation & Sub-Cycle Glitch scan
  const xAndGlitchStats = useMemo(() => {
    const xSigs: { index: number; name: string; firstXTime: number }[] = [];
    const glitchSigs: { index: number; name: string; count: number; times: number[] }[] = [];

    signals.forEach((sig, i) => {
      const st = steps[i];
      if (!st) return;
      for (let k = 0; k < st.n; k++) {
        const valStr = String(st.v[k]);
        if (valStr.includes('x') || valStr.includes('X')) {
          xSigs.push({ index: i, name: sig.n, firstXTime: st.t[k] });
          break;
        }
      }
      if (st.glitch && st.glitch.length > 0) {
        glitchSigs.push({
          index: i,
          name: sig.n,
          count: st.glitch.length,
          times: st.glitch.slice(0, 6),
        });
      }
    });

    return { xSigs, glitchSigs };
  }, [signals, steps]);

  const filteredMutants = useMemo(() => {
    if (mutantFilter === 'ALL') return mutationReport.mutants;
    return mutationReport.mutants.filter((m) => m.status === mutantFilter);
  }, [mutationReport.mutants, mutantFilter]);

  // 6. Iteration 3: VIKAS-CHITRA Sugiyama Layered RTL Schematic (AP-07)
  const sugiyamaGraph = useMemo(
    () => buildSugiyamaRtlSchematic(kdb, signals, steps, 150),
    [kdb, signals, steps]
  );

  // 7. Iteration 3: Formal Bit-Vector CDCL/SMT BMC + k-Induction Prover (NB-08..12 / AP-05)
  const formalReport = useMemo(
    () => runFormalBmcAndKInduction(kdb, files, bmcBoundK),
    [kdb, files, bmcBoundK]
  );

  // 8. Iteration 3: Built-In RTL-to-Gate Synthesis & Miter LEC (DB-01 / DB-03)
  const lecReport = useMemo(() => runRtlToGateSynthesisAndLec(kdb), [kdb]);

  // 9. Iteration 3: IEEE 1800-2023 Construct Elaborator & Work-Stealing DAG Partitioner (AP-03 / NB-02)
  const sv2023Constructs = useMemo(() => elaborateSystemVerilog2023Constructs(files), [files]);
  const dagPartition = useMemo(
    () => partitionWorkStealingDag(kdb, workerCount),
    [kdb, workerCount]
  );

  return (
    <div className="h-full overflow-y-auto bg-[#0b0e14] p-4 space-y-4 text-slate-200">
      {/* Top 5 KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-3">
        {/* Signoff Score */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 flex flex-col justify-between">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Tapeout Signoff
            </span>
            <Gauge className="w-4 h-4 text-cyan-400" />
          </div>
          <div className="my-2 flex items-baseline gap-2">
            <span
              className={`text-3xl font-extrabold font-mono ${
                signoffReport.verdict === 'SIGNOFF READY'
                  ? 'text-emerald-400'
                  : signoffReport.verdict === 'AT RISK'
                  ? 'text-amber-400'
                  : 'text-rose-400'
              }`}
            >
              {signoffReport.total_score}/100
            </span>
            <span
              className={`text-[10px] font-bold px-2 py-0.5 rounded border ${
                signoffReport.verdict === 'SIGNOFF READY'
                  ? 'bg-emerald-950/60 border-emerald-500/40 text-emerald-300'
                  : signoffReport.verdict === 'AT RISK'
                  ? 'bg-amber-950/60 border-amber-500/40 text-amber-300'
                  : 'bg-rose-950/60 border-rose-500/40 text-rose-300'
              }`}
            >
              {signoffReport.verdict}
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono">
            vikas signoff → exit code {signoffReport.exit_code}
          </div>
        </div>

        {/* Toggle Coverage */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 flex flex-col justify-between">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Toggle Coverage
            </span>
            <Activity className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="my-2 flex items-baseline gap-2">
            <span className="text-3xl font-extrabold font-mono text-emerald-400">
              {coverageStats.pct}%
            </span>
            <span className="text-xs text-slate-400 font-mono">
              ({coverageStats.toggled}/{coverageStats.total})
            </span>
          </div>
          <div className="w-full h-1.5 bg-slate-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-emerald-400 rounded-full"
              style={{ width: `${Math.min(100, coverageStats.pct)}%` }}
            />
          </div>
        </div>

        {/* Switching Energy & Power */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 flex flex-col justify-between">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Switching Energy
            </span>
            <Zap className="w-4 h-4 text-amber-400" />
          </div>
          <div className="my-2 flex items-baseline gap-2">
            <span className="text-2xl font-extrabold font-mono text-amber-400">
              {powerReport.total_energy_fj.toLocaleString()} fJ
            </span>
            <span className="text-xs text-cyan-300 font-mono">
              {powerReport.avg_power_uw} µW
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono">
            Clock tree share: {powerReport.clock_tree_share_pct}% @ {vdd}V
          </div>
        </div>

        {/* Mutation Kill Rate */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 flex flex-col justify-between">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Mutation Kill Score
            </span>
            <Bug className="w-4 h-4 text-purple-400" />
          </div>
          <div className="my-2 flex items-baseline gap-2">
            <span className="text-3xl font-extrabold font-mono text-purple-400">
              {mutationReport.score_pct}%
            </span>
            <span className="text-xs text-slate-400 font-mono">
              ({mutationReport.killed}/{mutationReport.total_mutants} killed)
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono">
            TB fault sensitivity · {mutationReport.wall_ms} ms
          </div>
        </div>

        {/* 3-Layer Inferred Latch Status */}
        <div
          onClick={() => {
            if (kdb.inferred_latches.length > 0) {
              onJumpToSourceFile(kdb.inferred_latches[0].file);
            }
          }}
          className={`rounded-lg border p-3.5 flex flex-col justify-between ${
            kdb.inferred_latches.length === 0
              ? 'border-slate-800 bg-[#111621]'
              : 'border-rose-500/40 bg-rose-950/20 cursor-pointer hover:border-rose-400 transition-colors'
          }`}
          title={
            kdb.inferred_latches.length > 0
              ? `Click to open ${kdb.inferred_latches[0].file} in RTL Editor`
              : 'Zero inferred latches across workspace'
          }
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              3-Layer Latch Guard
            </span>
            {kdb.inferred_latches.length === 0 ? (
              <ShieldCheck className="w-4 h-4 text-emerald-400" />
            ) : (
              <ShieldAlert className="w-4 h-4 text-rose-400" />
            )}
          </div>
          <div className="my-2 flex items-baseline gap-2">
            <span
              className={`text-2xl font-extrabold font-mono ${
                kdb.inferred_latches.length === 0 ? 'text-emerald-400' : 'text-rose-400'
              }`}
            >
              {kdb.inferred_latches.length === 0
                ? '0 LATCHES'
                : `${kdb.inferred_latches.length} LATCH TRAP(S)`}
            </span>
          </div>
          <div className="text-[11px] text-slate-400 font-mono">
            {kdb.inferred_latches.length === 0
              ? 'L1 AST + L2 Hold verified clean'
              : 'Click to open RTL Editor & apply L3 patch'}
          </div>
        </div>
      </div>

      {/* Row 2: 7-Pillar Signoff Scorecard + Switching Power & Clock-Tree Analyzer */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* 7-Pillar Signoff Card */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <ShieldCheck className="w-4 h-4 text-cyan-400" />
                Breakthrough #16 — One-Command Tapeout Signoff (`vikas signoff`)
              </h3>
              <p className="text-xs text-slate-400">
                7-pillar weighted verification gate across static AST, Tarjan SCC, CDC, coverage, liveness & mutation testing
              </p>
            </div>
            <span className="font-mono text-xs px-2.5 py-1 rounded bg-slate-900 border border-slate-700 text-cyan-300">
              Top: {signoffReport.top}
            </span>
          </div>

          <div className="space-y-2.5">
            {signoffReport.pillars.map((pillar, idx) => {
              const pct = Math.round((pillar.score / pillar.weight) * 100);
              return (
                <div key={idx} className="rounded border border-slate-800/80 bg-[#0d111a] p-2.5">
                  <div className="flex items-center justify-between text-xs mb-1">
                    <span className="font-semibold text-slate-200 flex items-center gap-2">
                      {pillar.status === 'PASS' ? (
                        <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                      ) : (
                        <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                      )}
                      {pillar.name}
                    </span>
                    <span className="font-mono text-xs font-bold text-cyan-300">
                      {pillar.score} / {pillar.weight} pts
                    </span>
                  </div>
                  <div className="w-full h-1.5 bg-slate-800 rounded-full overflow-hidden mb-1">
                    <div
                      className={`h-full rounded-full ${
                        pillar.status === 'PASS'
                          ? 'bg-emerald-400'
                          : pillar.status === 'WARN'
                          ? 'bg-amber-400'
                          : 'bg-rose-500'
                      }`}
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                  <div className="text-[11px] text-slate-400 font-mono">{pillar.detail}</div>
                </div>
              );
            })}
          </div>

          <div className="rounded border border-cyan-500/30 bg-cyan-950/20 p-2.5 text-xs text-cyan-200 space-y-1">
            <div className="font-bold uppercase tracking-wider text-[10px] text-cyan-400">
              Signoff Actionable Verdict
            </div>
            {signoffReport.recommendations.map((rec, i) => (
              <div key={i} className="font-mono text-[11px]">
                • {rec}
              </div>
            ))}
          </div>
        </div>

        {/* Switching Power & Hotspot Estimator (Breakthrough 15) */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3 flex flex-col">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Flame className="w-4 h-4 text-amber-400" />
                Breakthrough #15 — Pre-Synthesis Switching Power (`vikas power`)
              </h3>
              <p className="text-xs text-slate-400">
                Per-signal dynamic energy E = ½·C_eff·V²·N_toggles (fJ) & clock-tree gating diagnostics
              </p>
            </div>
            <div className="flex items-center gap-2">
              <label className="text-[11px] text-slate-400 font-mono flex items-center gap-1">
                VDD:
                <select
                  value={vdd}
                  onChange={(e) => setVdd( parseFloat(e.target.value) )}
                  className="bg-slate-900 border border-slate-700 rounded px-1.5 py-0.5 text-amber-300 font-mono text-xs"
                >
                  <option value={0.12}>0.12V (Sub-Vt)</option>
                  <option value={0.25}>0.25V (Near-Vt)</option>
                  <option value={0.4}>0.40V (ULP)</option>
                  <option value={0.6}>0.60V (Nominal-Low)</option>
                  <option value={0.8}>0.80V (7nm Nom)</option>
                  <option value={1.0}>1.00V (Overdrive)</option>
                </select>
              </label>
              <label className="text-[11px] text-slate-400 font-mono flex items-center gap-1">
                C_node:
                <select
                  value={cnodeFf}
                  onChange={(e) => setCnodeFf(parseFloat(e.target.value))}
                  className="bg-slate-900 border border-slate-700 rounded px-1.5 py-0.5 text-cyan-300 font-mono text-xs"
                >
                  <option value={0.8}>0.8 fF (3nm)</option>
                  <option value={1.4}>1.4 fF (7nm)</option>
                  <option value={2.5}>2.5 fF (16nm)</option>
                </select>
              </label>
              <label className="text-[11px] text-slate-400 font-mono flex items-center gap-1">
                F_clk:
                <select
                  value={freqMhz}
                  onChange={(e) => setFreqMhz(parseFloat(e.target.value))}
                  className="bg-slate-900 border border-slate-700 rounded px-1.5 py-0.5 text-emerald-300 font-mono text-xs"
                >
                  <option value={50}>50 MHz</option>
                  <option value={100}>100 MHz</option>
                  <option value={500}>500 MHz</option>
                  <option value={1000}>1.0 GHz</option>
                </select>
              </label>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-2 text-xs font-mono">
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400 uppercase">Total Dynamic Energy</div>
              <div className="text-base font-bold text-amber-400">
                {powerReport.total_energy_fj.toLocaleString()} fJ
              </div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400 uppercase">Peak 100ns Burst Window</div>
              <div className="text-base font-bold text-rose-400">
                {powerReport.peak_window_ns[0]}–{powerReport.peak_window_ns[1]} ns ({powerReport.peak_window_energy_fj} fJ)
              </div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400 uppercase">Clock Tree Share</div>
              <div className="text-base font-bold text-cyan-400">
                {powerReport.clock_tree_share_pct}%
              </div>
            </div>
          </div>

          <div className="rounded border border-amber-500/30 bg-amber-950/20 px-3 py-1.5 text-[11px] font-mono text-amber-200">
            {powerReport.clock_gating_verdict}
          </div>

          <div className="flex-1 overflow-y-auto max-h-64 border border-slate-800 rounded bg-[#0d111a]">
            <table className="w-full text-left border-collapse text-xs font-mono">
              <thead className="bg-slate-900/90 text-slate-400 sticky top-0 text-[10px] uppercase">
                <tr>
                  <th className="py-1.5 px-2.5">Signal Hotspot</th>
                  <th className="py-1.5 px-2">Width</th>
                  <th className="py-1.5 px-2">Toggles</th>
                  <th className="py-1.5 px-2">Energy (fJ)</th>
                  <th className="py-1.5 px-2">Power (µW)</th>
                  <th className="py-1.5 px-2.5">Share</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {powerReport.hotspots.map((h) => (
                  <tr
                    key={h.index}
                    onClick={() => onJumpToSignalInWave(h.index)}
                    className="hover:bg-slate-800/50 cursor-pointer transition-colors"
                    title="Click to inspect signal in Waveform Studio"
                  >
                    <td className="py-1.5 px-2.5 text-slate-200 truncate max-w-[200px]">
                      {h.isClock && (
                        <span className="inline-block mr-1.5 px-1 py-0.2 text-[9px] rounded bg-cyan-950 text-cyan-300 border border-cyan-500/30">
                          CLK
                        </span>
                      )}
                      {h.name}
                    </td>
                    <td className="py-1.5 px-2 text-slate-400">[{h.width - 1}:0]</td>
                    <td className="py-1.5 px-2 text-cyan-300">{h.toggles}</td>
                    <td className="py-1.5 px-2 text-amber-300 font-bold">{h.energy_fj.toFixed(1)} fJ</td>
                    <td className="py-1.5 px-2 text-emerald-300">{h.power_uw.toFixed(2)} µW</td>
                    <td className="py-1.5 px-2.5 w-28">
                      <div className="flex items-center gap-1.5">
                        <div className="w-14 h-1.5 bg-slate-800 rounded overflow-hidden">
                          <div
                            className="h-full bg-amber-400"
                            style={{ width: `${Math.min(100, h.share_pct * 3)}%` }}
                          />
                        </div>
                        <span className="text-[10px] text-slate-400">{h.share_pct}%</span>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Row 3: Built-In RTL Mutation Testing Engine (Breakthrough 14) */}
      <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div>
            <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
              <Bug className="w-4 h-4 text-purple-400" />
              Breakthrough #14 — Built-In RTL Mutation Testing (`vikas mutant`)
            </h3>
            <p className="text-xs text-slate-400">
              Injects synthetic hardware faults into DUT RTL only (never mutates `*_tb.sv` testbenches) and verifies if golden assertions kill each mutant
            </p>
          </div>
          <div className="flex items-center gap-2">
            <div className="flex rounded border border-slate-700 overflow-hidden text-xs font-mono">
              {(['ALL', 'KILLED', 'SURVIVED'] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setMutantFilter(f)}
                  className={`px-2.5 py-1 ${
                    mutantFilter === f
                      ? 'bg-purple-600 text-white font-bold'
                      : 'bg-slate-900 text-slate-400 hover:text-slate-200'
                  }`}
                >
                  {f}
                </button>
              ))}
            </div>
            <select
              value={maxMutants}
              onChange={(e) => setMaxMutants(parseInt(e.target.value, 10))}
              className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs font-mono text-purple-300"
            >
              <option value={12}>12 Mutants (Fast)</option>
              <option value={16}>16 Mutants (Standard)</option>
              <option value={32}>32 Mutants (Deep Audit)</option>
            </select>
          </div>
        </div>

        <div className="overflow-x-auto border border-slate-800 rounded bg-[#0d111a]">
          <table className="w-full text-left border-collapse text-xs font-mono">
            <thead className="bg-slate-900 text-slate-400 text-[10px] uppercase">
              <tr>
                <th className="py-2 px-3">ID</th>
                <th className="py-2 px-3">DUT File : Line</th>
                <th className="py-2 px-3">Module</th>
                <th className="py-2 px-3">Fault Category</th>
                <th className="py-2 px-3">Mutated RTL Site</th>
                <th className="py-2 px-3">Status</th>
                <th className="py-2 px-3">Testbench Coverage Diagnostic</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60">
              {filteredMutants.map((m) => (
                <tr key={m.id} className="hover:bg-slate-800/40">
                  <td className="py-1.5 px-3 text-slate-400">#{m.id}</td>
                  <td className="py-1.5 px-3 text-cyan-300">
                    <button
                      type="button"
                      onClick={() => onJumpToSourceFile(m.file)}
                      className="hover:underline text-left cursor-pointer text-cyan-300 hover:text-cyan-200"
                      title={`Jump to ${m.file}:${m.line} in RTL Editor`}
                    >
                      {m.file}:{m.line}
                    </button>
                  </td>
                  <td className="py-1.5 px-3 text-slate-300">{m.module}</td>
                  <td className="py-1.5 px-3 text-purple-300">{m.category}</td>
                  <td className="py-1.5 px-3 text-slate-300 max-w-xs truncate" title={`${m.original}  ==>  ${m.mutated}`}>
                    <span className="line-through text-rose-400/80 mr-1.5">{m.original}</span>
                    <span className="text-emerald-300">→ {m.mutated}</span>
                  </td>
                  <td className="py-1.5 px-3">
                    {m.status === 'KILLED' ? (
                      <span className="px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/40 text-emerald-300 text-[10px] font-bold">
                        KILLED @{m.detect_time_ns}ns
                      </span>
                    ) : (
                      <span className="px-2 py-0.5 rounded bg-amber-950/80 border border-amber-500/40 text-amber-300 text-[10px] font-bold">
                        SURVIVED
                      </span>
                    )}
                  </td>
                  <td className="py-1.5 px-3 text-slate-400">{m.tb_gap_hint}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Row 4: Untoggled / Dead Signals, X-Propagation & Sub-Cycle Glitch Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Untoggled Signals */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 space-y-2">
          <div className="text-xs font-bold text-slate-200 flex items-center justify-between">
            <span>Static / Untoggled Signals ({coverageStats.deadSigs.length})</span>
            <Cpu className="w-3.5 h-3.5 text-slate-400" />
          </div>
          {coverageStats.deadSigs.length === 0 ? (
            <div className="text-xs text-emerald-400 font-mono">100% of signals toggled during simulation ✓</div>
          ) : (
            <div className="space-y-1 max-h-36 overflow-y-auto text-xs font-mono">
              {coverageStats.deadSigs.map((d) => (
                <div
                  key={d.index}
                  onClick={() => onJumpToSignalInWave(d.index)}
                  className="flex items-center justify-between px-2 py-1 rounded bg-[#0d111a] hover:bg-slate-800 cursor-pointer"
                >
                  <span className="text-amber-300 truncate">{d.name}</span>
                  <span className="text-slate-500">[{d.width - 1}:0]</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* X-Propagation Scan */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 space-y-2">
          <div className="text-xs font-bold text-slate-200 flex items-center justify-between">
            <span>4-State Unknown (X) Scan ({xAndGlitchStats.xSigs.length})</span>
            <Sparkles className="w-3.5 h-3.5 text-rose-400" />
          </div>
          {xAndGlitchStats.xSigs.length === 0 ? (
            <div className="text-xs text-emerald-400 font-mono">
              Zero unknown (&apos;x&apos;) state transitions across all {signals.length} signals ✓
            </div>
          ) : (
            <div className="space-y-1 max-h-36 overflow-y-auto text-xs font-mono">
              {xAndGlitchStats.xSigs.map((x) => (
                <div
                  key={x.index}
                  onClick={() => onJumpToSignalInWave(x.index, x.firstXTime)}
                  className="flex items-center justify-between px-2 py-1 rounded bg-rose-950/30 border border-rose-500/30 hover:bg-rose-900/40 cursor-pointer"
                >
                  <span className="text-rose-300 truncate">{x.name}</span>
                  <span className="text-rose-400">@{x.firstXTime}ns</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Sub-Cycle Glitch Detector */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-3.5 space-y-2">
          <div className="text-xs font-bold text-slate-200 flex items-center justify-between">
            <span>Sub-Cycle Hazard / Glitch Scan ({xAndGlitchStats.glitchSigs.length})</span>
            <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
          </div>
          {xAndGlitchStats.glitchSigs.length === 0 ? (
            <div className="text-xs text-emerald-400 font-mono">
              Zero zero-time delta-cycle glitches detected ✓
            </div>
          ) : (
            <div className="space-y-1 max-h-36 overflow-y-auto text-xs font-mono">
              {xAndGlitchStats.glitchSigs.map((g) => (
                <div
                  key={g.index}
                  onClick={() => onJumpToSignalInWave(g.index, g.times[0])}
                  className="flex items-center justify-between px-2 py-1 rounded bg-amber-950/30 border border-amber-500/30 hover:bg-amber-900/40 cursor-pointer"
                >
                  <span className="text-amber-300 truncate">{g.name}</span>
                  <span className="text-amber-400">{g.count} glitches</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* ===================================================================== */}
      {/* ITERATION 3 SUITE: SUGIYAMA RTL SCHEMATIC, FORMAL BMC & RTL-TO-GATE LEC */}
      {/* ===================================================================== */}
      <div className="rounded-xl border border-cyan-500/40 bg-[#111621] p-4 space-y-4">
        <div className="flex items-center justify-between flex-wrap gap-2 border-b border-slate-800 pb-2.5">
          <div>
            <h2 className="text-sm font-bold font-mono text-cyan-300 flex items-center gap-2">
              <Cpu className="w-4 h-4 text-cyan-400" />
              <span>
                VIKAS-CHITRA Sugiyama Layered RTL Schematic (AP-07) · Barycenter Crossing Minimization
              </span>
            </h2>
            <p className="text-[11px] font-mono text-slate-400">
              Top: {sugiyamaGraph.top} · {sugiyamaGraph.nodes.length} Elaborated Nodes ·{' '}
              {sugiyamaGraph.edges.length} Orthogonal Nets · Critical Path FO4 ={' '}
              {sugiyamaGraph.criticalPathFo4}
            </p>
          </div>
          <div className="flex items-center gap-2 font-mono text-[11px]">
            <span className="px-2 py-0.5 rounded bg-sky-950/80 border border-sky-500/40 text-sky-300">
              Layer 0: Primary Inputs
            </span>
            <span className="px-2 py-0.5 rounded bg-purple-950/80 border border-purple-500/40 text-purple-300">
              Layer 1: Sequential FF/FSM
            </span>
            <span className="px-2 py-0.5 rounded bg-amber-950/80 border border-amber-500/40 text-amber-300">
              Layer 2: Comb ALU/Mux Cones
            </span>
            <span className="px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/40 text-emerald-300">
              Layer 3: Primary Outputs
            </span>
          </div>
        </div>

        <div className="bg-[#090d16] rounded-lg border border-slate-800 p-2 overflow-x-auto">
          <svg viewBox="0 0 840 420" className="w-full h-80 block font-mono">
            {/* Layer Column Headers */}
            <text x="70" y="24" fill="#38bdf8" fontSize="10" fontWeight="bold">
              L0: INPUT PORTS
            </text>
            <text x="260" y="24" fill="#a78bfa" fontSize="10" fontWeight="bold">
              L1: SEQUENTIAL FFs
            </text>
            <text x="470" y="24" fill="#fbbf24" fontSize="10" fontWeight="bold">
              L2: COMBINATIONAL CONES
            </text>
            <text x="680" y="24" fill="#34d399" fontSize="10" fontWeight="bold">
              L3: OUTPUT PORTS
            </text>

            {/* Orthogonal Manhattan Edges */}
            {sugiyamaGraph.edges.map((e, idx) => {
              const midX = (e.x1 + e.x2) / 2;
              const d = `M ${e.x1} ${e.y1} L ${midX} ${e.y1} L ${midX} ${e.y2} L ${e.x2} ${e.y2}`;
              return (
                <path
                  key={idx}
                  d={d}
                  fill="none"
                  stroke={e.isCritical ? '#fb7185' : 'rgba(56,189,248,0.45)'}
                  strokeWidth={e.isCritical ? '1.8' : '1.2'}
                />
              );
            })}

            {/* Sugiyama Nodes */}
            {sugiyamaGraph.nodes.map((node) => {
              const strokeCol =
                node.kind === 'INPUT'
                  ? '#38bdf8'
                  : node.kind === 'REG_FF'
                  ? '#a78bfa'
                  : node.kind === 'OUTPUT'
                  ? '#34d399'
                  : '#fbbf24';
              return (
                <g
                  key={node.id}
                  onClick={() => {
                    const si = signals.findIndex(
                      (s) => s.n === node.id || s.n.endsWith('.' + node.id)
                    );
                    if (si >= 0) onJumpToSignalInWave(si, 150);
                  }}
                  className="cursor-pointer"
                >
                  <rect
                    x={node.x}
                    y={node.y}
                    width="115"
                    height="36"
                    rx="5"
                    fill="#111827"
                    stroke={strokeCol}
                    strokeWidth="1.4"
                  />
                  <text x={node.x + 6} y={node.y + 14} fill="#f8fafc" fontSize="9.5" fontWeight="bold">
                    {node.label.slice(0, 14)}
                  </text>
                  <text x={node.x + 6} y={node.y + 28} fill="#34d399" fontSize="9">
                    {node.valAtCursor} ({node.width}b)
                  </text>
                </g>
              );
            })}
          </svg>
        </div>
      </div>

      {/* Formal BMC + k-Induction Prover (NB-08..12) AND RTL-to-Gate Synthesis & Miter LEC (DB-03) */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-4 font-mono">
        {/* Formal Bit-Vector CDCL/SMT BMC & k-Induction */}
        <div className="rounded-xl border border-purple-500/40 bg-[#111621] p-4 space-y-3">
          <div className="flex items-center justify-between flex-wrap gap-2">
            <div>
              <h3 className="text-xs font-bold uppercase tracking-wider text-purple-300">
                NB-08..12 Formal Bit-Vector CDCL/SMT BMC &amp; k-Induction Prover
              </h3>
              <p className="text-[11px] text-slate-400">
                Proven: {formalReport.provenCount} · Counterexamples: {formalReport.cexCount} ·
                Depth k = {formalReport.boundDepthK} cycles
              </p>
            </div>
            <div className="flex items-center gap-2 text-xs">
              <span className="text-slate-400">Depth k:</span>
              <input
                type="number"
                aria-label="Formal BMC Bound Depth k"
                min={4}
                max={64}
                value={bmcBoundK}
                onChange={(e) => setBmcBoundK(Number(e.target.value) || 24)}
                className="w-16 bg-[#0b0e14] border border-slate-700 rounded px-2 py-0.5 text-purple-300"
              />
            </div>
          </div>

          <div className="space-y-2 text-xs">
            {formalReport.properties.map((p) => (
              <div
                key={p.id}
                className="p-2.5 rounded bg-[#0b0e14] border border-slate-800 space-y-1.5"
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="font-bold text-cyan-300">
                    {p.id} [{p.engine}]
                  </span>
                  <span
                    className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                      p.status === 'PROVEN_UNBOUNDED'
                        ? 'bg-emerald-950/80 text-emerald-300 border border-emerald-500/40'
                        : 'bg-rose-950/80 text-rose-300 border border-rose-500/40'
                    }`}
                  >
                    {p.status} ({p.solveTimeMs} ms)
                  </span>
                </div>
                <div className="text-[11px] text-slate-300 break-all">{p.property}</div>
                {p.cexTestbenchSv && (
                  <pre className="mt-1 p-2 rounded bg-rose-950/20 border border-rose-500/30 text-[10px] text-rose-200 overflow-x-auto">
                    {p.cexTestbenchSv}
                  </pre>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Built-In RTL-to-Gate Synthesis & Miter LEC (DB-03) + IEEE 1800-2023 & DAG Partitioner */}
        <div className="rounded-xl border border-emerald-500/40 bg-[#111621] p-4 space-y-3">
          <div className="flex items-center justify-between flex-wrap gap-2">
            <div>
              <h3 className="text-xs font-bold uppercase tracking-wider text-emerald-300">
                DB-03 Built-In RTL-to-Gate Technology Mapper &amp; Miter XOR LEC
              </h3>
              <p className="text-[11px] text-slate-400">
                Miter Status: <span className="text-emerald-300 font-bold">{lecReport.miterStatus}</span> ·{' '}
                {lecReport.totalGates} Gates ({lecReport.totalAreaUm2} μm²) · Tcrit = {lecReport.criticalPathPs} ps
              </p>
            </div>
            <span className="px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/40 text-[10px] text-emerald-300 font-bold">
              {lecReport.conesVerified} Cones Verified
            </span>
          </div>

          <div className="grid grid-cols-3 gap-2 text-xs">
            {lecReport.cellBreakdown.map((c) => (
              <div
                key={c.cellType}
                className="p-2 rounded bg-[#0b0e14] border border-slate-800 flex items-center justify-between"
              >
                <span className="text-cyan-300 font-bold">{c.cellType}</span>
                <span className="text-slate-300">
                  {c.count} ({c.areaUm2}μm²)
                </span>
              </div>
            ))}
          </div>

          <pre className="p-2.5 rounded bg-[#0b0e14] border border-slate-800 text-[10px] text-emerald-300 overflow-x-auto">
            {lecReport.netlistPreview}
          </pre>

          <div className="pt-2 border-t border-slate-800 flex items-center justify-between text-[11px] text-slate-300 flex-wrap gap-2">
            <div>
              <span className="text-amber-300 font-bold">AP-03 SV-2023 Elaborator:</span>{' '}
              {sv2023Constructs.map((c) => `${c.name} (${c.totalBits}b)`).join(' · ')}
            </div>
            <div className="flex items-center gap-1.5">
              <span className="text-cyan-300 font-bold">NB-02 Work-Stealing DAG:</span>
              <select
                aria-label="Work-Stealing DAG Worker Threads"
                data-testid="dag-worker-threads-select"
                value={workerCount}
                onChange={(e) => setWorkerCount(Number(e.target.value))}
                className="bg-[#0b0e14] border border-slate-700 rounded px-1.5 py-0.5 text-[10px] text-cyan-300 font-bold cursor-pointer"
              >
                <option value={4}>4 threads</option>
                <option value={8}>8 threads</option>
                <option value={16}>16 threads</option>
                <option value={32}>32 threads</option>
                <option value={64}>64 threads</option>
              </select>
              <span className="text-emerald-300 font-bold">
                → {dagPartition.estimatedSpeedup}x speedup
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
