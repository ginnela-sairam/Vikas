import React, { useMemo, useState } from 'react';
import {
  Cpu,
  GitBranch,
  HelpCircle,
  Layers,
  Microscope,
  Network,
  Radio,
  Search,
  Sparkles,
  ThermometerSnowflake,
  Wand2,
} from 'lucide-react';
import {
  egraphOptimize,
  OpenKdb,
  propagateTaintDye,
  runSarAdcRk4Cosim,
  SignalMeta,
  SignalSteps,
  solveCounterfactual,
  sweepSubthresholdCorners,
} from '../engine/vikasEngine';

interface DesignIntelAndPhysicsTabProps {
  kdb: OpenKdb;
  signals: SignalMeta[];
  steps: SignalSteps[];
  cursorTimeNs: number;
  onJumpToSourceFile: (filePath: string) => void;
}

export const DesignIntelAndPhysicsTab: React.FC<DesignIntelAndPhysicsTabProps> = ({
  kdb,
  signals,
  steps,
  cursorTimeNs,
  onJumpToSourceFile,
}) => {
  const [driverSearch, setDriverSearch] = useState('');
  const [selectedSig, setSelectedSig] = useState<string>(() => {
    const first = kdb.drivers.find((d) => d.signal.includes('alu') || d.signal.includes('pc') || d.signal.includes('result'));
    return first ? first.signal : kdb.drivers[0]?.signal || 'result';
  });
  const [whyTimeNs, setWhyTimeNs] = useState<number>(cursorTimeNs || 120);
  const [whyTargetVal, setWhyTargetVal] = useState<string>('0x1');

  const [taintSrc, setTaintSrc] = useState<string>(() => kdb.signals[0] || 'rst_n');
  const [taintDst, setTaintDst] = useState<string>(() => kdb.signals[Math.min(8, kdb.signals.length - 1)] || 'result');

  const [tempK, setTempK] = useState<number>(300);
  const [adcBits, setAdcBits] = useState<number>(12);
  const [adcVdd, setAdcVdd] = useState<number>(0.8);

  const [egraphInput, setEgraphInput] = useState<string>(
    '((alu_a & alu_b) | (alu_a & !alu_b)) + 0 + (op_sel ^ op_sel)'
  );

  // Filtered Open-KDB drivers
  const filteredDrivers = useMemo(() => {
    const q = driverSearch.trim().toLowerCase();
    if (!q) return kdb.drivers.slice(0, 120);
    return kdb.drivers.filter(
      (d) =>
        d.signal.toLowerCase().includes(q) ||
        d.rhs_text.toLowerCase().includes(q) ||
        d.file.toLowerCase().includes(q) ||
        d.kind.toLowerCase().includes(q)
    );
  }, [kdb.drivers, driverSearch]);

  // Selected driver details for Schematic SVG
  const activeDriver = useMemo(() => {
    return (
      kdb.drivers.find((d) => d.signal === selectedSig) ||
      kdb.drivers[0] || {
        signal: selectedSig,
        rhs_text: 'a + b',
        file: 'nova_alu.sv',
        line: 42,
        kind: 'always_comb',
      }
    );
  }, [kdb.drivers, selectedSig]);

  // Counterfactual SAT/Boolean "Why" Solver
  const counterfactual = useMemo(
    () => solveCounterfactual(kdb, signals, steps, selectedSig, whyTimeNs, whyTargetVal),
    [kdb, signals, steps, selectedSig, whyTimeNs, whyTargetVal]
  );

  // 64-Bit Causality Taint-Dye Pathfinder
  const taintResult = useMemo(
    () => propagateTaintDye(kdb, taintSrc, taintDst),
    [kdb, taintSrc, taintDst]
  );

  // Sub-threshold & Cryo-CMOS Corner Sweep
  const corners = useMemo(
    () => sweepSubthresholdCorners([0.12, 0.25, 0.4, 0.6, 0.8], tempK, 0.28),
    [tempK]
  );

  // RK4 12-Bit SAR ADC Co-Simulation + Hann FFT ENOB
  const sarCosim = useMemo(
    () => runSarAdcRk4Cosim(adcBits, adcVdd, tempK),
    [adcBits, adcVdd, tempK]
  );

  // E-Graph Equality Saturation Optimizer
  const egraphRes = useMemo(() => egraphOptimize(egraphInput), [egraphInput]);

  return (
    <div className="h-full overflow-y-auto bg-[#0b0e14] p-4 space-y-4 text-slate-200">
      {/* Row 1: Open-KDB Driver Table + Interactive Gate/Mux Logic Cone Schematic */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
        {/* Open-KDB Driver Browser (7 cols) */}
        <div className="lg:col-span-7 rounded-lg border border-slate-800 bg-[#111621] p-4 flex flex-col space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Layers className="w-4 h-4 text-cyan-400" />
                Open-KDB Elaborated CSR Driver Graph ({kdb.drivers.length} Driver Sites)
              </h3>
              <p className="text-xs text-slate-400">
                Zero-license JSON/CSR replacement for proprietary Synopsys Verdi `.fsdb`/`.kdb` — click any signal to trace its logic cone
              </p>
            </div>
            <div className="relative">
              <Search className="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-2" />
              <input
                type="text"
                value={driverSearch}
                onChange={(e) => setDriverSearch(e.target.value)}
                placeholder="Filter signal, file, RHS..."
                className="bg-slate-900 border border-slate-700 rounded pl-8 pr-2.5 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyan-500"
              />
            </div>
          </div>

          <div className="overflow-y-auto max-h-72 border border-slate-800 rounded bg-[#0d111a]">
            <table className="w-full text-left border-collapse text-xs font-mono">
              <thead className="bg-slate-900 text-slate-400 sticky top-0 text-[10px] uppercase">
                <tr>
                  <th className="py-1.5 px-2.5">Signal</th>
                  <th className="py-1.5 px-2">Kind</th>
                  <th className="py-1.5 px-2">Source File:Line</th>
                  <th className="py-1.5 px-2">FO4</th>
                  <th className="py-1.5 px-2.5">Elaborated RHS Driver Expression</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {filteredDrivers.map((d, idx) => {
                  const fo4 = kdb.fo4_depth[d.signal] ?? 2;
                  const isSel = d.signal === selectedSig;
                  return (
                    <tr
                      key={`${d.signal}-${idx}`}
                      onClick={() => setSelectedSig(d.signal)}
                      className={`cursor-pointer transition-colors ${
                        isSel ? 'bg-cyan-950/50 text-cyan-200' : 'hover:bg-slate-800/50'
                      }`}
                    >
                      <td className="py-1.5 px-2.5 font-bold text-cyan-300">{d.signal}</td>
                      <td className="py-1.5 px-2">
                        <span
                          className={`px-1.5 py-0.5 rounded text-[10px] ${
                            d.kind === 'always_ff'
                              ? 'bg-purple-950/80 text-purple-300 border border-purple-500/30'
                              : d.kind === 'always_comb'
                              ? 'bg-amber-950/80 text-amber-300 border border-amber-500/30'
                              : 'bg-emerald-950/80 text-emerald-300 border border-emerald-500/30'
                          }`}
                        >
                          {d.kind}
                        </span>
                      </td>
                      <td
                        className="py-1.5 px-2 text-slate-400 hover:text-cyan-300 underline"
                        onClick={(e) => {
                          e.stopPropagation();
                          onJumpToSourceFile(d.file);
                        }}
                      >
                        {d.file}:{d.line}
                      </td>
                      <td className="py-1.5 px-2 text-amber-300">{fo4} FO4</td>
                      <td className="py-1.5 px-2.5 text-slate-300 truncate max-w-[260px]" title={d.rhs_text}>
                        {d.rhs_text}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>

        {/* Interactive Logic-Cone Gate/Mux Schematic SVG (5 cols) */}
        <div className="lg:col-span-5 rounded-lg border border-slate-800 bg-[#111621] p-4 flex flex-col justify-between space-y-3">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <GitBranch className="w-4 h-4 text-emerald-400" />
                RTL Logic-Cone Schematic (`vikas schematic`)
              </h3>
              <p className="text-xs text-slate-400 font-mono">
                {activeDriver.signal} ← {activeDriver.file}:{activeDriver.line} ({activeDriver.kind})
              </p>
            </div>
            <span className="text-xs font-mono px-2 py-0.5 rounded bg-emerald-950/60 border border-emerald-500/40 text-emerald-300">
              Depth: {kdb.fo4_depth[activeDriver.signal] ?? 2} FO4
            </span>
          </div>

          <div className="flex-1 flex items-center justify-center bg-[#090d14] border border-slate-800 rounded p-2">
            <svg viewBox="0 0 460 210" className="w-full h-48">
              {/* Fan-in input nets */}
              {counterfactual.fan_in_cone.slice(0, 4).map((inp, idx) => {
                const y = 32 + idx * 44;
                return (
                  <g key={inp}>
                    <rect
                      x={12}
                      y={y - 12}
                      width={118}
                      height={24}
                      rx={4}
                      fill="#131c2e"
                      stroke="#38bdf8"
                      strokeWidth={1.2}
                    />
                    <text x={20} y={y + 4} fill="#bae6fd" fontSize="10" fontFamily="monospace">
                      {inp.slice(0, 14)}
                    </text>
                    <path
                      d={`M 130 ${y} C 165 ${y}, 175 105, 210 ${85 + idx * 12}`}
                      fill="none"
                      stroke="#38bdf8"
                      strokeWidth={1.5}
                    />
                  </g>
                );
              })}

              {/* Central Combinational / Sequential Cell */}
              <rect
                x={210}
                y={55}
                width={115}
                height={100}
                rx={8}
                fill={activeDriver.kind === 'always_ff' ? '#2e1065' : '#064e3b'}
                stroke={activeDriver.kind === 'always_ff' ? '#a855f7' : '#10b981'}
                strokeWidth={2}
              />
              {activeDriver.kind === 'always_ff' && (
                <polygon points="210,135 222,142 210,149" fill="none" stroke="#c084fc" strokeWidth={1.5} />
              )}
              <text x={267} y={84} textAnchor="middle" fill="#f8fafc" fontSize="11" fontWeight="bold" fontFamily="monospace">
                {activeDriver.kind === 'always_ff' ? 'DFF / REG' : 'COMB CONE'}
              </text>
              <text x={267} y={104} textAnchor="middle" fill="#cbd5e1" fontSize="9.5" fontFamily="monospace">
                {activeDriver.rhs_text.slice(0, 16)}
              </text>
              <text x={267} y={124} textAnchor="middle" fill="#94a3b8" fontSize="9" fontFamily="monospace">
                {activeDriver.file}:{activeDriver.line}
              </text>

              {/* Output wire + target port */}
              <line x1={325} y1={105} x2={362} y2={105} stroke="#34d399" strokeWidth={2} />
              <rect
                x={362}
                y={88}
                width={88}
                height={34}
                rx={5}
                fill="#0f172a"
                stroke="#34d399"
                strokeWidth={1.8}
              />
              <text x={406} y={109} textAnchor="middle" fill="#6ee7b7" fontSize="10.5" fontWeight="bold" fontFamily="monospace">
                {activeDriver.signal.slice(0, 11)}
              </text>
            </svg>
          </div>

          <div className="text-[11px] font-mono text-slate-400 flex items-center justify-between">
            <span>Tarjan SCC Combinational Loops: {kdb.scc_loops.length}</span>
            <span>CDC Paths Audited: {kdb.cdc_paths.length}</span>
          </div>
        </div>
      </div>

      {/* Row 2: Counterfactual SAT/Boolean "Why" Solver + 64-Bit Causality Taint-Dye Pathfinder */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Counterfactual SAT/Boolean Why Solver */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <HelpCircle className="w-4 h-4 text-rose-400" />
                Breakthrough #2 — Counterfactual &ldquo;Why&rdquo; Solver (`vikas why`)
              </h3>
              <p className="text-xs text-slate-400">
                Minimal Hamming-distance boolean cone flip required to force `{selectedSig}` to target value
              </p>
            </div>
            <div className="flex items-center gap-2 font-mono text-xs">
              <label className="text-slate-400">
                @t(ns):
                <input
                  type="number"
                  value={whyTimeNs}
                  onChange={(e) => setWhyTimeNs(Math.max(0, parseInt(e.target.value || '0', 10)))}
                  className="ml-1 w-20 bg-slate-900 border border-slate-700 rounded px-1.5 py-0.5 text-cyan-300"
                />
              </label>
              <label className="text-slate-400">
                Want:
                <input
                  type="text"
                  value={whyTargetVal}
                  onChange={(e) => setWhyTargetVal(e.target.value)}
                  className="ml-1 w-16 bg-slate-900 border border-slate-700 rounded px-1.5 py-0.5 text-emerald-300"
                />
              </label>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-2 text-xs font-mono">
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">Actual @{counterfactual.time_ns}ns</div>
              <div className="text-sm font-bold text-rose-400">{counterfactual.current_val}</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">Counterfactual Goal</div>
              <div className="text-sm font-bold text-emerald-400">{counterfactual.target_val}</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">Min Hamming Distance</div>
              <div className="text-sm font-bold text-cyan-300">ΔH = {counterfactual.minimal_hamming_distance} bit(s)</div>
            </div>
          </div>

          <div className="space-y-1.5">
            {counterfactual.blocking_inputs.map((b, i) => (
              <div
                key={i}
                className="flex items-center justify-between rounded bg-[#0d111a] border border-slate-800/80 px-2.5 py-1.5 text-xs font-mono"
              >
                <div>
                  <span className="text-cyan-300 font-bold">{b.signal}</span>
                  <span className="text-slate-500 ml-2">{b.reason}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-rose-400">cur={b.current}</span>
                  <span className="text-slate-500">→</span>
                  <span className="text-emerald-400 font-bold">need={b.required}</span>
                </div>
              </div>
            ))}
          </div>

          <pre className="rounded bg-[#080b10] border border-slate-800 p-2.5 text-[11px] font-mono text-cyan-300 overflow-x-auto">
            {counterfactual.smt_clause}
            {'\n'}
            {counterfactual.sv_force_patch}
          </pre>
        </div>

        {/* 64-Bit Causality Taint-Dye Pathfinder */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Network className="w-4 h-4 text-teal-400" />
                Breakthrough #3 — 64-Bit Causality Taint-Dye (`vikas taint`)
              </h3>
              <p className="text-xs text-slate-400">
                Tracks architectural provenance & information-flow leakage across register/bus stages
              </p>
            </div>
            <div className="flex items-center gap-1.5 text-xs font-mono">
              <select
                value={taintSrc}
                onChange={(e) => setTaintSrc(e.target.value)}
                className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-teal-300 max-w-[130px]"
              >
                {kdb.signals.slice(0, 60).map((s) => (
                  <option key={s} value={s}>
                    src: {s}
                  </option>
                ))}
              </select>
              <span className="text-slate-500">→</span>
              <select
                value={taintDst}
                onChange={(e) => setTaintDst(e.target.value)}
                className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-amber-300 max-w-[130px]"
              >
                {kdb.signals.slice(0, 60).map((s) => (
                  <option key={s} value={s}>
                    dst: {s}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-2 text-xs font-mono">
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">64-Bit Dye Tag</div>
              <div className="text-xs font-bold text-teal-300 truncate">{taintResult.dye_tag_hex}</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">Propagation Hops</div>
              <div className="text-sm font-bold text-cyan-300">{taintResult.hops} stage(s)</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">Shannon Mutual Info</div>
              <div className="text-sm font-bold text-amber-300">{taintResult.mutual_info_bits} bits</div>
            </div>
          </div>

          <div className="rounded bg-[#0d111a] border border-slate-800 p-3">
            <div className="text-[10px] font-mono uppercase text-slate-400 mb-2">
              Causality Taint Flow Path
            </div>
            <div className="flex flex-wrap items-center gap-1.5 font-mono text-xs">
              {taintResult.path.map((node, i) => (
                <React.Fragment key={i}>
                  <span className="px-2 py-1 rounded bg-teal-950/60 border border-teal-500/40 text-teal-200 font-bold">
                    {node}
                  </span>
                  {i < taintResult.path.length - 1 && <span className="text-slate-500">⟹</span>}
                </React.Fragment>
              ))}
            </div>
          </div>

          <div className="rounded border border-teal-500/30 bg-teal-950/20 px-3 py-2 text-xs font-mono text-teal-200">
            {taintResult.security_verdict}
          </div>
        </div>
      </div>

      {/* Row 3: Pre-Synthesis Silicon Physics (Sub-Threshold 0.12V & Cryo 77K) + RK4 SAR ADC Co-Sim */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Sub-Threshold 0.12V & Cryo-CMOS 77K Corner Sweep */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
          <div className="flex items-center justify-between border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <ThermometerSnowflake className="w-4 h-4 text-cyan-400" />
                Breakthrough #5 — Sub-Threshold (0.12V) & Cryo-CMOS (77K) Physics (`vikas physics`)
              </h3>
              <p className="text-xs text-slate-400">
                C++20 / TypeScript MOSFET sub-threshold current I_sub = I_0·exp((V_gs - V_th + η·V_ds)/(n·V_T))
              </p>
            </div>
            <select
              value={tempK}
              onChange={(e) => setTempK(parseInt(e.target.value, 10))}
              className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs font-mono text-cyan-300"
            >
              <option value={4}>4 K (Deep Cryo Quantum)</option>
              <option value={77}>77 K (Liquid Nitrogen Cryo)</option>
              <option value={300}>300 K (Room Temp 27°C)</option>
              <option value={398}>398 K (Automotive 125°C)</option>
            </select>
          </div>

          <div className="overflow-x-auto border border-slate-800 rounded bg-[#0d111a]">
            <table className="w-full text-left border-collapse text-xs font-mono">
              <thead className="bg-slate-900 text-slate-400 text-[10px] uppercase">
                <tr>
                  <th className="py-1.5 px-2.5">VDD</th>
                  <th className="py-1.5 px-2">V_T (kT/q)</th>
                  <th className="py-1.5 px-2">I_on (nA)</th>
                  <th className="py-1.5 px-2">Delay Mult</th>
                  <th className="py-1.5 px-2">Energy %</th>
                  <th className="py-1.5 px-2.5">Silicon Operating Regime</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {corners.map((c) => (
                  <tr key={c.vdd} className="hover:bg-slate-800/40">
                    <td className="py-1.5 px-2.5 font-bold text-amber-300">{c.vdd.toFixed(2)} V</td>
                    <td className="py-1.5 px-2 text-cyan-300">{c.vt_mv} mV</td>
                    <td className="py-1.5 px-2 text-emerald-300">{c.isub_na.toLocaleString()} nA</td>
                    <td className="py-1.5 px-2 text-rose-300">{c.delay_mult}×</td>
                    <td className="py-1.5 px-2 text-purple-300">{c.energy_scale_pct}%</td>
                    <td className="py-1.5 px-2.5 text-slate-300 text-[11px]">{c.verdict}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* RK4 12-Bit SAR ADC Co-Simulation + Hann FFT Spectrum */}
        <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Radio className="w-4 h-4 text-amber-400" />
                Breakthrough #6 — RK4 SAR ADC Co-Sim & FFT ENOB (`vikas cosim` / `spectrum`)
              </h3>
              <p className="text-xs text-slate-400">
                8-substep 4th-order Runge-Kutta RC DAC settling + kT/C thermal noise + 64-pt Hann DFT
              </p>
            </div>
            <div className="flex items-center gap-2 text-xs font-mono">
              <select
                value={adcBits}
                onChange={(e) => setAdcBits(parseInt(e.target.value, 10))}
                className="bg-slate-900 border border-slate-700 rounded px-2 py-1 text-amber-300"
              >
                <option value={8}>8-Bit SAR</option>
                <option value={10}>10-Bit SAR</option>
                <option value={12}>12-Bit SAR</option>
              </select>
            </div>
          </div>

          <div className="grid grid-cols-5 gap-2 text-xs font-mono">
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">ENOB</div>
              <div className="text-sm font-bold text-emerald-400">{sarCosim.enob} bits</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">SINAD</div>
              <div className="text-sm font-bold text-cyan-300">{sarCosim.sinad_db} dB</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">SFDR</div>
              <div className="text-sm font-bold text-amber-300">{sarCosim.sfdr_db} dBc</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">THD</div>
              <div className="text-sm font-bold text-purple-300">{sarCosim.thd_db} dB</div>
            </div>
            <div className="rounded bg-[#0d111a] border border-slate-800 p-2">
              <div className="text-[10px] text-slate-400">kT/C Noise</div>
              <div className="text-sm font-bold text-rose-300">{sarCosim.thermal_noise_uv} µV</div>
            </div>
          </div>

          {/* Dual SVG: Left = Time-Domain Vin vs Vdac, Right = FFT Spectrum */}
          <div className="grid grid-cols-2 gap-2">
            <div className="bg-[#090d14] border border-slate-800 rounded p-2">
              <div className="text-[10px] font-mono text-slate-400 mb-1">
                RK4 Analog V_in (cyan) vs SAR V_dac (amber)
              </div>
              <svg viewBox="0 0 260 90" className="w-full h-24">
                <polyline
                  fill="none"
                  stroke="#22d3ee"
                  strokeWidth="1.5"
                  points={sarCosim.waveform
                    .map((w, i) => `${(i / 63) * 250 + 5},${85 - (w.vin / sarCosim.vdd) * 75}`)
                    .join(' ')}
                />
                <polyline
                  fill="none"
                  stroke="#fbbf24"
                  strokeWidth="1.2"
                  strokeDasharray="3,2"
                  points={sarCosim.waveform
                    .map((w, i) => `${(i / 63) * 250 + 5},${85 - (w.vdac / sarCosim.vdd) * 75}`)
                    .join(' ')}
                />
              </svg>
            </div>
            <div className="bg-[#090d14] border border-slate-800 rounded p-2">
              <div className="text-[10px] font-mono text-slate-400 mb-1">
                Hann-Windowed FFT Spectrum (dBFS)
              </div>
              <svg viewBox="0 0 260 90" className="w-full h-24">
                {sarCosim.spectrum.slice(1, 32).map((bin, i) => {
                  const x = 8 + i * 7.8;
                  const normH = Math.max(3, Math.min(78, 80 + bin.mag_db * 0.75));
                  return (
                    <rect
                      key={bin.bin}
                      x={x}
                      y={85 - normH}
                      width={5}
                      height={normH}
                      fill={bin.bin === 5 ? '#34d399' : '#6366f1'}
                      rx={1}
                    />
                  );
                })}
              </svg>
            </div>
          </div>
        </div>
      </div>

      {/* Row 4: E-Graph Equality Saturation RTL Repair Engine (Breakthrough 7) */}
      <div className="rounded-lg border border-slate-800 bg-[#111621] p-4 space-y-3">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800 pb-2.5">
          <div>
            <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
              <Wand2 className="w-4 h-4 text-emerald-400" />
              Breakthrough #7 — E-Graph Equality Saturation Optimizer (`vikas egraph`)
            </h3>
            <p className="text-xs text-slate-400">
              Algebraic equivalence saturation rewrites redundant combinational cones & reduces critical-path FO4 delay
            </p>
          </div>
          <div className="flex items-center gap-2 text-xs font-mono">
            <span className="px-2 py-0.5 rounded bg-slate-900 border border-slate-700 text-amber-300">
              FO4 Before: {egraphRes.fo4_before}
            </span>
            <span className="text-slate-500">→</span>
            <span className="px-2 py-0.5 rounded bg-emerald-950/60 border border-emerald-500/40 text-emerald-300 font-bold">
              FO4 After: {egraphRes.fo4_after} (-{egraphRes.gate_reduction_pct}% gates)
            </span>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
          <div className="space-y-2">
            <label className="block text-xs font-mono text-slate-400">
              Input SystemVerilog Combinational Expression:
            </label>
            <input
              type="text"
              value={egraphInput}
              onChange={(e) => setEgraphInput(e.target.value)}
              className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-2 text-xs font-mono text-cyan-200 focus:outline-none focus:border-emerald-500"
            />
            <div className="text-[11px] font-mono text-slate-400 space-y-1">
              <div className="font-bold text-slate-300">Applied Equality Saturation Rewrite Rules:</div>
              {egraphRes.applied_rules.map((r, i) => (
                <div key={i} className="text-emerald-300">
                  ✓ {r}
                </div>
              ))}
            </div>
          </div>

          <pre className="rounded bg-[#080b10] border border-slate-800 p-3 text-xs font-mono text-emerald-300 overflow-x-auto">
            {egraphRes.sv_patch}
          </pre>
        </div>
      </div>
    </div>
  );
};
