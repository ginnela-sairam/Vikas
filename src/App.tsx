import React, { useEffect, useMemo, useState } from 'react';
import {
  Activity,
  CheckCircle2,
  Code2,
  Command,
  Cpu,
  Download,
  ExternalLink,
  Gauge,
  HelpCircle,
  Palette,
  ShieldAlert,
  Sparkles,
  Terminal,
  Zap,
} from 'lucide-react';
import {
  buildOpenKdb,
  computeSwitchingPower,
  OpenKdb,
  SignalMeta,
  SignalSteps,
  simulateVerilogSource,
} from './engine/vikasEngine';
import { WaveformStudioTab } from './components/WaveformStudioTab';
import { ProjectFileItem, RtlEditorIdeTab } from './components/RtlEditorIdeTab';
import { AnalysisHubAndSignoffTab } from './components/AnalysisHubAndSignoffTab';
import { DesignIntelAndPhysicsTab } from './components/DesignIntelAndPhysicsTab';
import { TerminalAndGithubSyncTab } from './components/TerminalAndGithubSyncTab';

type TabId = 'waveforms' | 'editor' | 'analysis' | 'intel' | 'terminal';
type ThemeId = 'dark' | 'verdi' | 'light' | 'synth';

export function App() {
  const [activeTab, setActiveTab] = useState<TabId>('waveforms');
  const [theme, setTheme] = useState<ThemeId>('dark');
  const [loading, setLoading] = useState<boolean>(true);
  const [simulating, setSimulating] = useState<boolean>(false);

  // Active simulation session state
  const [activeTarget, setActiveTarget] = useState<string>('examples/nova_soc/nova.f');
  const [sessionTitle, setSessionTitle] = useState<string>(
    'nova_soc_tb — NOVA-PASS 9/9 (26 files · 5 hierarchy levels)'
  );
  const [verdict, setVerdict] = useState<string>('NOVA-PASS 9/9');
  const [verdictKind, setVerdictKind] = useState<string>('ok');
  const [signals, setSignals] = useState<SignalMeta[]>([]);
  const [steps, setSteps] = useState<SignalSteps[]>([]);
  const [endNs, setEndNs] = useState<number>(3000);
  const [nev, setNev] = useState<number>(14579);
  const [kdb, setKdb] = useState<OpenKdb>(() =>
    buildOpenKdb([{ path: 'nova_soc_top.sv', content: 'module nova_soc_top(input logic clk); endmodule' }])
  );

  // Waveform cursors & default radix
  const [curA, setCurA] = useState<number | null>(150);
  const [curB, setCurB] = useState<number | null>(650);
  const [defRadix, setDefRadix] = useState<'hex' | 'bin' | 'dec' | 'signed' | 'oct' | 'ascii'>('hex');

  // Project RTL & manifest files for VIKAS-VED IDE
  const [files, setFiles] = useState<ProjectFileItem[]>([]);
  const [activeFilePath, setActiveFilePath] = useState<string>('examples/nova_soc/rtl/core/nova_alu.sv');

  // Modals
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState<boolean>(false);
  const [cmdQuery, setCmdQuery] = useState<string>('');
  const [helpModalOpen, setHelpModalOpen] = useState<boolean>(false);

  // Load default 455-signal NOVA-SoC session + workspace files on mount
  useEffect(() => {
    let cancelled = false;
    async function init() {
      try {
        const [sessRes, filesRes] = await Promise.all([
          fetch('/api/session/default'),
          fetch('/api/files'),
        ]);
        if (!cancelled && sessRes.ok) {
          const data = await sessRes.json();
          setSessionTitle(data.title || 'nova_soc_tb');
          setSignals(data.signals || []);
          setSteps(data.steps || []);
          setEndNs(data.end || 3000);
          setNev(data.nev || 14579);
          if (data.kdb) setKdb(data.kdb);
          setVerdict(data.verdict || 'NOVA-PASS 9/9');
          setVerdictKind(data.verdictKind || 'ok');
        }
        if (!cancelled && filesRes.ok) {
          const fData = await filesRes.json();
          if (Array.isArray(fData.files) && fData.files.length > 0) {
            setFiles(fData.files);
            const alu = fData.files.find((f: ProjectFileItem) => f.path.includes('nova_alu.sv'));
            if (alu) setActiveFilePath(alu.path);
          }
        }
      } catch (err) {
        console.error('Failed to load initial session:', err);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    init();
    return () => {
      cancelled = true;
    };
  }, []);

  // Global keyboard shortcuts (Ctrl+K command palette, F1 help)
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCmdPaletteOpen((v) => !v);
      } else if (e.key === 'F1') {
        e.preventDefault();
        setHelpModalOpen((v) => !v);
      } else if (e.key === 'Escape') {
        setCmdPaletteOpen(false);
        setHelpModalOpen(false);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  // Live switching energy summary for header chip
  const totalEnergyFj = useMemo(() => {
    const rep = computeSwitchingPower(signals, steps, 0.8, 1.4, 100);
    return rep.total_energy_fj;
  }, [signals, steps]);

  // Simulate a selected target project or demo file
  const handleSelectProject = async (target: string) => {
    setActiveTarget(target);
    setSimulating(true);
    try {
      const res = await fetch('/api/simulate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ target, maxNs: target.includes('nova') ? 3000 : 1200 }),
      });
      if (res.ok) {
        const data = await res.json();
        setSessionTitle(data.title || target);
        setSignals(data.signals || []);
        setSteps(data.steps || []);
        setEndNs(data.end || 1200);
        setNev(data.nev || 0);
        if (data.kdb) setKdb(data.kdb);
        setVerdict(data.verdict || 'SIMULATED');
        setVerdictKind(data.verdictKind || 'ok');
      }
    } finally {
      setSimulating(false);
    }
  };

  // Update file buffer in state
  const handleUpdateFileContent = (path: string, content: string) => {
    setFiles((prev) => prev.map((f) => (f.path === path ? { ...f, content } : f)));
  };

  // Save file to server disk
  const handleSaveFile = async (filePath: string, content: string) => {
    handleUpdateFileContent(filePath, content);
    await fetch('/api/files/save', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ filePath, content }),
    });
  };

  // Apply L3 SystemVerilog Latch Auto-Patch
  const handleAutoPatchLatches = async (filePath: string, content: string) => {
    const res = await fetch('/api/patch-latches', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content }),
    });
    if (res.ok) {
      const data = await res.json();
      if (data.patched) {
        handleUpdateFileContent(filePath, data.patched);
        await handleSaveFile(filePath, data.patched);
      }
    }
  };

  // Simulate edited buffer directly from VIKAS-VED IDE
  const handleSimulateFile = async (filePath: string, content: string) => {
    setSimulating(true);
    try {
      await handleSaveFile(filePath, content);
      const res = await fetch('/api/simulate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          target: filePath,
          filesOverride: [{ path: filePath, content }],
          maxNs: 1200,
        }),
      });
      if (res.ok) {
        const data = await res.json();
        setActiveTarget(filePath);
        setSessionTitle(data.title || filePath);
        setSignals(data.signals || []);
        setSteps(data.steps || []);
        setEndNs(data.end || 1200);
        setNev(data.nev || 0);
        if (data.kdb) setKdb(data.kdb);
        setVerdict(data.verdict || 'PASS');
        setVerdictKind(data.verdictKind || 'ok');
      }
    } finally {
      setSimulating(false);
    }
  };

  // Create new RTL file in workspace
  const handleCreateNewFile = (path: string, content: string) => {
    setFiles((prev) => {
      if (prev.some((f) => f.path === path)) {
        return prev.map((f) => (f.path === path ? { ...f, content } : f));
      }
      return [...prev, { path, content }];
    });
    handleSaveFile(path, content);
  };

  // Jump from waveform signal or Open-KDB driver to RTL Editor
  const handleJumpToSource = (sigOrFile: string) => {
    const shortSig = sigOrFile.split('.').pop() || sigOrFile;
    const drv = kdb.drivers.find(
      (d) => d.signal === shortSig || d.file === sigOrFile || d.file.endsWith(sigOrFile)
    );
    if (drv) {
      const matchFile = files.find((f) => f.path.endsWith(drv.file));
      if (matchFile) {
        setActiveFilePath(matchFile.path);
      }
    } else {
      const byFile = files.find((f) => f.path.endsWith(sigOrFile));
      if (byFile) {
        setActiveFilePath(byFile.path);
      }
    }
    setActiveTab('editor');
  };

  // Import local .vcd or .sv file from user's disk
  const handleImportVcdOrVwf = async (file: File) => {
    const text = await file.text();
    if (/\.(sv|v)$/i.test(file.name)) {
      const sim = simulateVerilogSource([{ path: file.name, content: text }], 1500);
      setSessionTitle(`${file.name} — Imported RTL Simulation`);
      setSignals(sim.signals);
      setSteps(sim.steps);
      setEndNs(sim.end);
      setNev(sim.nev);
      setKdb(sim.kdb);
      setVerdict(sim.latches.length > 0 ? `LATCH ALERT (${sim.latches.length})` : 'CLEAN RTL');
      setVerdictKind(sim.latches.length > 0 ? 'fail' : 'ok');
      return;
    }
    // Simple IEEE-1364 VCD parser for uploaded .vcd files
    const symMap = new Map<string, { name: string; width: number; t: number[]; v: (number | string)[] }>();
    const lines = text.split(/\r?\n/);
    let curT = 0;
    for (const raw of lines) {
      const line = raw.trim();
      if (line.startsWith('$var')) {
        const parts = line.split(/\s+/);
        if (parts.length >= 5) {
          const w = parseInt(parts[2], 10) || 1;
          const sym = parts[3];
          const name = parts[4];
          symMap.set(sym, { name, width: w, t: [], v: [] });
        }
      } else if (line.startsWith('#')) {
        curT = parseInt(line.slice(1), 10) || curT;
      } else if (/^[01xzXZ]/.test(line) && line.length >= 2) {
        const val = line[0].toLowerCase();
        const sym = line.slice(1).trim();
        const ent = symMap.get(sym);
        if (ent) {
          ent.t.push(curT);
          ent.v.push(val);
        }
      } else if (/^[bB][01xzXZ]+\s+\S+/.test(line)) {
        const [bpart, sym] = line.slice(1).split(/\s+/);
        const ent = symMap.get(sym);
        if (ent) {
          ent.t.push(curT);
          ent.v.push(/[xz]/i.test(bpart) ? bpart : parseInt(bpart, 2) >>> 0);
        }
      }
    }
    if (symMap.size > 0) {
      const newSigs: SignalMeta[] = [];
      const newSteps: SignalSteps[] = [];
      for (const ent of symMap.values()) {
        newSigs.push({
          n: ent.name,
          w: ent.width,
          k: /clk|clock/i.test(ent.name) ? 'clock' : /rst|reset/i.test(ent.name) ? 'reset' : 'rtl',
        });
        newSteps.push({
          t: ent.t,
          v: ent.v,
          n: ent.v.length,
          glitch: [],
          uniq: new Set(ent.v).size,
          minT: ent.t[0] ?? 0,
          maxT: ent.t[ent.t.length - 1] ?? curT,
        });
      }
      setSessionTitle(`${file.name} — Imported VCD (${newSigs.length} signals)`);
      setSignals(newSigs);
      setSteps(newSteps);
      setEndNs(Math.max(100, curT));
      setNev(newSteps.reduce((a, s) => a + s.n, 0));
      setVerdict('VCD LOADED');
      setVerdictKind('ok');
    }
  };

  const paletteActions = [
    { label: 'Simulate NOVA-SoC (26 files · 455 signals · NOVA-PASS 9/9)', run: () => handleSelectProject('examples/nova_soc/nova.f') },
    { label: 'Simulate Inferred Latch Trap Demo (Breakthrough #13)', run: () => handleSelectProject('examples/demo_latch_trap.sv') },
    { label: 'Simulate 12-Bit SAR ADC Mixed-Signal Demo', run: () => handleSelectProject('examples/demo_sar_adc.sv') },
    { label: 'Simulate RISC-V ALU Pipeline Hazard Demo', run: () => handleSelectProject('examples/demo_riscv_alu_hazard.sv') },
    { label: 'Switch to Waveforms Studio Tab', run: () => setActiveTab('waveforms') },
    { label: 'Switch to Built-In Multi-File RTL Editor (VIKAS-VED)', run: () => setActiveTab('editor') },
    { label: 'Switch to Tapeout Signoff & Mutation Testing Hub', run: () => setActiveTab('analysis') },
    { label: 'Switch to Open-KDB, Counterfactual SAT & Silicon Physics Lab', run: () => setActiveTab('intel') },
    { label: 'Switch to 60-Command CLI, 5-Terminal QA & GitHub Sync', run: () => setActiveTab('terminal') },
    { label: 'Download Air-Gapped Linux + Rust Full Tarball (.tar.gz)', run: () => window.location.assign('/api/download-bundle') },
  ];

  const filteredPalette = paletteActions.filter((a) =>
    a.label.toLowerCase().includes(cmdQuery.trim().toLowerCase())
  );

  return (
    <div
      data-theme={theme}
      className={`h-screen w-screen flex flex-col overflow-hidden select-none ${
        theme === 'light' ? 'bg-slate-100 text-slate-900' : 'bg-[#0b0e14] text-slate-100'
      }`}
    >
      {/* Top Studio Header Bar */}
      <header className="h-12 shrink-0 bg-[#111621] border-b border-slate-800 px-3 flex items-center justify-between gap-2">
        {/* Brand + Active Session Verdict */}
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="flex items-center gap-2 px-2.5 py-1 rounded bg-gradient-to-r from-cyan-500/20 to-emerald-500/15 border border-cyan-500/40">
            <Cpu className="w-4 h-4 text-cyan-400 shrink-0" />
            <span className="font-extrabold tracking-wider text-xs text-cyan-300 font-mono">
              VIKAS STUDIO v1.2
            </span>
          </div>

          <span className="text-xs font-mono text-slate-300 truncate max-w-[260px]" title={sessionTitle}>
            {sessionTitle}
          </span>

          <span
            className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold border shrink-0 flex items-center gap-1 ${
              verdictKind === 'ok'
                ? 'bg-emerald-950/70 border-emerald-500/40 text-emerald-300'
                : 'bg-rose-950/70 border-rose-500/40 text-rose-300'
            }`}
          >
            {verdictKind === 'ok' ? (
              <CheckCircle2 className="w-3 h-3 text-emerald-400" />
            ) : (
              <ShieldAlert className="w-3 h-3 text-rose-400" />
            )}
            {verdict}
          </span>
        </div>

        {/* Center Navigation Tabs */}
        <nav className="flex items-center gap-1 bg-[#0b0e14] p-1 rounded-lg border border-slate-800">
          <button
            onClick={() => setActiveTab('waveforms')}
            className={`px-2.5 py-1 rounded text-xs font-mono font-semibold flex items-center gap-1.5 transition-colors cursor-pointer ${
              activeTab === 'waveforms'
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Activity className="w-3.5 h-3.5" />
            Waveforms ({signals.length})
          </button>

          <button
            onClick={() => setActiveTab('editor')}
            className={`px-2.5 py-1 rounded text-xs font-mono font-semibold flex items-center gap-1.5 transition-colors cursor-pointer ${
              activeTab === 'editor'
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Code2 className="w-3.5 h-3.5" />
            RTL Editor (VIKAS-VED)
          </button>

          <button
            onClick={() => setActiveTab('analysis')}
            className={`px-2.5 py-1 rounded text-xs font-mono font-semibold flex items-center gap-1.5 transition-colors cursor-pointer ${
              activeTab === 'analysis'
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Gauge className="w-3.5 h-3.5" />
            Signoff & Mutants
          </button>

          <button
            onClick={() => setActiveTab('intel')}
            className={`px-2.5 py-1 rounded text-xs font-mono font-semibold flex items-center gap-1.5 transition-colors cursor-pointer ${
              activeTab === 'intel'
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Sparkles className="w-3.5 h-3.5" />
            Open-KDB & Physics
          </button>

          <button
            onClick={() => setActiveTab('terminal')}
            className={`px-2.5 py-1 rounded text-xs font-mono font-semibold flex items-center gap-1.5 transition-colors cursor-pointer ${
              activeTab === 'terminal'
                ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Terminal className="w-3.5 h-3.5" />
            CLI · 5-Term QA · GitHub
          </button>
        </nav>

        {/* Right Telemetry Chips & Utilities */}
        <div className="hidden xl:flex items-center gap-2 text-[11px] font-mono">
          <span className="px-2 py-0.5 rounded bg-slate-900 border border-slate-800 text-slate-300">
            {Math.round(endNs / 10)} cyc · {nev.toLocaleString()} ev
          </span>
          <span className="px-2 py-0.5 rounded bg-slate-900 border border-slate-800 text-amber-300 flex items-center gap-1">
            <Zap className="w-3 h-3 text-amber-400" />
            E ≈ {totalEnergyFj.toLocaleString()} fJ
          </span>

          <button
            onClick={() => setCmdPaletteOpen(true)}
            className="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 flex items-center gap-1 cursor-pointer"
            title="Open Command Palette (Ctrl+K)"
          >
            <Command className="w-3 h-3 text-cyan-400" />
            <span>Ctrl+K</span>
          </button>

          <button
            onClick={() =>
              setTheme((prev) =>
                prev === 'dark' ? 'verdi' : prev === 'verdi' ? 'synth' : prev === 'synth' ? 'light' : 'dark'
              )
            }
            className="px-2 py-1 rounded bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 flex items-center gap-1 cursor-pointer"
            title="Cycle Studio Theme"
          >
            <Palette className="w-3 h-3 text-purple-400" />
            <span>{theme}</span>
          </button>

          <a
            href="/api/download-bundle"
            download="vikas-v1.2-linux-airgapped-full.tar.gz"
            className="px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold flex items-center gap-1"
            title="Download complete Linux + Rust + C++20 air-gapped repository tarball"
          >
            <Download className="w-3 h-3" />
            <span>Bundle</span>
          </a>

          <button
            onClick={() => setHelpModalOpen(true)}
            className="p-1 rounded bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 cursor-pointer"
            title="Keyboard Shortcuts (F1)"
          >
            <HelpCircle className="w-3.5 h-3.5" />
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 min-h-0 overflow-hidden relative">
        {loading ? (
          <div className="h-full w-full flex flex-col items-center justify-center gap-3 font-mono text-sm text-cyan-300">
            <div className="w-8 h-8 border-2 border-cyan-400 border-t-transparent rounded-full animate-spin" />
            <div>Loading VIKAS v1.2 Topo-Scheduled NOVA-SoC Session (455 signals · 14,579 events)...</div>
          </div>
        ) : activeTab === 'waveforms' ? (
          <WaveformStudioTab
            signals={signals}
            steps={steps}
            endNs={endNs}
            nev={nev}
            curA={curA}
            curB={curB}
            setCurA={setCurA}
            setCurB={setCurB}
            defRadix={defRadix}
            setDefRadix={setDefRadix}
            onSelectProject={handleSelectProject}
            activeTarget={activeTarget}
            simulating={simulating}
            onJumpToSource={handleJumpToSource}
            onImportVcdOrVwf={handleImportVcdOrVwf}
          />
        ) : activeTab === 'editor' ? (
          <RtlEditorIdeTab
            files={files}
            activeFilePath={activeFilePath}
            setActiveFilePath={setActiveFilePath}
            onUpdateFileContent={handleUpdateFileContent}
            onSaveFile={handleSaveFile}
            onAutoPatchLatches={handleAutoPatchLatches}
            onSimulateFile={handleSimulateFile}
            onCreateNewFile={handleCreateNewFile}
            signals={signals}
            steps={steps}
            cursorTimeNs={curA ?? 150}
          />
        ) : activeTab === 'analysis' ? (
          <AnalysisHubAndSignoffTab
            signals={signals}
            steps={steps}
            kdb={kdb}
            files={files}
            onJumpToSignalInWave={(_sigIdx, timeNs) => {
              if (typeof timeNs === 'number') setCurA(timeNs);
              setActiveTab('waveforms');
            }}
            onJumpToSourceFile={(fp) => handleJumpToSource(fp)}
          />
        ) : activeTab === 'intel' ? (
          <DesignIntelAndPhysicsTab
            kdb={kdb}
            signals={signals}
            steps={steps}
            cursorTimeNs={curA ?? 150}
            onJumpToSourceFile={(fp) => handleJumpToSource(fp)}
          />
        ) : (
          <TerminalAndGithubSyncTab />
        )}
      </main>

      {/* Command Palette Modal (Ctrl+K) */}
      {cmdPaletteOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/70 flex items-start justify-center pt-20"
          onClick={() => setCmdPaletteOpen(false)}
        >
          <div
            className="w-full max-w-xl rounded-lg border border-cyan-500/40 bg-[#111621] shadow-2xl overflow-hidden"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-3 border-b border-slate-800 flex items-center gap-2">
              <Command className="w-4 h-4 text-cyan-400" />
              <input
                type="text"
                autoFocus
                value={cmdQuery}
                onChange={(e) => setCmdQuery(e.target.value)}
                placeholder="Type a VIKAS Studio action, simulation target, or breakthrough..."
                className="w-full bg-transparent text-xs font-mono text-slate-100 focus:outline-none"
              />
            </div>
            <div className="max-h-72 overflow-y-auto divide-y divide-slate-800/60">
              {filteredPalette.map((item, idx) => (
                <button
                  key={idx}
                  onClick={() => {
                    item.run();
                    setCmdPaletteOpen(false);
                  }}
                  className="w-full text-left px-4 py-2.5 text-xs font-mono text-slate-200 hover:bg-cyan-950/50 hover:text-cyan-300 transition-colors cursor-pointer"
                >
                  {item.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Keyboard Shortcuts Help Modal (F1) */}
      {helpModalOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/70 flex items-center justify-center p-4"
          onClick={() => setHelpModalOpen(false)}
        >
          <div
            className="w-full max-w-lg rounded-lg border border-slate-700 bg-[#111621] p-5 space-y-3"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between border-b border-slate-800 pb-2">
              <h3 className="text-sm font-bold font-mono text-cyan-300">
                VIKAS STUDIO v1.2 — Keyboard Shortcuts
              </h3>
              <button
                onClick={() => setHelpModalOpen(false)}
                className="text-xs font-mono text-slate-400 hover:text-white"
              >
                ESC
              </button>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs font-mono text-slate-300">
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">W / S</kbd> or <kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">Wheel</kbd> : Zoom In / Out</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">A / D</kbd> : Pan Timeline Left / Right</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">F</kbd> or <kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">0</kbd> : Fit Full Simulation</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">N / P</kbd> : Next / Prev Signal Edge</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">Click</kbd> : Set Primary Cursor A</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">Shift+Click</kbd> : Set Reference Cursor B</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">Ctrl+K</kbd> : Command Palette</div>
              <div><kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-cyan-300">Double-Click</kbd> : Jump to RTL Source</div>
            </div>
            <div className="pt-2 border-t border-slate-800 flex items-center justify-between text-xs font-mono text-slate-400">
              <span>Post-Duopoly Open-Source Linux RTL Platform</span>
              <a
                href="/studio-standalone"
                target="_blank"
                rel="noreferrer"
                className="text-cyan-400 hover:underline flex items-center gap-1"
              >
                <ExternalLink className="w-3 h-3" /> Raw HTML5 Studio
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
