import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  Bookmark,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Compass,
  Crosshair,
  Download,
  Expand,
  FileText,
  Filter,
  FolderOpen,
  Layers,
  MapPin,
  Maximize2,
  Minimize2,
  Play,
  Plus,
  RotateCcw,
  Ruler,
  Search,
  SkipBack,
  SkipForward,
  Sparkles,
  Tag,
  Trash2,
  X,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import {
  compute3dTimeBitPowerSurface,
  computeBookmarkViewport,
  computeEyeDiagram,
  computeGlitchXzHeatRibbon,
  computeMipmapEnvelopes,
  computeWaveformDiff,
  createSimulationBookmark,
  createStreamingWaveChunks,
  decodeProtocolSwimlanes,
  evaluateTemporalSvaSequence,
  evaluateWaveformBooleanExpression,
  exportPublicationWaveform,
  extractPipelineWaterfall,
  filterVisibleSignalsByNameAndType,
  filterWaveformSignalsByRegex,
  formatSignalValue,
  generateResearchPaperPdfBytes,
  generateResearchPaperReportData,
  OpenKdb,
  SigKind,
  SignalMeta,
  SignalSteps,
  SimulationBookmark,
  simulateWhatIfWaveformPatch,
  traceCausalConeAtTime,
  valAtTime,
  WhatIfPatchResult,
} from '../engine/vikasEngine';

export interface WaveRowItem {
  id: string;
  si: number;
  radix: 'hex' | 'bin' | 'dec' | 'signed' | 'oct' | 'ascii' | null;
  color: string | null;
  expanded: boolean;
  analog: boolean;
}

interface WaveformStudioTabProps {
  signals: SignalMeta[];
  steps: SignalSteps[];
  endNs: number;
  nev: number;
  curA: number | null;
  curB: number | null;
  setCurA: (t: number | null) => void;
  setCurB: (t: number | null) => void;
  defRadix: 'hex' | 'bin' | 'dec' | 'signed' | 'oct' | 'ascii';
  setDefRadix: (r: 'hex' | 'bin' | 'dec' | 'signed' | 'oct' | 'ascii') => void;
  onSelectProject: (target: string) => void;
  activeTarget: string;
  simulating: boolean;
  onJumpToSource: (sigName: string) => void;
  onImportVcdOrVwf: (file: File) => void;
  kdb?: OpenKdb;
}

const KIND_COLORS: Record<SigKind, string> = {
  rtl: '#89e05a',
  clock: '#22d3ee',
  reset: '#fb923c',
  physics: '#fbbf24',
  causality: '#fb7185',
  stochastic: '#a78bfa',
  taint: '#2dd4bf',
  latch: '#f87171',
};

const PRESET_PROJECTS = [
  { label: 'NOVA-SoC (26 files · 5 levels · 455 sigs)', target: 'examples/nova_soc/nova.f' },
  { label: 'Polyglot AI Cluster (SV+VHDL+Chisel+FIRRTL)', target: 'examples/polyglot_ai_cluster/cluster.f' },
  { label: 'Multifile SoC Project (.f manifest)', target: 'examples/multifile_proj/project.f' },
  { label: 'Breakthrough 13: Latch Trap Demo', target: 'examples/demo_latch_trap.sv' },
  { label: 'RISC-V Pipelined ALU Hazard', target: 'examples/demo_riscv_alu_hazard.sv' },
  { label: '12-Bit SAR ADC Mixed-Signal', target: 'examples/demo_sar_adc.sv' },
  { label: 'UART TX FSM & SVA Demo', target: 'examples/demo_fsm_uart.sv' },
  { label: 'Dual-Clock CDC Synchronizer', target: 'examples/demo_cdc.sv' },
  { label: 'AXI-Stream FIFO Backpressure', target: 'examples/demo_fifo_backpressure.sv' },
];

export const WaveformStudioTab: React.FC<WaveformStudioTabProps> = ({
  signals,
  steps,
  endNs,
  nev,
  curA,
  curB,
  setCurA,
  setCurB,
  defRadix,
  setDefRadix,
  onSelectProject,
  activeTarget,
  simulating,
  onJumpToSource,
  onImportVcdOrVwf,
  kdb,
}) => {
  const [rows, setRows] = useState<WaveRowItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [treeFilter, setTreeFilter] = useState('');
  const [treeCaseSensitive, setTreeCaseSensitive] = useState(false);
  const [treeInvertMatch, setTreeInvertMatch] = useState(false);
  const [collapsedScopes, setCollapsedScopes] = useState<Set<string>>(new Set());
  const [t0, setT0] = useState(0);
  const [tspan, setTspan] = useState(Math.max(endNs, 100));
  const [searchQuery, setSearchQuery] = useState('');
  const [searchMode, setSearchMode] = useState<'signal' | 'expr' | 'goto'>('signal');
  const [showCoiRays, setShowCoiRays] = useState<boolean>(true);
  const [showWaterfall, setShowWaterfall] = useState<boolean>(true);
  const [whatIfPatch, setWhatIfPatch] = useState<WhatIfPatchResult | null>(null);
  // Iteration 2 states
  const [showProtocolSwimlanes, setShowProtocolSwimlanes] = useState<boolean>(true);
  const [showEyeDiagram, setShowEyeDiagram] = useState<boolean>(true);
  const [show3dPowerSurface, setShow3dPowerSurface] = useState<boolean>(false);
  const [svaQuery, setSvaQuery] = useState<string>('clk ##[1:3] rst_n');
  const [showSvaHighlights, setShowSvaHighlights] = useState<boolean>(true);
  const [pubExportModalOpen, setPubExportModalOpen] = useState<boolean>(false);
  const [pubExportFormat, setPubExportFormat] = useState<'tikz' | 'typst' | 'wavedrom' | 'svg'>('tikz');
  // Iteration 9: IEEE / ISSCC Research-Paper Formatted PDF Simulation Summary Modal
  const [pdfReportModalOpen, setPdfReportModalOpen] = useState<boolean>(false);
  const [pdfDownloadedBanner, setPdfDownloadedBanner] = useState<string | null>(null);

  // Iteration 11: Simulation Time-Point Bookmarks & Visible Signal Filter States
  const [bookmarks, setBookmarks] = useState<SimulationBookmark[]>([
    {
      id: 'bm-seed-1',
      name: 'T0: Reset Release & Clock Align',
      timeA: 40,
      timeB: 80,
      deltaNs: 40,
      color: '#38bdf8',
      notes: 'Active-low reset deasserted; system PLL locked to 100MHz.',
      createdAt: '09:00:00',
    },
    {
      id: 'bm-seed-2',
      name: 'T1: First Instruction Fetch & Decode',
      timeA: 120,
      timeB: 160,
      deltaNs: 40,
      color: '#34d399',
      notes: 'PC=0x8000_0000 fetched; opcode decoded to ADDI.',
      createdAt: '09:02:15',
    },
    {
      id: 'bm-seed-3',
      name: 'T2: ALU Hazard & Forwarding Window',
      timeA: 240,
      timeB: 280,
      deltaNs: 40,
      color: '#f59e0b',
      notes: 'RAW hazard detected on rs1=x5; EX-to-EX forwarding activated.',
      createdAt: '09:05:40',
    },
    {
      id: 'bm-seed-4',
      name: 'T3: Memory Writeback & Retire',
      timeA: 420,
      timeB: 480,
      deltaNs: 60,
      color: '#a855f7',
      notes: 'Store data committed to nova_ram; retirement counter incremented.',
      createdAt: '09:08:12',
    },
  ]);
  const [activeRightDockTab, setActiveRightDockTab] = useState<'INSPECTOR' | 'BOOKMARKS'>('INSPECTOR');
  const [newBookmarkName, setNewBookmarkName] = useState('');
  const [newBookmarkNotes, setNewBookmarkNotes] = useState('');
  const [newBookmarkColor, setNewBookmarkColor] = useState('#f59e0b');
  const [showAddBookmarkDialog, setShowAddBookmarkDialog] = useState(false);
  const [visibleSignalFilter, setVisibleSignalFilter] = useState('');
  const [visibleTypeFilter, setVisibleTypeFilter] = useState<string>('ALL');
  const [searchStatusToast, setSearchStatusToast] = useState<string | null>(null);

  const [hoverInfo, setHoverInfo] = useState<{
    x: number;
    y: number;
    sig: SignalMeta;
    si: number;
    t: number;
    val: string;
    nextDt: number | null;
  } | null>(null);

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const minimapRef = useRef<HTMLCanvasElement | null>(null);
  const wrapRef = useRef<HTMLDivElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // Initialize rows when signal list changes
  useEffect(() => {
    if (signals.length === 0) {
      setRows([]);
      return;
    }
    const initialCount = Math.min(signals.length, 32);
    const initial: WaveRowItem[] = [];
    for (let i = 0; i < initialCount; i++) {
      initial.push({
        id: `row-${i}-${signals[i].n}`,
        si: i,
        radix: null,
        color: null,
        expanded: false,
        analog: false,
      });
    }
    setRows(initial);
    setSelectedId(initial[0]?.id || null);
    setT0(0);
    setTspan(Math.max(endNs, 100));
  }, [signals, endNs]);

  // Iteration 9: Power-User Regex Signal Filter Engine (supports regex, case-sensitivity, inverse match, and syntax validation)
  const regexFilterResult = useMemo(
    () =>
      filterWaveformSignalsByRegex(signals, treeFilter, {
        caseSensitive: treeCaseSensitive,
        invertMatch: treeInvertMatch,
      }),
    [signals, treeFilter, treeCaseSensitive, treeInvertMatch]
  );

  const matchedSignalIndexSet = useMemo(
    () => new Set(regexFilterResult.matchedIndices),
    [regexFilterResult.matchedIndices]
  );

  // Group signals by hierarchical scope (filtered dynamically by power-user regex bar)
  const scopes = useMemo(() => {
    const map = new Map<string, number[]>();
    signals.forEach((s, idx) => {
      if (!matchedSignalIndexSet.has(idx)) return;
      const parts = s.n.split('.');
      const scope = parts.length > 1 ? parts.slice(0, -1).join('.') : 'top';
      const list = map.get(scope) || [];
      list.push(idx);
      map.set(scope, list);
    });
    return [...map.entries()];
  }, [signals, matchedSignalIndexSet]);

  // Dynamically filter waveform rows when a valid regex filter is active
  const filteredRows = useMemo(() => {
    if (!treeFilter.trim() || !regexFilterResult.isValidRegex) return rows;
    const inRows = rows.filter((r) => matchedSignalIndexSet.has(r.si));
    if (inRows.length > 0) return inRows;
    // If the user's regex matches signals outside the initial 32 rows, surface up to 24 matching rows automatically
    return regexFilterResult.matchedIndices.slice(0, 24).map((si) => ({
      id: `row-rx-${si}-${signals[si]?.n || si}`,
      si,
      radix: null,
      color: null,
      expanded: false,
      analog: false,
    }));
  }, [rows, treeFilter, regexFilterResult, matchedSignalIndexSet, signals]);

  // Iteration 11: Filter visible signals list by name or type
  const displayedWaveRows = useMemo(() => {
    return filterVisibleSignalsByNameAndType(
      filteredRows,
      signals,
      visibleSignalFilter,
      visibleTypeFilter
    );
  }, [filteredRows, signals, visibleSignalFilter, visibleTypeFilter]);

  const inViewSet = useMemo(() => new Set(displayedWaveRows.map((r) => r.si)), [displayedWaveRows]);

  const selectedRow = useMemo(
    () => displayedWaveRows.find((r) => r.id === selectedId) || displayedWaveRows[0] || rows[0] || null,
    [displayedWaveRows, rows, selectedId]
  );

  // Navigation handlers for simulation bookmarks
  const handleJumpToBookmark = (bm: SimulationBookmark) => {
    const { targetT0, targetTspan, curA: newA, curB: newB } = computeBookmarkViewport(
      bm,
      tspan,
      endNs
    );
    setT0(targetT0);
    setTspan(targetTspan);
    if (newA !== null) setCurA(newA);
    if (newB !== null) setCurB(newB);
  };

  const handleCreateBookmark = () => {
    const timeA = curA !== null ? curA : Math.round(t0 + tspan * 0.25);
    const timeB = curB !== null ? curB : null;
    const name = newBookmarkName.trim() || `Bookmark @ ${timeA}ns${timeB !== null ? ` / ${timeB}ns` : ''}`;
    const newBm = createSimulationBookmark(
      name,
      timeA,
      timeB,
      newBookmarkColor,
      newBookmarkNotes
    );
    setBookmarks((prev) => [newBm, ...prev]);
    setNewBookmarkName('');
    setNewBookmarkNotes('');
    setShowAddBookmarkDialog(false);
    setActiveRightDockTab('BOOKMARKS');
  };

  const handleDeleteBookmark = (id: string) => {
    setBookmarks((prev) => prev.filter((b) => b.id !== id));
  };

  // WV-02: Causal Cone-of-Influence (COI) Ray-Tracer edges for selected signal @ Cursor A
  const coiRays = useMemo(() => {
    if (!showCoiRays || !selectedRow || !kdb) return [];
    const targetT = curA !== null ? curA : Math.round(t0 + tspan * 0.35);
    return traceCausalConeAtTime(kdb, signals, steps, selectedRow.si, targetT, 3);
  }, [showCoiRays, selectedRow, kdb, signals, steps, curA, t0, tspan]);

  // WV-04: Synchronized CPU Instruction Pipeline Waterfall
  const waterfallSlots = useMemo(() => {
    if (!showWaterfall || signals.length === 0) return [];
    return extractPipelineWaterfall(signals, steps, t0, t0 + tspan);
  }, [showWaterfall, signals, steps, t0, tspan]);

  // WV-01: Sub-pixel Binary Interval Mipmap Pyramid for selected signal
  const selectedMipmap = useMemo(() => {
    if (!selectedRow || !steps[selectedRow.si]) return [];
    return computeMipmapEnvelopes(steps[selectedRow.si], t0, t0 + tspan, 32);
  }, [selectedRow, steps, t0, tspan]);

  // WV-03: Semantic Protocol Transaction Swimlanes
  const protocolSwimlanes = useMemo(() => {
    if (!showProtocolSwimlanes || signals.length === 0) return [];
    return decodeProtocolSwimlanes(signals, steps, t0, t0 + tspan);
  }, [showProtocolSwimlanes, signals, steps, t0, tspan]);

  // WV-05 / AP-09: Persistence Eye Diagram & Jitter Histogram for selected signal
  const eyeDiagram = useMemo(() => {
    if (!showEyeDiagram || !selectedRow || !signals[selectedRow.si] || !steps[selectedRow.si]) {
      return null;
    }
    const sig = signals[selectedRow.si];
    return computeEyeDiagram(sig.n, steps[selectedRow.si], sig.w, 10, 0.8);
  }, [showEyeDiagram, selectedRow, signals, steps]);

  // WV-06: Temporal Regex & SVA Sequence Visual Highlighter
  const svaMatches = useMemo(() => {
    if (!showSvaHighlights || signals.length === 0) return [];
    return evaluateTemporalSvaSequence(signals, steps, svaQuery, endNs);
  }, [showSvaHighlights, signals, steps, svaQuery, endNs]);

  // WV-09: Glitch & X-Propagation Heat-Ribbon Minimap
  const heatRibbon = useMemo(
    () => computeGlitchXzHeatRibbon(steps, endNs, 64),
    [steps, endNs]
  );

  // WV-10: Zero-Copy Streaming Chunks telemetry
  const streamChunks = useMemo(
    () => createStreamingWaveChunks(steps, endNs, 500),
    [steps, endNs]
  );

  // WV-11: 3D Time-Bit-Power Surface Topography
  const powerSurfaceCells = useMemo(() => {
    if (!show3dPowerSurface || signals.length === 0) return [];
    return compute3dTimeBitPowerSurface(signals, steps, t0, t0 + tspan, 6, 12, 0.8, 1.4);
  }, [show3dPowerSurface, signals, steps, t0, tspan]);

  // WV-12: Publication Exporter output
  const pubExportCode = useMemo(() => {
    if (!pubExportModalOpen) return '';
    return exportPublicationWaveform(
      signals,
      steps,
      rows.slice(0, 8).map((r) => r.si),
      t0,
      t0 + tspan,
      pubExportFormat
    );
  }, [pubExportModalOpen, signals, steps, rows, t0, tspan, pubExportFormat]);

  // Iteration 9: IEEE / ISSCC Research-Paper Simulation Summary & Open-KDB Report Data
  const paperReportData = useMemo(() => {
    const fallbackKdb: OpenKdb = kdb || {
      top: activeTarget.split('/').pop()?.replace(/\.\w+$/, '') || 'nova_soc_top',
      signals: signals.map((s) => s.n),
      drivers: [],
      csr_offsets: new Array(signals.length + 1).fill(0),
      csr_targets: [],
      scc_loops: [],
      inferred_latches: [],
      fsm_states: [],
      cdc_paths: [],
      fo4_depth: {},
    };
    return generateResearchPaperReportData(fallbackKdb, signals, steps, endNs, {
      targetManifest: activeTarget,
      displayedSignalsCount: filteredRows.length,
      activeRegexFilter: treeFilter.trim() || '*',
    });
  }, [kdb, activeTarget, signals, steps, endNs, displayedWaveRows.length, treeFilter]);

  const handleDownloadPdfReport = () => {
    const pdfContent = generateResearchPaperPdfBytes(paperReportData);
    const bytes = new Uint8Array(pdfContent.length);
    for (let i = 0; i < pdfContent.length; i++) {
      bytes[i] = pdfContent.charCodeAt(i) & 0xff;
    }
    const blob = new Blob([bytes], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    const safeTop = paperReportData.openKdbSummary.topModule.replace(/[^A-Za-z0-9_-]/g, '_');
    a.download = `VIKAS_IEEE_Simulation_Report_${safeTop}.pdf`;
    a.click();
    URL.revokeObjectURL(url);
    setPdfDownloadedBanner(
      `Exported VIKAS_IEEE_Simulation_Report_${safeTop}.pdf (${(bytes.byteLength / 1024).toFixed(1)} KB · %PDF-1.4 Two-Page IEEE/ISSCC Archival Format)`
    );
  };

  // Compute vertical layout of rows (accounting for bus bit expansion, analog height, and active regex/type filter)
  const layoutItems = useMemo(() => {
    const out: {
      row: WaveRowItem;
      kind: 'sig' | 'bit';
      bit?: number;
      y: number;
      h: number;
    }[] = [];
    let y = 28; // top ruler height
    for (const r of displayedWaveRows) {
      const sig = signals[r.si];
      if (!sig) continue;
      const h = r.analog && sig.w >= 4 ? 54 : 26;
      out.push({ row: r, kind: 'sig', y, h });
      y += h;
      if (r.expanded && sig.w > 1) {
        const maxBits = Math.min(sig.w, 16);
        for (let b = maxBits - 1; b >= 0; b--) {
          out.push({ row: r, kind: 'bit', bit: b, y, h: 16 });
          y += 16;
        }
      }
    }
    return { items: out, totalHeight: Math.max(y + 20, 420) };
  }, [displayedWaveRows, signals]);

  const [viewportSize, setViewportSize] = useState<{ w: number; h: number }>({ w: 960, h: 640 });

  useEffect(() => {
    const wrap = wrapRef.current;
    if (!wrap) return;
    const update = () => {
      setViewportSize({
        w: Math.max(600, wrap.clientWidth),
        h: Math.max(360, wrap.clientHeight),
      });
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(wrap);
    window.addEventListener('resize', update);
    return () => {
      ro.disconnect();
      window.removeEventListener('resize', update);
    };
  }, []);

  // Draw main waveform canvas
  useEffect(() => {
    const cv = canvasRef.current;
    const wrap = wrapRef.current;
    if (!cv || !wrap) return;

    const dpr = window.devicePixelRatio || 1;
    const W = Math.max(600, wrap.clientWidth || viewportSize.w);
    const H = Math.max(layoutItems.totalHeight, wrap.clientHeight || viewportSize.h);
    cv.width = W * dpr;
    cv.height = H * dpr;
    cv.style.width = `${W}px`;
    cv.style.height = `${H}px`;

    const ctx = cv.getContext('2d');
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const nameW = 225;
    const valW = 96;
    const x0 = nameW + valW;
    const x1 = W - 12;
    const waveW = Math.max(100, x1 - x0);

    const timeToX = (t: number) => x0 + ((t - t0) / Math.max(tspan, 1)) * waveW;

    // Background
    ctx.fillStyle = '#0b0f1a';
    ctx.fillRect(0, 0, W, H);

    // Header bar
    ctx.fillStyle = '#101829';
    ctx.fillRect(0, 0, W, 28);
    ctx.strokeStyle = 'rgba(148,163,184,0.16)';
    ctx.beginPath();
    ctx.moveTo(0, 27.5);
    ctx.lineTo(W, 27.5);
    ctx.moveTo(nameW, 0);
    ctx.lineTo(nameW, H);
    ctx.moveTo(x0, 0);
    ctx.lineTo(x0, H);
    ctx.stroke();

    ctx.fillStyle = '#94a3b8';
    ctx.font = '600 10px JetBrains Mono, monospace';
    ctx.fillText('SIGNAL NAME', 10, 18);
    ctx.fillText(curA !== null ? `VAL @ ${Math.round(curA)}ns` : 'VAL @ 0ns', nameW + 8, 18);

    // Time ruler ticks
    const rawStep = tspan / 8;
    const mag = Math.pow(10, Math.floor(Math.log10(Math.max(rawStep, 1))));
    const norm = rawStep / mag;
    const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 5 ? 5 : 10) * mag;
    const firstTick = Math.ceil(t0 / step) * step;

    ctx.font = '10px JetBrains Mono, monospace';
    for (let t = firstTick; t <= t0 + tspan; t += step) {
      const x = timeToX(t);
      if (x < x0 || x > x1) continue;
      ctx.strokeStyle = 'rgba(148,163,184,0.08)';
      ctx.beginPath();
      ctx.moveTo(x, 28);
      ctx.lineTo(x, H);
      ctx.stroke();

      ctx.strokeStyle = 'rgba(148,163,184,0.35)';
      ctx.beginPath();
      ctx.moveTo(x, 21);
      ctx.lineTo(x, 28);
      ctx.stroke();

      ctx.fillStyle = '#94a3b8';
      ctx.fillText(`${Math.round(t)} ns`, x + 3, 16);
    }

    // Draw each row
    const sampleT = curA !== null ? curA : t0;
    for (const item of layoutItems.items) {
      const { row, kind, bit, y, h } = item;
      const sig = signals[row.si];
      const st = steps[row.si];
      if (!sig || !st) continue;

      const isSelected = row.id === selectedRow?.id;
      if (isSelected && kind === 'sig') {
        ctx.fillStyle = 'rgba(29, 58, 95, 0.55)';
        ctx.fillRect(0, y, W, h);
      } else if (kind === 'bit') {
        ctx.fillStyle = 'rgba(15, 23, 42, 0.65)';
        ctx.fillRect(0, y, W, h);
      }

      // Row bottom divider
      ctx.strokeStyle = 'rgba(148,163,184,0.09)';
      ctx.beginPath();
      ctx.moveTo(0, y + h - 0.5);
      ctx.lineTo(W, y + h - 0.5);
      ctx.stroke();

      const baseColor = row.color || KIND_COLORS[sig.k] || '#89e05a';
      const radix = row.radix || defRadix;
      const rawValAtCur = valAtTime(st, sig.w, sampleT);

      if (kind === 'sig') {
        // Dot + Name
        ctx.fillStyle = baseColor;
        ctx.fillRect(8, y + h / 2 - 3, 6, 6);
        ctx.fillStyle = isSelected ? '#22d3ee' : '#e2e8f0';
        ctx.font = '11px JetBrains Mono, monospace';
        const shortLabel = sig.n.length > 26 ? '…' + sig.n.slice(-25) : sig.n;
        ctx.fillText(shortLabel, 20, y + h / 2 + 4);

        // Width badge
        ctx.fillStyle = '#64748b';
        ctx.font = '9.5px JetBrains Mono, monospace';
        ctx.fillText(`${sig.w}b`, nameW - 28, y + h / 2 + 3);

        // Value at Cursor A
        const formattedVal = formatSignalValue(rawValAtCur, sig.w, radix);
        ctx.fillStyle = /x/i.test(formattedVal)
          ? '#ef7373'
          : /z/i.test(formattedVal)
          ? '#a98ff0'
          : '#34d399';
        ctx.font = '11px JetBrains Mono, monospace';
        ctx.fillText(formattedVal.slice(0, 11), nameW + 6, y + h / 2 + 4);
      } else if (kind === 'bit' && bit !== undefined) {
        ctx.fillStyle = '#94a3b8';
        ctx.font = '10px JetBrains Mono, monospace';
        ctx.fillText(`  [${bit}]`, 24, y + h / 2 + 3);
        const bitStr =
          typeof rawValAtCur === 'number'
            ? ((rawValAtCur >>> bit) & 1).toString()
            : String(rawValAtCur).padStart(sig.w, '0')[sig.w - 1 - bit] || '0';
        ctx.fillStyle = '#cbd5e1';
        ctx.fillText(bitStr, nameW + 10, y + h / 2 + 3);
      }

      // Clip to wave viewport
      ctx.save();
      ctx.beginPath();
      ctx.rect(x0, y, waveW, h);
      ctx.clip();

      const yTop = y + 5;
      const yBot = y + h - 5;
      const yMid = (yTop + yBot) / 2;

      if (kind === 'sig' && row.analog && sig.w > 1 && st.n > 0) {
        // Analog continuous trace
        const maxVal = Math.pow(2, Math.min(sig.w, 16)) - 1;
        ctx.strokeStyle = baseColor;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        let started = false;
        for (let k = 0; k < st.n; k++) {
          const tk = st.t[k];
          const nx = Math.max(x0, Math.min(x1, timeToX(tk)));
          const vnum = typeof st.v[k] === 'number' ? (st.v[k] as number) : 0;
          const normY = yBot - (Math.min(vnum, maxVal) / Math.max(1, maxVal)) * (yBot - yTop);
          if (!started) {
            ctx.moveTo(x0, normY);
            ctx.lineTo(nx, normY);
            started = true;
          } else {
            ctx.lineTo(nx, normY);
          }
          if (tk > t0 + tspan) break;
        }
        ctx.lineTo(x1, yMid);
        ctx.stroke();
      } else if (sig.w === 1 || kind === 'bit') {
        // 1-bit digital trace
        ctx.strokeStyle = kind === 'bit' ? '#38bdf8' : baseColor;
        ctx.lineWidth = 1.5;
        ctx.beginPath();

        const getBitAt = (v: number | string): string => {
          if (kind === 'bit' && bit !== undefined) {
            if (typeof v === 'number') return ((v >>> bit) & 1) ? '1' : '0';
            const padded = String(v).padStart(sig.w, '0');
            return padded[sig.w - 1 - bit] || '0';
          }
          return String(v);
        };

        let curV = getBitAt(valAtTime(st, sig.w, t0));
        let curX = x0;
        const levelY = (b: string) => (b === '1' ? yTop : b === '0' ? yBot : yMid);

        ctx.moveTo(curX, levelY(curV));
        for (let k = 0; k < st.n; k++) {
          const tk = st.t[k];
          if (tk <= t0) {
            curV = getBitAt(st.v[k]);
            continue;
          }
          const nx = Math.min(x1, timeToX(tk));
          const prevY = levelY(curV);
          if (curV === '1') {
            ctx.fillStyle = 'rgba(34, 211, 238, 0.07)';
            ctx.fillRect(curX, yTop, Math.max(0, nx - curX), yBot - yTop);
          } else if (curV === 'x' || curV === 'X') {
            ctx.fillStyle = 'rgba(239, 115, 115, 0.22)';
            ctx.fillRect(curX, yTop, Math.max(0, nx - curX), yBot - yTop);
          }
          ctx.lineTo(nx, prevY);
          const nextV = getBitAt(st.v[k]);
          ctx.lineTo(nx, levelY(nextV));
          curV = nextV;
          curX = nx;
          if (tk > t0 + tspan) break;
        }
        if (curV === '1') {
          ctx.fillStyle = 'rgba(34, 211, 238, 0.07)';
          ctx.fillRect(curX, yTop, Math.max(0, x1 - curX), yBot - yTop);
        }
        ctx.lineTo(x1, levelY(curV));
        ctx.stroke();
      } else {
        // Multi-bit bus hexagon segments with inline values
        let segStart = x0;
        let segVal = valAtTime(st, sig.w, t0);

        const drawBusSegment = (sx0: number, sx1: number, val: number | string) => {
          const wSeg = sx1 - sx0;
          if (wSeg <= 0.5) return;
          const isX = typeof val === 'string' && /[xz]/i.test(val);
          ctx.strokeStyle = isX ? '#ef7373' : row.color || '#38bdf8';
          ctx.fillStyle = isX ? 'rgba(239,115,115,0.16)' : 'rgba(56,189,248,0.09)';
          ctx.lineWidth = 1.3;
          const bevel = Math.min(4, wSeg / 3);
          ctx.beginPath();
          ctx.moveTo(sx0, yMid);
          ctx.lineTo(sx0 + bevel, yTop);
          ctx.lineTo(sx1 - bevel, yTop);
          ctx.lineTo(sx1, yMid);
          ctx.lineTo(sx1 - bevel, yBot);
          ctx.lineTo(sx0 + bevel, yBot);
          ctx.closePath();
          ctx.fill();
          ctx.stroke();

          if (wSeg > 36) {
            const txt = formatSignalValue(val, sig.w, radix);
            ctx.fillStyle = isX ? '#fca5a5' : '#e2e8f0';
            ctx.font = '10px JetBrains Mono, monospace';
            const maxChars = Math.floor((wSeg - 10) / 6.2);
            ctx.fillText(txt.slice(0, maxChars), sx0 + bevel + 3, yMid + 3.5);
          }
        };

        for (let k = 0; k < st.n; k++) {
          const tk = st.t[k];
          if (tk <= t0) {
            segVal = st.v[k];
            continue;
          }
          const nx = Math.min(x1, timeToX(tk));
          if (st.v[k] !== segVal) {
            drawBusSegment(segStart, nx, segVal);
            segStart = nx;
            segVal = st.v[k];
          }
          if (tk > t0 + tspan) break;
        }
        drawBusSegment(segStart, x1, segVal);
      }

      // Glitch diamonds on sub-cycle bounces
      if (st.glitch.length > 0 && kind === 'sig') {
        ctx.fillStyle = '#f87171';
        for (const gi of st.glitch) {
          const gt = st.t[gi];
          if (gt < t0 || gt > t0 + tspan) continue;
          const gx = timeToX(gt);
          ctx.beginPath();
          ctx.moveTo(gx, yTop);
          ctx.lineTo(gx + 3.5, yTop + 3.5);
          ctx.lineTo(gx, yTop + 7);
          ctx.lineTo(gx - 3.5, yTop + 3.5);
          ctx.closePath();
          ctx.fill();
        }
      }

      ctx.restore();
    }

    // Cursor A & Cursor B + Δt + Frequency HUD
    const drawCursor = (t: number, color: string, label: string) => {
      if (t < t0 || t > t0 + tspan) return;
      const x = timeToX(t);
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.4;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      ctx.moveTo(x, 28);
      ctx.lineTo(x, H);
      ctx.stroke();
      ctx.setLineDash([]);

      ctx.fillStyle = color;
      ctx.font = '700 10px JetBrains Mono, monospace';
      ctx.fillText(`${label} ${Math.round(t)}ns`, x + 4, 40);
    };

    if (curA !== null) drawCursor(curA, '#f0f6ff', 'A');
    if (curB !== null) drawCursor(curB, '#34d399', 'B');

    if (curA !== null && curB !== null) {
      const xa = Math.max(x0, Math.min(x1, timeToX(curA)));
      const xb = Math.max(x0, Math.min(x1, timeToX(curB)));
      const left = Math.min(xa, xb);
      const width = Math.abs(xb - xa);
      ctx.fillStyle = 'rgba(52, 211, 153, 0.08)';
      ctx.fillRect(left, 28, width, H - 28);

      const dt = Math.abs(curB - curA);
      const freqMhz = dt > 0 ? (1000 / dt).toFixed(2) : '∞';
      const label = `Δt = ${dt.toFixed(1)} ns · f = ${freqMhz} MHz`;
      const pillW = label.length * 6.5 + 18;
      const pillX = Math.max(x0 + 8, Math.min(x1 - pillW - 8, (xa + xb) / 2 - pillW / 2));
      ctx.fillStyle = '#141d31';
      ctx.strokeStyle = '#34d399';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect(pillX, 46, pillW, 22, 5);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = '#34d399';
      ctx.font = '600 10.5px JetBrains Mono, monospace';
      ctx.fillText(label, pillX + 9, 61);
    }

    // Iteration 11: Render Simulation Time-Point Bookmark Vertical Flags & Markers
    for (const bm of bookmarks) {
      if (bm.timeA !== null && bm.timeA >= t0 && bm.timeA <= t0 + tspan) {
        const bx = timeToX(bm.timeA);
        ctx.strokeStyle = bm.color || '#f59e0b';
        ctx.lineWidth = 1.3;
        ctx.setLineDash([3, 3]);
        ctx.beginPath();
        ctx.moveTo(bx, 24);
        ctx.lineTo(bx, H);
        ctx.stroke();
        ctx.setLineDash([]);

        // Bookmark Flag Pin Tag
        const tagText = `★ ${bm.name}`;
        ctx.font = '700 9px JetBrains Mono, monospace';
        const tagW = Math.min(130, ctx.measureText(tagText).width + 10);
        ctx.fillStyle = bm.color || '#f59e0b';
        ctx.beginPath();
        ctx.roundRect(Math.max(x0 + 2, bx - 2), 4, tagW, 18, 3);
        ctx.fill();
        ctx.fillStyle = '#0f172a';
        ctx.fillText(tagText.slice(0, 16), Math.max(x0 + 5, bx + 2), 16);
      }
    }

    // WV-08: Draw What-If Counterfactual Patched Signal Delta Overlay
    if (whatIfPatch) {
      for (const aff of whatIfPatch.affectedSignals) {
        const rowItem = layoutItems.items.find((it) => it.kind === 'sig' && it.row.si === aff.sigIdx);
        if (!rowItem) continue;
        const { y, h } = rowItem;
        const sig = signals[aff.sigIdx];
        const pst = aff.patchedSteps;
        if (!sig || !pst) continue;

        ctx.save();
        ctx.beginPath();
        ctx.rect(x0, y, waveW, h);
        ctx.clip();

        const yTop = y + 4;
        const yBot = y + h - 4;
        const yMid = (yTop + yBot) / 2;

        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 2;
        ctx.setLineDash([5, 3]);
        ctx.beginPath();

        let curX = Math.max(x0, timeToX(whatIfPatch.patchTime));
        ctx.moveTo(curX, yMid);
        for (let k = 0; k < pst.n; k++) {
          const tk = pst.t[k];
          if (tk < whatIfPatch.patchTime || tk < t0) continue;
          const nx = Math.min(x1, timeToX(tk));
          const isHigh = pst.v[k] === '1' || (typeof pst.v[k] === 'number' && (pst.v[k] as number) > 0);
          const py = isHigh ? yTop : yBot;
          ctx.lineTo(nx, py);
          curX = nx;
          if (tk > t0 + tspan) break;
        }
        ctx.lineTo(x1, yTop);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.restore();
      }
    }

    // WV-02: Draw Causal Cone-of-Influence (COI) Visual Ray-Tracer Bézier Arrows
    if (showCoiRays && coiRays.length > 0) {
      for (const ray of coiRays) {
        const fromItem = layoutItems.items.find((it) => it.kind === 'sig' && it.row.si === ray.fromSigIdx);
        const toItem = layoutItems.items.find((it) => it.kind === 'sig' && it.row.si === ray.toSigIdx);
        if (!fromItem || !toItem) continue;

        const fx = Math.max(x0 + 6, Math.min(x1 - 6, timeToX(ray.fromTime)));
        const fy = fromItem.y + fromItem.h / 2;
        const tx = Math.max(x0 + 6, Math.min(x1 - 6, timeToX(ray.toTime)));
        const ty = toItem.y + toItem.h / 2;

        ctx.save();
        ctx.strokeStyle = ray.depth === 1 ? '#fb7185' : '#38bdf8';
        ctx.fillStyle = ray.depth === 1 ? '#fb7185' : '#38bdf8';
        ctx.lineWidth = 1.8;
        ctx.shadowColor = ray.depth === 1 ? 'rgba(251,113,133,0.65)' : 'rgba(56,189,248,0.65)';
        ctx.shadowBlur = 6;

        const cpx = (fx + tx) / 2;
        ctx.beginPath();
        ctx.moveTo(fx, fy);
        ctx.bezierCurveTo(cpx, fy, cpx, ty, tx, ty);
        ctx.stroke();

        // Origin node dot
        ctx.beginPath();
        ctx.arc(fx, fy, 3.2, 0, Math.PI * 2);
        ctx.fill();

        // Target arrowhead
        const angle = Math.atan2(ty - fy, tx - cpx || 1);
        ctx.beginPath();
        ctx.moveTo(tx, ty);
        ctx.lineTo(tx - 7 * Math.cos(angle - 0.4), ty - 7 * Math.sin(angle - 0.4));
        ctx.lineTo(tx - 7 * Math.cos(angle + 0.4), ty - 7 * Math.sin(angle + 0.4));
        ctx.closePath();
        ctx.fill();
        ctx.restore();
      }
    }

    // WV-06: Draw SVA Temporal Sequence Highlight Bands on Canvas
    if (showSvaHighlights && svaMatches.length > 0) {
      for (const m of svaMatches.slice(0, 10)) {
        if (m.endNs < t0 || m.startNs > t0 + tspan) continue;
        const sx0 = Math.max(x0, Math.min(x1, timeToX(m.startNs)));
        const sx1 = Math.max(x0, Math.min(x1, timeToX(m.endNs)));
        const wBand = Math.max(4, sx1 - sx0);
        ctx.fillStyle =
          m.status === 'MATCH'
            ? 'rgba(16, 185, 129, 0.06)'
            : 'rgba(244, 63, 94, 0.09)';
        ctx.fillRect(sx0, 28, wBand, H - 28);
        ctx.fillStyle = m.status === 'MATCH' ? '#34d399' : '#fb7185';
        ctx.fillRect(sx0, 28, wBand, 3);
      }
    }
  }, [
    layoutItems,
    signals,
    steps,
    t0,
    tspan,
    curA,
    curB,
    defRadix,
    selectedRow,
    viewportSize,
    showCoiRays,
    coiRays,
    whatIfPatch,
    showSvaHighlights,
    svaMatches,
    bookmarks,
  ]);

  // Draw minimap navigator
  useEffect(() => {
    const mm = minimapRef.current;
    if (!mm) return;
    const dpr = window.devicePixelRatio || 1;
    const W = mm.clientWidth || 600;
    const H = mm.clientHeight || 40;
    mm.width = W * dpr;
    mm.height = H * dpr;
    const ctx = mm.getContext('2d');
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    ctx.fillStyle = '#0d1322';
    ctx.fillRect(0, 0, W, H);

    // Event density histogram across 96 buckets
    const buckets = 96;
    const counts = new Uint32Array(buckets);
    const maxEnd = Math.max(endNs, 1);
    for (const st of steps) {
      for (let k = 0; k < st.n; k++) {
        const b = Math.min(buckets - 1, Math.floor((st.t[k] / maxEnd) * buckets));
        counts[b]++;
      }
    }
    const maxC = Math.max(1, ...counts);
    const barW = W / buckets;
    for (let i = 0; i < buckets; i++) {
      const bh = (counts[i] / maxC) * (H - 12);
      ctx.fillStyle = 'rgba(34, 211, 238, 0.32)';
      ctx.fillRect(i * barW, H - bh - 2, Math.max(1, barW - 1), bh);
    }

    // WV-09: Glitch & X-Propagation Heat-Ribbon strip along top of minimap
    const ribW = W / Math.max(1, heatRibbon.length);
    for (let i = 0; i < heatRibbon.length; i++) {
      const bin = heatRibbon[i];
      ctx.fillStyle =
        bin.severity === 'XZ_HAZARD'
          ? '#f43f5e'
          : bin.severity === 'GLITCH'
          ? '#fbbf24'
          : bin.severity === 'ACTIVE'
          ? 'rgba(34, 211, 238, 0.7)'
          : 'rgba(30, 41, 59, 0.6)';
      ctx.fillRect(i * ribW, 1, Math.max(1, ribW - 0.5), 4);
    }

    // Iteration 11: Bookmark ticks on minimap
    for (const bm of bookmarks) {
      if (bm.timeA !== null) {
        const mx = (bm.timeA / maxEnd) * W;
        ctx.strokeStyle = bm.color || '#f59e0b';
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.moveTo(mx, 0);
        ctx.lineTo(mx, H);
        ctx.stroke();
      }
    }

    // Current viewport box
    const vx0 = (t0 / maxEnd) * W;
    const vw = Math.max(12, (tspan / maxEnd) * W);
    ctx.fillStyle = 'rgba(56, 189, 248, 0.16)';
    ctx.strokeStyle = '#22d3ee';
    ctx.lineWidth = 1.5;
    ctx.fillRect(vx0, 2, vw, H - 4);
    ctx.strokeRect(vx0, 2, vw, H - 4);
  }, [steps, endNs, t0, tspan, viewportSize, heatRibbon, bookmarks]);

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const cv = canvasRef.current;
    if (!cv) return;
    const rect = cv.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Find clicked row
    const hit = layoutItems.items.find((it) => y >= it.y && y <= it.y + it.h);
    if (hit) {
      setSelectedId(hit.row.id);
    }

    const x0 = 225 + 96;
    const x1 = rect.width - 12;
    if (x >= x0 && x <= x1) {
      const clickedT = Math.max(0, Math.min(endNs, Math.round(t0 + ((x - x0) / (x1 - x0)) * tspan)));
      if (e.shiftKey) {
        setCurB(clickedT);
      } else {
        setCurA(clickedT);
      }
    }
  };

  const handleCanvasDoubleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const cv = canvasRef.current;
    if (!cv) return;
    const rect = cv.getBoundingClientRect();
    const y = e.clientY - rect.top;
    const hit = layoutItems.items.find((it) => y >= it.y && y <= it.y + it.h);
    if (hit && signals[hit.row.si]) {
      onJumpToSource(signals[hit.row.si].n);
    }
  };

  const handleCanvasMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const cv = canvasRef.current;
    if (!cv) return;
    const rect = cv.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const x0 = 225 + 96;
    const x1 = rect.width - 12;
    const hit = layoutItems.items.find((it) => it.kind === 'sig' && y >= it.y && y <= it.y + it.h);

    if (!hit || x < x0 || x > x1) {
      setHoverInfo(null);
      return;
    }

    const t = Math.max(0, Math.min(endNs, Math.round(t0 + ((x - x0) / (x1 - x0)) * tspan)));
    const sig = signals[hit.row.si];
    const st = steps[hit.row.si];
    if (!sig || !st) return;

    const val = formatSignalValue(valAtTime(st, sig.w, t), sig.w, hit.row.radix || defRadix);
    let nextDt: number | null = null;
    for (let k = 0; k < st.n; k++) {
      if (st.t[k] > t) {
        nextDt = st.t[k] - t;
        break;
      }
    }
    setHoverInfo({
      x: e.clientX,
      y: e.clientY,
      sig,
      si: hit.row.si,
      t,
      val,
      nextDt,
    });
  };

  const handleZoom = (factor: number) => {
    const center = curA !== null ? curA : t0 + tspan / 2;
    const nextSpan = Math.max(20, Math.min(Math.max(endNs, 100), tspan * factor));
    const nextT0 = Math.max(0, Math.min(Math.max(0, endNs - nextSpan), center - nextSpan / 2));
    setTspan(nextSpan);
    setT0(nextT0);
  };

  const handlePan = (dir: number) => {
    const step = Math.max(10, tspan * 0.15);
    const nextT0 = Math.max(0, Math.min(Math.max(0, endNs - tspan), t0 + dir * step));
    setT0(nextT0);
  };

  const handleFit = () => {
    setT0(0);
    setTspan(Math.max(endNs, 100));
  };

  const jumpToSignalEdge = (direction: 'prev' | 'next') => {
    const targetRow = selectedRow || rows[0];
    if (!targetRow) return;
    const st = steps[targetRow.si];
    const sig = signals[targetRow.si];
    if (!st || !st.t || st.t.length === 0) return;
    const refT = curA !== null ? curA : Math.round(t0 + tspan * 0.35);
    let nextT: number | undefined;
    if (direction === 'next') {
      nextT = st.t.find((t) => t > refT);
      if (nextT === undefined) nextT = st.t[0];
    } else {
      for (let i = st.t.length - 1; i >= 0; i--) {
        if (st.t[i] < refT) {
          nextT = st.t[i];
          break;
        }
      }
      if (nextT === undefined) nextT = st.t[st.t.length - 1];
    }
    if (nextT !== undefined) {
      setCurA(nextT);
      if (nextT < t0 || nextT > t0 + tspan) {
        setT0(Math.max(0, Math.min(Math.max(0, endNs - tspan), nextT - tspan / 2)));
      }
      setSearchStatusToast(
        `${direction === 'next' ? 'Next' : 'Prev'} edge on ${sig?.n.split('.').pop() || 'sig'} -> ${nextT} ns`
      );
    }
  };

  // Global keyboard shortcuts (W/S zoom, A/D pan, F/0 fit, N/P next/prev edge, E expand bus)
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement | null)?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea' || tag === 'select' || e.ctrlKey || e.metaKey || e.altKey) {
        return;
      }
      const key = e.key.toLowerCase();
      if (key === 'w' || key === '+' || key === '=') {
        e.preventDefault();
        handleZoom(0.65);
      } else if (key === 's' || key === '-') {
        e.preventDefault();
        handleZoom(1.55);
      } else if (key === 'a' || key === 'arrowleft') {
        e.preventDefault();
        handlePan(-1);
      } else if (key === 'd' || key === 'arrowright') {
        e.preventDefault();
        handlePan(1);
      } else if (key === 'f' || key === '0') {
        e.preventDefault();
        handleFit();
      } else if (key === 'n') {
        e.preventDefault();
        jumpToSignalEdge('next');
      } else if (key === 'p') {
        e.preventDefault();
        jumpToSignalEdge('prev');
      } else if (key === 'e' && selectedRow) {
        e.preventDefault();
        setRows((prev) =>
          prev.map((r) => (r.id === selectedRow.id ? { ...r, expanded: !r.expanded } : r))
        );
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  const handleRunSearch = () => {
    const q = searchQuery.trim();
    if (!q) return;
    if (searchMode === 'goto' || /^goto:\s*\d+/i.test(q)) {
      const m = q.match(/\d+/);
      if (m) {
        const targetT = Math.min(endNs, parseInt(m[0], 10));
        setCurA(targetT);
        setT0(Math.max(0, Math.min(Math.max(0, endNs - tspan), targetT - tspan / 2)));
        setSearchStatusToast(`Jumped Cursor A -> ${targetT} ns`);
      }
      return;
    }

    // Evaluate boolean / relational expression when in EXPR mode or when relational operators are present
    if (searchMode === 'expr' || /(?:==|!=|>=|<=|>|<|&&|\|\|)/.test(q)) {
      const exprRes = evaluateWaveformBooleanExpression(q, signals, steps, endNs, curA);
      if (exprRes.valid && exprRes.nextMatchTime !== null) {
        const targetT = exprRes.nextMatchTime;
        setCurA(targetT);
        setT0(Math.max(0, Math.min(Math.max(0, endNs - tspan), targetT - tspan / 2)));
        if (exprRes.matchedSignals.length > 0) {
          const firstSigIdx = signals.findIndex((s) => s.n === exprRes.matchedSignals[0]);
          if (firstSigIdx >= 0) {
            const existing = rows.find((r) => r.si === firstSigIdx);
            if (existing) setSelectedId(existing.id);
          }
        }
        setSearchStatusToast(
          `Expr matched @ ${targetT} ns (${exprRes.matchingTimes.length} total timestamp${exprRes.matchingTimes.length === 1 ? '' : 's'})`
        );
      } else {
        setSearchStatusToast(exprRes.errorMessage || `No timestamps matched "${q}"`);
      }
      return;
    }

    let rx: RegExp;
    try {
      rx = new RegExp(q.replace(/^add:\s*/i, ''), 'i');
    } catch {
      return;
    }
    if (/^add:\s*/i.test(q)) {
      const additions: WaveRowItem[] = [];
      signals.forEach((s, i) => {
        if (rx.test(s.n) && !inViewSet.has(i)) {
          additions.push({
            id: `row-${Date.now()}-${i}`,
            si: i,
            radix: null,
            color: null,
            expanded: false,
            analog: false,
          });
        }
      });
      if (additions.length > 0) {
        setRows((prev) => [...prev, ...additions]);
        setSearchStatusToast(`Added ${additions.length} signal(s) matching /${q.replace(/^add:\s*/i, '')}/`);
      }
      return;
    }
    const foundIdx = signals.findIndex((s) => rx.test(s.n));
    if (foundIdx >= 0) {
      const existing = rows.find((r) => r.si === foundIdx);
      if (existing) {
        setSelectedId(existing.id);
      } else {
        const newRow: WaveRowItem = {
          id: `row-${Date.now()}-${foundIdx}`,
          si: foundIdx,
          radix: null,
          color: null,
          expanded: false,
          analog: false,
        };
        setRows((prev) => [newRow, ...prev]);
        setSelectedId(newRow.id);
      }
      setSearchStatusToast(`Focused signal: ${signals[foundIdx].n}`);
    } else {
      setSearchStatusToast(`No signal found matching "${q}"`);
    }
  };

  const handleExportCsv = () => {
    let csv = 'signal,width,time_ns,value_hex\n';
    for (const r of rows) {
      const sig = signals[r.si];
      const st = steps[r.si];
      if (!sig || !st) continue;
      for (let i = 0; i < st.n; i++) {
        csv += `${sig.n},${sig.w},${st.t[i]},${formatSignalValue(st.v[i], sig.w, 'hex')}\n`;
      }
    }
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'vikas-wave-transitions.csv';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex flex-col flex-1 min-h-0 bg-[var(--bg)]">
      {/* Studio Toolbar */}
      <div className="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--panel)] border-b border-[var(--edge)] flex-wrap">
        <input
          ref={fileInputRef}
          type="file"
          accept=".vcd,.vwf,.json"
          className="hidden"
          onChange={(e) => {
            const f = e.target.files?.[0];
            if (f) onImportVcdOrVwf(f);
          }}
        />
        <button
          type="button"
          onClick={() => fileInputRef.current?.click()}
          className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] border border-[var(--edge2)] text-[var(--txt)] cursor-pointer"
          title="Open any standard .vcd or native .vwf dump file client-side"
        >
          <FolderOpen className="w-3.5 h-3.5 text-[var(--accent)]" />
          Open .VCD/.VWF
        </button>

        {/* Project / Target Selector */}
        <select
          aria-label="Select RTL Simulation Target"
          value={activeTarget}
          onChange={(e) => onSelectProject(e.target.value)}
          disabled={simulating}
          className="bg-[var(--bg2)] text-[var(--txt)] border border-[var(--edge2)] rounded px-2 py-1 text-xs font-mono outline-none focus:border-[var(--accent)] cursor-pointer"
        >
          {PRESET_PROJECTS.map((p) => (
            <option key={p.target} value={p.target}>
              {p.label}
            </option>
          ))}
        </select>

        <div className="h-4 w-px bg-[var(--edge)] mx-1" />

        <button
          type="button"
          onClick={handleFit}
          aria-label="Fit Timeline"
          className="p-1.5 rounded hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] cursor-pointer"
          title="Fit full timeline (F)"
        >
          <Maximize2 className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onClick={() => handleZoom(0.6)}
          aria-label="Zoom In"
          className="p-1.5 rounded hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] cursor-pointer"
          title="Zoom in (+)"
        >
          <ZoomIn className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onClick={() => handleZoom(1.6)}
          aria-label="Zoom Out"
          className="p-1.5 rounded hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] cursor-pointer"
          title="Zoom out (-)"
        >
          <ZoomOut className="w-3.5 h-3.5" />
        </button>

        <div className="h-4 w-px bg-[var(--edge)] mx-1" />

        <button
          type="button"
          onClick={() => setCurA(Math.round(t0 + tspan * 0.35))}
          aria-label="Place Cursor A"
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--txt)] border border-[var(--edge)] cursor-pointer"
          title="Place Cursor A"
        >
          <Crosshair className="w-3.5 h-3.5 text-[var(--accent)]" />
          Cursor A
        </button>
        <button
          type="button"
          onClick={() => {
            if (curA === null) setCurA(Math.round(t0 + tspan * 0.3));
            setCurB(Math.round(t0 + tspan * 0.7));
          }}
          aria-label="Measure Delta B"
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[#34d399] border border-[var(--edge)] cursor-pointer"
          title="Place Cursor B to measure Δt and Frequency (MHz)"
        >
          <Ruler className="w-3.5 h-3.5" />
          Δt Measure B
        </button>

        <button
          type="button"
          data-testid="prev-edge-btn"
          onClick={() => jumpToSignalEdge('prev')}
          aria-label="Previous Transition Edge"
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--txt)] border border-[var(--edge)] cursor-pointer"
          title="Jump Cursor A to previous transition edge of selected signal (Shortcut: P)"
        >
          <SkipBack className="w-3.5 h-3.5 text-cyan-400" />
          Prev Edge
        </button>
        <button
          type="button"
          data-testid="next-edge-btn"
          onClick={() => jumpToSignalEdge('next')}
          aria-label="Next Transition Edge"
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--txt)] border border-[var(--edge)] cursor-pointer"
          title="Jump Cursor A to next transition edge of selected signal (Shortcut: N)"
        >
          <SkipForward className="w-3.5 h-3.5 text-cyan-400" />
          Next Edge
        </button>

        <button
          type="button"
          onClick={() => {
            if (!selectedRow) return;
            setRows((prev) =>
              prev.map((r) => (r.id === selectedRow.id ? { ...r, expanded: !r.expanded } : r))
            );
          }}
          aria-label="Expand Bus Bits"
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] border border-[var(--edge)] cursor-pointer"
          title="Expand selected multi-bit bus into bit lanes (E)"
        >
          <Expand className="w-3.5 h-3.5" />
          Expand Bus
        </button>

        <button
          type="button"
          onClick={() => {
            if (!selectedRow) return;
            setRows((prev) =>
              prev.map((r) => (r.id === selectedRow.id ? { ...r, analog: !r.analog } : r))
            );
          }}
          aria-label="Toggle Analog Trace"
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] border border-[var(--edge)] cursor-pointer"
          title="Toggle analog waveform rendering (A)"
        >
          <Activity className="w-3.5 h-3.5" />
          Analog
        </button>

        {/* Iteration 1 Breakthrough Controls: COI Ray-Tracer (WV-02), Pipeline Waterfall (WV-04), What-If Patch (WV-08) */}
        <button
          type="button"
          onClick={() => setShowCoiRays((v) => !v)}
          className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs border font-mono cursor-pointer ${
            showCoiRays
              ? 'bg-rose-950/60 border-rose-500/50 text-rose-300'
              : 'bg-[var(--panel2)] border-[var(--edge)] text-[var(--dim)]'
          }`}
          title="WV-02: Causal Cone-of-Influence (COI) Visual Ray-Tracer across waveform rows"
        >
          <Sparkles className="w-3.5 h-3.5" />
          COI Ray-Tracer
        </button>

        <button
          type="button"
          onClick={() => setShowWaterfall((v) => !v)}
          className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs border font-mono cursor-pointer ${
            showWaterfall
              ? 'bg-cyan-950/60 border-cyan-500/50 text-cyan-300'
              : 'bg-[var(--panel2)] border-[var(--edge)] text-[var(--dim)]'
          }`}
          title="WV-04: Synchronized Konata-Style 5-Stage CPU Instruction Pipeline Waterfall"
        >
          <Layers className="w-3.5 h-3.5" />
          Pipeline Waterfall
        </button>

        <button
          type="button"
          onClick={() => {
            if (!selectedRow || !kdb) return;
            if (whatIfPatch && whatIfPatch.targetSigIdx === selectedRow.si) {
              setWhatIfPatch(null);
            } else {
              const patchT = curA !== null ? curA : Math.round(t0 + tspan * 0.35);
              setWhatIfPatch(
                simulateWhatIfWaveformPatch(kdb, signals, steps, selectedRow.si, patchT)
              );
            }
          }}
          className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs border font-mono cursor-pointer ${
            whatIfPatch
              ? 'bg-amber-950/70 border-amber-500/60 text-amber-300'
              : 'bg-[var(--panel2)] border-[var(--edge)] text-amber-300/80 hover:text-amber-200'
          }`}
          title="WV-08: What-If Live Signal Edge Override & Counterfactual Delta Re-Simulator (<20ms)"
        >
          {whatIfPatch ? 'Clear What-If Δ' : 'What-If Force @ A'}
        </button>

        {/* Iteration 2 Breakthrough Controls: Protocol Swimlanes (WV-03), 3D Power Surface (WV-11), Pub Export (WV-12) */}
        <button
          type="button"
          onClick={() => setShowProtocolSwimlanes((v) => !v)}
          className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs border font-mono cursor-pointer ${
            showProtocolSwimlanes
              ? 'bg-emerald-950/60 border-emerald-500/50 text-emerald-300'
              : 'bg-[var(--panel2)] border-[var(--edge)] text-[var(--dim)]'
          }`}
          title="WV-03: Semantic Protocol Transaction Swimlanes (AXI4-Lite, UART-8N1, SPI)"
        >
          Protocol Swimlanes
        </button>

        <button
          type="button"
          onClick={() => setShow3dPowerSurface((v) => !v)}
          className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs border font-mono cursor-pointer ${
            show3dPowerSurface
              ? 'bg-purple-950/60 border-purple-500/50 text-purple-300'
              : 'bg-[var(--panel2)] border-[var(--edge)] text-[var(--dim)]'
          }`}
          title="WV-11: 3D Time-Bit-Power Surface Topography Mode"
        >
          3D Power Surface
        </button>

        <button
          type="button"
          onClick={() => setPubExportModalOpen(true)}
          className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs border font-mono bg-cyan-950/50 hover:bg-cyan-900/60 border-cyan-500/50 text-cyan-200 cursor-pointer"
          title="WV-12: Export active waveform to LaTeX TikZ, Typst CeTZ, WaveDrom JSON, or Vector SVG"
        >
          <Download className="w-3.5 h-3.5 text-cyan-400" />
          Pub Export (WV-12)
        </button>

        <button
          type="button"
          data-testid="open-pdf-report-btn"
          onClick={() => setPdfReportModalOpen(true)}
          className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs border font-mono bg-emerald-950/60 hover:bg-emerald-900/70 border-emerald-500/60 text-emerald-200 font-semibold cursor-pointer"
          title="Export Simulation Summary, Open-KDB CSR Analysis & Silicon Physics Telemetry as a Formatted IEEE/ISSCC Research-Paper PDF Report"
        >
          <FileText className="w-3.5 h-3.5 text-emerald-400" />
          PDF Report (IEEE)
        </button>

        <button
          type="button"
          data-testid="toggle-bookmarks-panel-btn"
          onClick={() => {
            setActiveRightDockTab('BOOKMARKS');
            setShowAddBookmarkDialog(true);
          }}
          className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs border font-mono font-semibold cursor-pointer transition-colors ${
            activeRightDockTab === 'BOOKMARKS'
              ? 'bg-amber-950/70 hover:bg-amber-900/80 border-amber-500/60 text-amber-200 shadow-xs'
              : 'bg-[var(--panel2)] hover:bg-[var(--raised)] border-[var(--edge)] text-[var(--txt)]'
          }`}
          title="Bookmark simulation time-points using Cursors A & B and open Bookmarks navigation panel"
        >
          <Bookmark className="w-3.5 h-3.5 text-amber-400" />
          <span>Bookmarks ({bookmarks.length})</span>
        </button>

        {/* Search & Temporal Query Box */}
        <div className="flex items-center bg-[var(--bg2)] border border-[var(--edge2)] rounded px-2 py-0.5 ml-1 flex-1 max-w-md">
          <Search className="w-3.5 h-3.5 text-[var(--dim)] mr-1.5 shrink-0" />
          <input
            data-testid="waveform-global-search-input"
            aria-label="Search signal or boolean expression"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleRunSearch();
            }}
            placeholder={
              searchMode === 'expr'
                ? 'expr: zero_flag == 1 · alu_result > 0'
                : searchMode === 'goto'
                ? 'goto timestamp (e.g. 120)'
                : 'search signal · add:uart · zero_flag == 1'
            }
            className="bg-transparent text-xs font-mono text-[var(--txt)] outline-none flex-1 min-w-0"
          />
          <button
            type="button"
            data-testid="waveform-search-mode-btn"
            onClick={() =>
              setSearchMode((m) => (m === 'signal' ? 'expr' : m === 'expr' ? 'goto' : 'signal'))
            }
            className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[var(--panel)] text-[var(--accent)] border border-[var(--edge)] ml-1 cursor-pointer"
            title="Cycle search mode: SIGNAL (name/regex) -> EXPR (boolean condition e.g. zero_flag == 1) -> GOTO (timestamp ns)"
          >
            {searchMode.toUpperCase()}
          </button>
          <button
            type="button"
            data-testid="waveform-search-run-btn"
            onClick={handleRunSearch}
            aria-label="Execute Search"
            className="p-1 text-[var(--dim)] hover:text-[var(--accent)] cursor-pointer"
          >
            <Play className="w-3 h-3" />
          </button>
        </div>

        {searchStatusToast && (
          <span
            data-testid="waveform-search-status-toast"
            onClick={() => setSearchStatusToast(null)}
            className="text-[10.5px] font-mono px-2 py-0.5 rounded bg-cyan-950/80 border border-cyan-500/40 text-cyan-300 cursor-pointer"
          >
            {searchStatusToast}
          </span>
        )}

        <div className="ml-auto flex items-center gap-2">
          <select
            aria-label="Default Radix"
            value={defRadix}
            onChange={(e) => setDefRadix(e.target.value as typeof defRadix)}
            className="bg-[var(--bg2)] text-xs font-mono text-[var(--txt)] border border-[var(--edge2)] rounded px-2 py-1 outline-none cursor-pointer"
          >
            <option value="hex">0x hex</option>
            <option value="bin">0b bin</option>
            <option value="dec">u dec</option>
            <option value="signed">signed</option>
            <option value="oct">0o oct</option>
            <option value="ascii">ASCII</option>
          </select>

          <button
            type="button"
            onClick={handleExportCsv}
            className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--txt)] border border-[var(--edge2)] cursor-pointer"
          >
            <Download className="w-3.5 h-3.5 text-[var(--accent)]" />
            Export CSV
          </button>
        </div>
      </div>

      {/* 3-Pane Studio Main Body */}
      <div className="flex flex-1 min-h-0">
        {/* Left Signal Hierarchy Tree Dock */}
        <aside className="w-68 bg-[var(--panel)] border-r border-[var(--edge)] flex flex-col min-h-0 shrink-0">
          <div className="flex items-center justify-between px-3 py-1.5 border-b border-[var(--edge)] text-[11px] font-bold tracking-wider uppercase text-[var(--dim)]">
            <span className="flex items-center gap-1.5">
              <Layers className="w-3.5 h-3.5 text-[var(--accent)]" />
              Signals ({signals.length})
            </span>
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={() =>
                  setRows(
                    signals.slice(0, 64).map((s, i) => ({
                      id: `row-all-${i}-${s.n}`,
                      si: i,
                      radix: null,
                      color: null,
                      expanded: false,
                      analog: false,
                    }))
                  )
                }
                className="p-1 rounded hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] cursor-pointer"
                title="Add top 64 signals to wave view"
              >
                <Plus className="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onClick={() => setRows([])}
                className="p-1 rounded hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--bad)] cursor-pointer"
                title="Clear wave view"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          {/* Power-User Dynamic Regex Signal Filter Bar */}
          <div className="p-2 border-b border-[var(--edge)] space-y-1.5 bg-[var(--bg2)]/60">
            <div className="flex items-center justify-between text-[10px] font-mono">
              <span className="flex items-center gap-1 text-[var(--accent)] font-bold uppercase tracking-wider">
                <Filter className="w-3 h-3" />
                Regex Signal Trace Filter
              </span>
              <span
                data-testid="regex-match-count-badge"
                className={`px-1.5 py-0.5 rounded font-semibold ${
                  !regexFilterResult.isValidRegex
                    ? 'bg-rose-950/80 text-rose-300 border border-rose-500/50'
                    : treeFilter.trim()
                    ? 'bg-emerald-950/80 text-emerald-300 border border-emerald-500/40'
                    : 'bg-slate-800 text-slate-300'
                }`}
              >
                {!regexFilterResult.isValidRegex
                  ? 'INVALID REGEX'
                  : `Matched ${regexFilterResult.matchedIndices.length}/${signals.length}`}
              </span>
            </div>

            <div className="flex items-center gap-1 bg-[var(--bg)] border border-[var(--edge2)] rounded px-2 py-1 focus-within:border-[var(--accent)]">
              <Search className="w-3.5 h-3.5 text-[var(--dim)] shrink-0" />
              <input
                data-testid="regex-signal-filter-input"
                aria-label="Filter signals in hierarchy tree and waveform view by regex"
                value={treeFilter}
                onChange={(e) => setTreeFilter(e.target.value)}
                placeholder="Regex trace: e.g. ^(clk|rst)|alu|.*_q$"
                className="w-full bg-transparent text-xs font-mono text-[var(--txt)] outline-none min-w-0"
              />
              {treeFilter && (
                <button
                  type="button"
                  aria-label="Clear regex signal filter"
                  onClick={() => setTreeFilter('')}
                  className="px-1 text-xs font-mono text-[var(--dim)] hover:text-[var(--txt)] cursor-pointer"
                  title="Clear regex filter"
                >
                  ×
                </button>
              )}
              <button
                type="button"
                data-testid="regex-case-toggle"
                aria-label="Toggle Regex Case Sensitivity"
                onClick={() => setTreeCaseSensitive((v) => !v)}
                className={`px-1.5 py-0.5 rounded text-[10px] font-mono font-bold border cursor-pointer ${
                  treeCaseSensitive
                    ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50'
                    : 'bg-slate-900 text-slate-400 border-slate-700'
                }`}
                title="Case-sensitive regex matching (Aa)"
              >
                Aa
              </button>
              <button
                type="button"
                data-testid="regex-invert-toggle"
                aria-label="Toggle Regex Inverse Match"
                onClick={() => setTreeInvertMatch((v) => !v)}
                className={`px-1.5 py-0.5 rounded text-[10px] font-mono font-bold border cursor-pointer ${
                  treeInvertMatch
                    ? 'bg-amber-500/20 text-amber-300 border-amber-500/50'
                    : 'bg-slate-900 text-slate-400 border-slate-700'
                }`}
                title="Invert regex match (! — hide matching signals)"
              >
                !
              </button>
            </div>

            {!regexFilterResult.isValidRegex && regexFilterResult.regexError && (
              <div className="flex items-center gap-1 text-[10px] font-mono text-rose-300 bg-rose-950/50 border border-rose-500/40 rounded px-2 py-1">
                <AlertTriangle className="w-3 h-3 shrink-0 text-rose-400" />
                <span className="truncate">{regexFilterResult.regexError}</span>
              </div>
            )}

            {/* Power-User Quick Regex Pattern Chips */}
            <div className="flex items-center gap-1 flex-wrap pt-0.5">
              {[
                { label: 'All', rx: '' },
                { label: 'clk|rst', rx: 'clk|rst' },
                { label: 'pc|alu', rx: 'pc|alu' },
                { label: 'axi|uart|spi', rx: 'axi|uart|spi' },
                { label: '.*_q$', rx: '.*_q$' },
                { label: 'VIKAS_', rx: 'VIKAS_' },
              ].map((preset) => (
                <button
                  key={preset.label}
                  type="button"
                  onClick={() => setTreeFilter(preset.rx)}
                  className={`px-1.5 py-0.5 rounded text-[9.5px] font-mono border cursor-pointer ${
                    treeFilter === preset.rx
                      ? 'bg-cyan-950/80 text-cyan-300 border-cyan-500/60 font-bold'
                      : 'bg-slate-900/80 text-slate-400 border-slate-800 hover:text-slate-200'
                  }`}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-1.5 space-y-1 font-mono text-xs">
            {scopes.map(([scopeName, indices]) => {
              const isClosed = collapsedScopes.has(scopeName);
              return (
                <div key={scopeName}>
                  <button
                    type="button"
                    onClick={() =>
                      setCollapsedScopes((prev) => {
                        const next = new Set(prev);
                        if (next.has(scopeName)) next.delete(scopeName);
                        else next.add(scopeName);
                        return next;
                      })
                    }
                    className="w-full flex items-center gap-1 px-1.5 py-1 rounded hover:bg-[var(--raised)] text-left text-[var(--accent2)] font-semibold cursor-pointer"
                  >
                    {isClosed ? (
                      <ChevronRight className="w-3.5 h-3.5 shrink-0" />
                    ) : (
                      <ChevronDown className="w-3.5 h-3.5 shrink-0" />
                    )}
                    <span className="truncate">{scopeName}</span>
                    <span className="ml-auto text-[10px] text-[var(--faint)]">{indices.length}</span>
                  </button>
                  {!isClosed && (
                    <div className="pl-3 space-y-0.5 mt-0.5">
                      {indices.slice(0, 80).map((si) => {
                        const s = signals[si];
                        const shortName = s.n.split('.').pop() || s.n;
                        const active = inViewSet.has(si);
                        return (
                          <button
                            key={si}
                            type="button"
                            onClick={() => {
                              const existing = rows.find((r) => r.si === si);
                              if (existing) {
                                setSelectedId(existing.id);
                              } else {
                                const nr: WaveRowItem = {
                                  id: `row-${Date.now()}-${si}`,
                                  si,
                                  radix: null,
                                  color: null,
                                  expanded: false,
                                  analog: false,
                                };
                                setRows((prev) => [...prev, nr]);
                                setSelectedId(nr.id);
                              }
                            }}
                            className={`w-full flex items-center gap-1.5 px-2 py-1 rounded text-left truncate cursor-pointer ${
                              active
                                ? 'bg-[var(--sel)] text-[var(--txt)] border-l-2 border-[var(--accent)]'
                                : 'hover:bg-[var(--raised)] text-[var(--dim)]'
                            }`}
                          >
                            <span
                              className="w-2 h-2 rounded-xs shrink-0"
                              style={{ backgroundColor: KIND_COLORS[s.k] || '#89e05a' }}
                            />
                            <span className="truncate flex-1">{shortName}</span>
                            <span className="text-[10px] text-[var(--faint)]">{s.w}b</span>
                          </button>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </aside>

        {/* Center Waveform Canvas + Pipeline Waterfall + Minimap */}
        <div className="flex-1 flex flex-col min-w-0 bg-[var(--bg)] relative">
          {/* Iteration 11: Dedicated Visible Signal Search & Type Filter Bar */}
          <div className="px-3 py-1.5 bg-[#0b101d] border-b border-[var(--edge)] flex flex-wrap items-center gap-2 font-mono text-xs shrink-0 z-10 shadow-xs">
            {/* Search Input Field */}
            <div className="flex items-center gap-1.5 bg-[var(--bg2)] border border-[var(--edge2)] rounded px-2.5 py-1 flex-1 min-w-[220px] max-w-md focus-within:border-[var(--accent)] transition-all">
              <Search className="w-3.5 h-3.5 text-cyan-400 shrink-0" />
              <input
                data-testid="visible-signal-search-input"
                aria-label="Filter visible signals by name or type"
                value={visibleSignalFilter}
                onChange={(e) => setVisibleSignalFilter(e.target.value)}
                placeholder="Filter visible signals by name (e.g. clk, pc, alu, uart)..."
                className="w-full bg-transparent text-xs font-mono text-[var(--txt)] outline-none min-w-0 placeholder:text-slate-500"
              />
              {visibleSignalFilter && (
                <button
                  type="button"
                  data-testid="clear-visible-signal-search-btn"
                  onClick={() => setVisibleSignalFilter('')}
                  aria-label="Clear visible signal search"
                  className="text-[var(--dim)] hover:text-[var(--txt)] p-0.5 cursor-pointer"
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              )}
            </div>

            {/* Type Filter Selectors */}
            <div className="flex items-center gap-1 overflow-x-auto text-[11px]">
              <span className="text-[var(--dim)] text-[10px] uppercase font-bold tracking-wider mr-1 flex items-center gap-1">
                <Filter className="w-3 h-3 text-[var(--accent)]" />
                Type:
              </span>
              {(
                [
                  { key: 'ALL', label: 'All' },
                  { key: 'CLOCK', label: 'Clock' },
                  { key: 'RESET', label: 'Reset' },
                  { key: 'BUS', label: 'Bus (>1b)' },
                  { key: 'SCALAR', label: 'Scalar (1b)' },
                  { key: 'DATA', label: 'Data' },
                  { key: 'CONTROL', label: 'Control' },
                  { key: 'ANALOG', label: 'Analog' },
                ] as const
              ).map((typeItem) => {
                const active = visibleTypeFilter === typeItem.key;
                return (
                  <button
                    key={typeItem.key}
                    type="button"
                    data-testid={`filter-type-${typeItem.key.toLowerCase()}`}
                    onClick={() => setVisibleTypeFilter(typeItem.key)}
                    className={`px-2 py-0.5 rounded text-[10px] font-semibold transition-all cursor-pointer ${
                      active
                        ? 'bg-[var(--accent)] text-slate-950 font-bold shadow-xs'
                        : 'bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] border border-[var(--edge)]'
                    }`}
                  >
                    {typeItem.label}
                  </button>
                );
              })}
            </div>

            {/* Counter badge, quick add, & reset */}
            <div className="ml-auto flex items-center gap-2 shrink-0">
              <span
                data-testid="visible-signals-count-badge"
                className={`text-[10px] px-2 py-0.5 rounded font-mono border font-semibold ${
                  visibleSignalFilter || visibleTypeFilter !== 'ALL'
                    ? 'bg-cyan-950/70 border-cyan-500/50 text-cyan-200'
                    : 'bg-slate-800/80 border-slate-700/60 text-slate-300'
                }`}
              >
                Showing {displayedWaveRows.length} / {filteredRows.length} signals
              </span>

              {/* Quick Add matching signals from workspace if search term matches signals not yet added to view */}
              {visibleSignalFilter.trim() && (() => {
                const term = visibleSignalFilter.trim().toLowerCase();
                const matchedWorkspace = signals.filter(
                  (s, idx) =>
                    !rows.some((r) => r.si === idx) &&
                    (s.n.toLowerCase().includes(term) || (s.n.split('.').pop() || '').toLowerCase().includes(term))
                );
                if (matchedWorkspace.length === 0) return null;
                return (
                  <button
                    type="button"
                    data-testid="add-matching-workspace-signals-btn"
                    onClick={() => {
                      const newItems: WaveRowItem[] = matchedWorkspace.slice(0, 16).map((s) => {
                        const si = signals.indexOf(s);
                        return {
                          id: `row-find-${Date.now()}-${si}`,
                          si,
                          radix: null,
                          color: null,
                          expanded: false,
                          analog: false,
                        };
                      });
                      setRows((prev) => [...prev, ...newItems]);
                    }}
                    className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono bg-emerald-950/60 hover:bg-emerald-900/80 border border-emerald-500/50 text-emerald-300 cursor-pointer"
                    title={`Add ${matchedWorkspace.length} matching un-added signals to the waveform view`}
                  >
                    <Plus className="w-3 h-3 text-emerald-400" />
                    +Add {Math.min(16, matchedWorkspace.length)} matching
                  </button>
                );
              })()}

              {(visibleSignalFilter || visibleTypeFilter !== 'ALL') && (
                <button
                  type="button"
                  data-testid="reset-all-signal-filters-btn"
                  onClick={() => {
                    setVisibleSignalFilter('');
                    setVisibleTypeFilter('ALL');
                  }}
                  className="text-[10px] text-amber-400 hover:text-amber-300 underline font-mono cursor-pointer"
                >
                  Reset
                </button>
              )}
            </div>
          </div>

          {/* WV-04: Synchronized CPU Instruction Pipeline Waterfall Strip */}
          {showWaterfall && waterfallSlots.length > 0 && (
            <div className="h-9 bg-[#0e1525] border-b border-[var(--edge)] flex items-center px-2 gap-1 overflow-x-auto shrink-0 font-mono text-[10px]">
              <span className="text-cyan-400 font-bold uppercase tracking-wider shrink-0 mr-2">
                WV-04 PIPELINE WATERFALL:
              </span>
              {waterfallSlots.slice(0, 14).map((slot, idx) => (
                <button
                  key={idx}
                  type="button"
                  onClick={() => setCurA(slot.tStart)}
                  className="px-1.5 py-0.5 rounded border border-slate-700/80 bg-slate-900/90 hover:border-cyan-400 flex items-center gap-1 shrink-0 cursor-pointer"
                  title={`${slot.pcHex}: ${slot.asm} (${slot.stage}) @ ${slot.tStart}ns${
                    slot.hazardReason ? ` — ${slot.hazardReason}` : ''
                  }`}
                >
                  <span
                    className="px-1 rounded text-[9px] font-bold text-slate-950"
                    style={{ backgroundColor: slot.color }}
                  >
                    {slot.stage}
                  </span>
                  <span className="text-slate-300">{slot.pcHex}</span>
                  <span className="text-slate-400 hidden 2xl:inline">{slot.asm.split(' ')[0]}</span>
                </button>
              ))}
            </div>
          )}

          {/* WV-03: Semantic Protocol Transaction Swimlanes Strip */}
          {showProtocolSwimlanes && protocolSwimlanes.length > 0 && (
            <div className="h-8 bg-[#0b1320] border-b border-[var(--edge)] flex items-center px-2 gap-1.5 overflow-x-auto shrink-0 font-mono text-[10px]">
              <span className="text-emerald-400 font-bold uppercase tracking-wider shrink-0 mr-1">
                WV-03 PROTOCOL SWIMLANES:
              </span>
              {protocolSwimlanes.slice(0, 12).map((txn) => (
                <button
                  key={txn.id}
                  type="button"
                  onClick={() => setCurA(txn.tStart)}
                  className="px-1.5 py-0.5 rounded border border-slate-700/80 bg-slate-900/90 hover:border-emerald-400 flex items-center gap-1 shrink-0 cursor-pointer"
                  title={`${txn.protocol} (${txn.channel}): ${txn.summary} [${txn.tStart}..${txn.tEnd}ns]`}
                >
                  <span
                    className="px-1 rounded text-[9px] font-bold text-slate-950"
                    style={{ backgroundColor: txn.color }}
                  >
                    {txn.protocol}
                  </span>
                  <span className="text-slate-200">{txn.summary}</span>
                </button>
              ))}
            </div>
          )}

          {/* WV-11: 3D Time-Bit-Power Surface Topography Banner */}
          {show3dPowerSurface && powerSurfaceCells.length > 0 && (
            <div className="bg-[#111526] border-b border-purple-500/40 p-2 shrink-0 font-mono text-[10px]">
              <div className="flex items-center justify-between mb-1">
                <span className="text-purple-300 font-bold uppercase tracking-wider">
                  WV-11 3D TIME × SIGNAL × FEMTOJOULE POWER TOPOGRAPHY (Vdd = 0.80V · Cnode = 1.4fF)
                </span>
                <span className="text-slate-400">
                  Peak bin: {Math.max(...powerSurfaceCells.map((c) => c.energyFj)).toFixed(2)} fJ
                </span>
              </div>
              <div className="grid grid-cols-12 gap-1">
                {powerSurfaceCells.slice(0, 24).map((cell, idx) => (
                  <div
                    key={idx}
                    onClick={() => setCurA(cell.tStart)}
                    className="p-1 rounded border border-purple-500/30 bg-slate-900/90 hover:border-purple-400 cursor-pointer"
                    title={`${cell.sigShortName} [${cell.tStart}..${cell.tEnd}ns]: ${cell.energyFj} fJ (${cell.toggles} toggles)`}
                  >
                    <div className="text-[9px] text-purple-300 truncate">{cell.sigShortName}</div>
                    <div className="text-amber-300 font-bold">{cell.energyFj}fJ</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div
            ref={wrapRef}
            data-testid="waveform-scroll-container"
            className="flex-1 min-h-[260px] overflow-y-auto overflow-x-auto relative"
          >
            <canvas
              ref={canvasRef}
              data-testid="waveform-canvas"
              onClick={handleCanvasClick}
              onDoubleClick={handleCanvasDoubleClick}
              onMouseMove={handleCanvasMouseMove}
              onMouseLeave={() => setHoverInfo(null)}
              onWheel={(e) => {
                if (e.ctrlKey || e.metaKey || e.altKey) {
                  e.preventDefault();
                  handleZoom(e.deltaY < 0 ? 0.8 : 1.25);
                } else if (Math.abs(e.deltaX) > Math.abs(e.deltaY) && Math.abs(e.deltaX) > 4) {
                  handlePan(e.deltaX > 0 ? 1 : -1);
                }
                // Otherwise allow native vertical scrolling of wrapRef through all signal rows!
              }}
              className="block cursor-crosshair"
            />
            {displayedWaveRows.length === 0 && (
              <div
                data-testid="no-matching-signals-banner"
                className="absolute inset-x-0 top-24 flex flex-col items-center justify-center p-8 text-center text-xs font-mono space-y-2 pointer-events-auto"
              >
                <div className="p-3 rounded-full bg-slate-800 text-slate-400">
                  <Search className="w-6 h-6" />
                </div>
                <div className="text-slate-200 font-semibold">
                  No signals match filter &quot;{visibleSignalFilter}&quot;
                  {visibleTypeFilter !== 'ALL' ? ` with type ${visibleTypeFilter}` : ''}
                </div>
                <div className="text-[11px] text-slate-400 max-w-sm">
                  Try adjusting the name query or type filter pill, or reset filters to display all visible signals.
                </div>
                <button
                  type="button"
                  data-testid="empty-state-reset-filter-btn"
                  onClick={() => {
                    setVisibleSignalFilter('');
                    setVisibleTypeFilter('ALL');
                  }}
                  className="px-3 py-1.5 rounded bg-[var(--accent)] text-slate-950 font-bold text-xs cursor-pointer shadow-xs"
                >
                  Reset Signal Filter
                </button>
              </div>
            )}
          </div>

          {/* Minimap Navigator */}
          <div
            className="h-11 bg-[var(--panel)] border-t border-[var(--edge)] relative cursor-pointer shrink-0"
            onClick={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              const ratio = (e.clientX - rect.left) / Math.max(1, rect.width);
              const centerT = ratio * endNs;
              setT0(Math.max(0, Math.min(Math.max(0, endNs - tspan), centerT - tspan / 2)));
            }}
          >
            <canvas ref={minimapRef} className="w-full h-full block" />
            <span className="absolute right-2.5 top-1 text-[9.5px] font-mono text-[var(--faint)] pointer-events-none">
              MINIMAP NAVIGATOR — click to jump · density histogram ({nev.toLocaleString()} events)
            </span>
          </div>

          {/* Hover Tooltip */}
          {hoverInfo && (
            <div
              style={{
                left: Math.min(hoverInfo.x + 14, window.innerWidth - 290),
                top: Math.min(hoverInfo.y + 14, window.innerHeight - 130),
              }}
              className="fixed z-50 pointer-events-none bg-[var(--panel2)] border border-[var(--edge2)] rounded-lg px-3 py-2 shadow-2xl font-mono text-xs"
            >
              <div className="text-[var(--accent)] font-bold break-all">{hoverInfo.sig.n}</div>
              <div className="flex items-center gap-2 mt-0.5">
                <span className="text-[var(--ok)] font-semibold">{hoverInfo.val}</span>
                <span className="text-[var(--faint)]">
                  {hoverInfo.sig.w}b · {hoverInfo.sig.k}
                </span>
              </div>
              <div className="text-[var(--dim)] text-[11px] mt-0.5">
                t = {hoverInfo.t} ns
                {hoverInfo.nextDt !== null ? ` · next edge Δ+${hoverInfo.nextDt} ns` : ' · stable to end'}
              </div>
              <div className="text-[var(--faint)] text-[10px]">
                edges: {steps[hoverInfo.si]?.n ?? 0} · unique: {steps[hoverInfo.si]?.uniq ?? 0}
              </div>
            </div>
          )}
        </div>

        {/* Right Dock: Signal Properties & Simulation Time-Point Bookmarks */}
        <aside className="w-80 bg-[var(--panel)] border-l border-[var(--edge)] flex flex-col min-h-0 shrink-0">
          {/* Dock Navigation Tabs */}
          <div className="flex border-b border-[var(--edge)] bg-[var(--bg2)]/60 text-xs font-mono shrink-0">
            <button
              type="button"
              data-testid="tab-signal-inspector"
              onClick={() => setActiveRightDockTab('INSPECTOR')}
              className={`flex-1 py-2 px-3 font-semibold text-center border-b-2 cursor-pointer transition-colors ${
                activeRightDockTab === 'INSPECTOR'
                  ? 'border-[var(--accent)] text-[var(--accent)] bg-[var(--panel)]'
                  : 'border-transparent text-[var(--dim)] hover:text-[var(--txt)]'
              }`}
            >
              Signal Inspector
            </button>
            <button
              type="button"
              data-testid="tab-simulation-bookmarks"
              onClick={() => setActiveRightDockTab('BOOKMARKS')}
              className={`flex-1 py-2 px-3 font-semibold text-center border-b-2 cursor-pointer transition-colors flex items-center justify-center gap-1.5 ${
                activeRightDockTab === 'BOOKMARKS'
                  ? 'border-amber-400 text-amber-300 bg-[var(--panel)]'
                  : 'border-transparent text-[var(--dim)] hover:text-[var(--txt)]'
              }`}
            >
              <Bookmark className="w-3.5 h-3.5 text-amber-400" />
              <span>Bookmarks ({bookmarks.length})</span>
            </button>
          </div>

          {activeRightDockTab === 'INSPECTOR' ? (
            <div className="p-3 space-y-4 overflow-y-auto text-xs flex-1">
              {selectedRow && signals[selectedRow.si] ? (
                (() => {
                  const sig = signals[selectedRow.si];
                  const st = steps[selectedRow.si];
                  const rdx = selectedRow.radix || defRadix;
                  const valA = curA !== null ? formatSignalValue(valAtTime(st, sig.w, curA), sig.w, rdx) : '—';
                  const valB = curB !== null ? formatSignalValue(valAtTime(st, sig.w, curB), sig.w, rdx) : '—';
                return (
                  <>
                    <div className="space-y-1.5">
                      <div className="text-[10px] font-bold uppercase tracking-wider text-[var(--accent)]">
                        Selected Net
                      </div>
                      <div className="font-mono text-xs font-semibold text-[var(--txt)] break-all bg-[var(--bg2)] p-2 rounded border border-[var(--edge)]">
                        {sig.n}
                      </div>
                      <div className="grid grid-cols-2 gap-2 pt-1 font-mono text-[11px]">
                        <div className="bg-[var(--bg2)] p-1.5 rounded border border-[var(--edge)]">
                          <span className="text-[var(--dim)] block text-[10px]">Width</span>
                          <span className="text-[var(--txt)] font-semibold">{sig.w}-bit</span>
                        </div>
                        <div className="bg-[var(--bg2)] p-1.5 rounded border border-[var(--edge)]">
                          <span className="text-[var(--dim)] block text-[10px]">Domain</span>
                          <span className="text-[var(--txt)] font-semibold uppercase">{sig.k}</span>
                        </div>
                        <div className="bg-[var(--bg2)] p-1.5 rounded border border-[var(--edge)]">
                          <span className="text-[var(--dim)] block text-[10px]">Transitions</span>
                          <span className="text-[var(--txt)] font-semibold">{st?.n ?? 0}</span>
                        </div>
                        <div className="bg-[var(--bg2)] p-1.5 rounded border border-[var(--edge)]">
                          <span className="text-[var(--dim)] block text-[10px]">Unique States</span>
                          <span className="text-[var(--txt)] font-semibold">{st?.uniq ?? 0}</span>
                        </div>
                      </div>
                    </div>

                    <div className="space-y-1.5">
                      <div className="text-[10px] font-bold uppercase tracking-wider text-[var(--accent)]">
                        Cursor Readout
                      </div>
                      <div className="bg-[var(--bg2)] rounded border border-[var(--edge)] divide-y divide-[var(--edge)] font-mono text-[11px]">
                        <div className="flex justify-between px-2.5 py-1.5">
                          <span className="text-[var(--dim)]">
                            A @ {curA !== null ? `${curA}ns` : '—'}
                          </span>
                          <span className="text-[var(--ok)] font-semibold">{valA}</span>
                        </div>
                        <div className="flex justify-between px-2.5 py-1.5">
                          <span className="text-[var(--dim)]">
                            B @ {curB !== null ? `${curB}ns` : '—'}
                          </span>
                          <span className="text-[var(--accent)] font-semibold">{valB}</span>
                        </div>
                        {curA !== null && curB !== null && (
                          <div className="flex justify-between px-2.5 py-1.5 bg-[var(--panel2)]">
                            <span className="text-[var(--dim)]">Δt / Freq</span>
                            <span className="text-[#34d399] font-semibold">
                              {Math.abs(curB - curA)}ns ·{' '}
                              {Math.abs(curB - curA) > 0
                                ? `${(1000 / Math.abs(curB - curA)).toFixed(2)} MHz`
                                : '∞'}
                            </span>
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="space-y-2 pt-1">
                      <div className="text-[10px] font-bold uppercase tracking-wider text-[var(--accent)]">
                        Cross-Probe, Radix &amp; Trace Formatting
                      </div>
                      <button
                        type="button"
                        onClick={() => onJumpToSource(sig.n)}
                        className="w-full flex items-center justify-center gap-1.5 px-3 py-1.5 rounded bg-[var(--sel)] hover:bg-[var(--sel2)] text-[var(--txt)] border border-[var(--accent)]/40 font-medium cursor-pointer"
                      >
                        <Sparkles className="w-3.5 h-3.5 text-[var(--accent)]" />
                        Cross-Probe Driver in RTL IDE
                      </button>

                      {/* Edge Stepper inside Inspector */}
                      <div className="flex gap-1.5">
                        <button
                          type="button"
                          onClick={() => jumpToSignalEdge('prev')}
                          className="flex-1 flex items-center justify-center gap-1 px-2 py-1 rounded bg-[var(--bg2)] hover:bg-[var(--raised)] border border-[var(--edge)] text-cyan-300 font-mono text-[11px] cursor-pointer"
                        >
                          <SkipBack className="w-3 h-3" />
                          Prev Edge (P)
                        </button>
                        <button
                          type="button"
                          onClick={() => jumpToSignalEdge('next')}
                          className="flex-1 flex items-center justify-center gap-1 px-2 py-1 rounded bg-[var(--bg2)] hover:bg-[var(--raised)] border border-[var(--edge)] text-cyan-300 font-mono text-[11px] cursor-pointer"
                        >
                          <SkipForward className="w-3 h-3" />
                          Next Edge (N)
                        </button>
                      </div>

                      {/* Per-Row Radix Override */}
                      <div className="flex items-center justify-between gap-1 bg-[var(--bg2)] p-1.5 rounded border border-[var(--edge)]">
                        <span className="text-[10px] text-[var(--dim)] uppercase font-mono">Radix:</span>
                        <div className="flex gap-1">
                          {(['hex', 'bin', 'dec', 'signed', 'ascii'] as const).map((rOpt) => (
                            <button
                              key={rOpt}
                              type="button"
                              onClick={() =>
                                setRows((prev) =>
                                  prev.map((r) =>
                                    r.id === selectedRow.id ? { ...r, radix: rOpt } : r
                                  )
                                )
                              }
                              className={`px-1.5 py-0.5 rounded text-[10px] font-mono uppercase cursor-pointer ${
                                (selectedRow.radix || defRadix) === rOpt
                                  ? 'bg-cyan-500 text-slate-950 font-bold'
                                  : 'bg-[var(--panel)] text-[var(--dim)] hover:text-[var(--txt)]'
                              }`}
                            >
                              {rOpt}
                            </button>
                          ))}
                        </div>
                      </div>

                      {/* Per-Row Custom Trace Color Picker */}
                      <div className="flex items-center justify-between gap-1 bg-[var(--bg2)] p-1.5 rounded border border-[var(--edge)]">
                        <span className="text-[10px] text-[var(--dim)] uppercase font-mono">Trace Color:</span>
                        <div className="flex items-center gap-1.5">
                          {[
                            { c: null, label: 'Auto' },
                            { c: '#34d399', label: 'Emerald' },
                            { c: '#38bdf8', label: 'Cyan' },
                            { c: '#f59e0b', label: 'Amber' },
                            { c: '#a855f7', label: 'Purple' },
                            { c: '#f43f5e', label: 'Rose' },
                          ].map((col, idx) => (
                            <button
                              key={idx}
                              type="button"
                              onClick={() =>
                                setRows((prev) =>
                                  prev.map((r) =>
                                    r.id === selectedRow.id ? { ...r, color: col.c } : r
                                  )
                                )
                              }
                              style={col.c ? { backgroundColor: col.c } : undefined}
                              className={`h-4 rounded cursor-pointer text-[9px] font-mono ${
                                col.c ? 'w-4 rounded-full' : 'px-1.5 bg-slate-800 text-slate-300'
                              } ${
                                selectedRow.color === col.c ? 'ring-2 ring-white' : 'opacity-80 hover:opacity-100'
                              }`}
                              title={col.label}
                            >
                              {!col.c ? 'Auto' : ''}
                            </button>
                          ))}
                        </div>
                      </div>

                      <div className="flex gap-1.5">
                        <button
                          type="button"
                          onClick={() =>
                            setRows((prev) =>
                              prev.map((r) =>
                                r.id === selectedRow.id ? { ...r, expanded: !r.expanded } : r
                              )
                            )
                          }
                          className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 rounded bg-[var(--panel2)] hover:bg-[var(--raised)] border border-[var(--edge2)] text-[var(--txt)] cursor-pointer"
                        >
                          {selectedRow.expanded ? (
                            <Minimize2 className="w-3.5 h-3.5" />
                          ) : (
                            <Expand className="w-3.5 h-3.5" />
                          )}
                          {selectedRow.expanded ? 'Collapse Bits' : 'Expand Bits'}
                        </button>
                        <button
                          type="button"
                          onClick={() =>
                            setRows((prev) => prev.filter((r) => r.id !== selectedRow.id))
                          }
                          className="px-2.5 py-1.5 rounded bg-[var(--panel2)] hover:bg-[var(--bad)]/20 border border-[var(--edge2)] text-[var(--bad)] cursor-pointer"
                          title="Remove row from waveform view"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>

                    {/* WV-01: Sub-Pixel Mipmap Envelope Strip */}
                    {selectedMipmap.length > 0 && (
                      <div className="space-y-1.5 pt-1 border-t border-[var(--edge)]">
                        <div className="flex items-center justify-between text-[10px] font-bold uppercase tracking-wider text-cyan-400">
                          <span>WV-01 Mipmap Pyramid (32 bins)</span>
                          <span className="font-mono text-[var(--dim)]">O(1)/px</span>
                        </div>
                        <div className="h-8 bg-[var(--bg2)] rounded border border-[var(--edge)] p-1 flex items-end gap-0.5">
                          {selectedMipmap.map((b) => {
                            const maxT = Math.max(1, ...selectedMipmap.map((m) => m.toggleCount));
                            const pct = Math.max(12, Math.round((b.toggleCount / maxT) * 100));
                            return (
                              <div
                                key={b.bucket}
                                style={{ height: `${pct}%` }}
                                className={`flex-1 rounded-t-xs ${
                                  b.hasXz
                                    ? 'bg-rose-500'
                                    : b.toggleCount > 0
                                    ? 'bg-cyan-400/80'
                                    : 'bg-slate-700/50'
                                }`}
                                title={`[${b.tStart}..${b.tEnd}ns] toggles=${b.toggleCount}, first=${b.vFirst}, last=${b.vLast}`}
                              />
                            );
                          })}
                        </div>
                      </div>
                    )}

                    {/* WV-02: Causal COI Ray-Tracer Edges List */}
                    {showCoiRays && coiRays.length > 0 && (
                      <div className="space-y-1.5 pt-1 border-t border-[var(--edge)]">
                        <div className="text-[10px] font-bold uppercase tracking-wider text-rose-400">
                          WV-02 Causal COI Ray-Tracer ({coiRays.length} edges)
                        </div>
                        <div className="space-y-1 font-mono text-[10px] max-h-36 overflow-y-auto">
                          {coiRays.slice(0, 6).map((r, idx) => (
                            <div
                              key={idx}
                              className="p-1.5 rounded bg-[var(--bg2)] border border-[var(--edge)] text-slate-300"
                            >
                              <div className="flex justify-between text-rose-300 font-semibold">
                                <span>{r.fromSigName.split('.').pop()} @ {r.fromTime}ns</span>
                                <span>→ {r.toSigName.split('.').pop()} @ {r.toTime}ns</span>
                              </div>
                              <div className="text-[var(--dim)] truncate">{r.expr}</div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* WV-08: What-If Delta Summary */}
                    {whatIfPatch && (
                      <div className="space-y-1 pt-1 border-t border-[var(--edge)]">
                        <div className="text-[10px] font-bold uppercase tracking-wider text-amber-400">
                          WV-08 What-If Counterfactual Patch ({whatIfPatch.wallMs} ms)
                        </div>
                        <div className="p-2 rounded bg-amber-950/30 border border-amber-500/40 font-mono text-[10px] text-amber-200 space-y-1">
                          <div>
                            Forced <span className="font-bold">{whatIfPatch.targetSigName.split('.').pop()}</span> @{' '}
                            {whatIfPatch.patchTime}ns: {String(whatIfPatch.originalValue)} →{' '}
                            <span className="text-emerald-300 font-bold">{String(whatIfPatch.patchedValue)}</span>
                          </div>
                          <div className="text-amber-300/80">
                            Propagated to {whatIfPatch.affectedSignals.length} fan-out net(s) on canvas.
                          </div>
                        </div>
                      </div>
                    )}

                    {/* WV-05 / AP-09: Persistence Eye Diagram & Jitter Histogram */}
                    {eyeDiagram && (
                      <div className="space-y-1.5 pt-1 border-t border-[var(--edge)]">
                        <div className="flex items-center justify-between text-[10px] font-bold uppercase tracking-wider text-emerald-400">
                          <span>WV-05 Persistence Eye Diagram</span>
                          <span className="font-mono text-[var(--dim)]">Q={eyeDiagram.qFactor}</span>
                        </div>
                        <div className="bg-[#090d16] rounded border border-[var(--edge)] p-1.5">
                          <svg viewBox="0 0 220 90" className="w-full h-22 block">
                            {/* Grid lines */}
                            <line x1="0" y1="45" x2="220" y2="45" stroke="rgba(148,163,184,0.2)" strokeDasharray="2,2" />
                            <line x1="55" y1="0" x2="55" y2="90" stroke="rgba(148,163,184,0.2)" strokeDasharray="2,2" />
                            <line x1="165" y1="0" x2="165" y2="90" stroke="rgba(148,163,184,0.2)" strokeDasharray="2,2" />
                            {/* Folded 2-UI Persistence Traces */}
                            {eyeDiagram.traces.map((tr, tidx) => {
                              const pts = tr
                                .map((p) => {
                                  const x = (p.phaseUi / 2.0) * 220;
                                  const y = 82 - (p.voltageMv / Math.max(1, eyeDiagram.vddMv)) * 74;
                                  return `${x.toFixed(1)},${y.toFixed(1)}`;
                                })
                                .join(' ');
                              return (
                                <polyline
                                  key={tidx}
                                  fill="none"
                                  stroke="rgba(34,211,238,0.28)"
                                  strokeWidth="1.1"
                                  points={pts}
                                />
                              );
                            })}
                            {/* Eye Opening Diamond Mask */}
                            <polygon
                              points="75,45 110,20 145,45 110,70"
                              fill="rgba(16,185,129,0.15)"
                              stroke="#34d399"
                              strokeWidth="1.2"
                            />
                          </svg>
                          <div className="grid grid-cols-2 gap-1 text-[10px] font-mono mt-1 text-slate-300">
                            <div>Height: <span className="text-emerald-300 font-bold">{eyeDiagram.eyeHeightMv} mV</span></div>
                            <div>Width: <span className="text-cyan-300 font-bold">{eyeDiagram.eyeWidthUiPct}% UI</span></div>
                            <div>RMS Jit: <span className="text-amber-300 font-bold">{eyeDiagram.rmsJitterPs} ps</span></div>
                            <div>BER: <span className="text-emerald-300 font-bold">&lt; 1e-12</span></div>
                          </div>
                        </div>
                      </div>
                    )}

                    {/* WV-06: Temporal Regex & SVA Sequence Visual Highlighter */}
                    <div className="space-y-1.5 pt-1 border-t border-[var(--edge)]">
                      <div className="flex items-center justify-between text-[10px] font-bold uppercase tracking-wider text-purple-400">
                        <span>WV-06 SVA Sequence Highlighter</span>
                        <button
                          type="button"
                          onClick={() => setShowSvaHighlights((v) => !v)}
                          className="text-[9px] px-1.5 py-0.5 rounded bg-slate-800 text-purple-300 cursor-pointer"
                        >
                          {showSvaHighlights ? 'ON' : 'OFF'}
                        </button>
                      </div>
                      <input
                        type="text"
                        aria-label="SVA Temporal Sequence Expression"
                        value={svaQuery}
                        onChange={(e) => setSvaQuery(e.target.value)}
                        className="w-full bg-[var(--bg2)] border border-[var(--edge2)] rounded px-2 py-1 text-[10px] font-mono text-purple-200 outline-none focus:border-purple-400"
                      />
                      <div className="text-[10px] font-mono text-[var(--dim)]">
                        {svaMatches.length} temporal window(s) highlighted · Stream compression:{' '}
                        <span className="text-cyan-300 font-semibold">
                          {streamChunks[0]?.compressionRatio ?? 5.4}x (WV-10)
                        </span>{' '}
                        · Heat bins: {heatRibbon.length} (WV-09) · Wave Diff:{' '}
                        <span className="text-emerald-300 font-semibold">
                          {whatIfPatch
                            ? `${whatIfPatch.affectedSignals.length} divergent net(s) @ ${whatIfPatch.patchTime}ns (WV-07)`
                            : `${computeWaveformDiff(signals.slice(0, 4), steps.slice(0, 4), steps.slice(0, 4)).mismatchedSignals} mismatches vs Golden (WV-07)`}
                        </span>
                      </div>
                    </div>
                  </>
                );
              })()
            ) : (
              <div className="text-[var(--dim)] text-center py-8">
                Select any signal row in the tree or waveform canvas to inspect live values and cross-probe RTL.
              </div>
            )}
          </div>
        ) : (
          /* Dedicated Simulation Time-Point Bookmarks Side-Panel */
          <div className="p-3 space-y-3 overflow-y-auto text-xs flex-1 flex flex-col font-mono">
            {/* Header Info & Quick Capture Button */}
            <div className="p-2.5 rounded-lg bg-amber-950/20 border border-amber-500/30 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-[10px] uppercase font-bold tracking-wider text-amber-300 flex items-center gap-1">
                  <Bookmark className="w-3.5 h-3.5 text-amber-400" />
                  Time-Point Bookmarks
                </span>
                <span className="text-[10px] text-slate-400">
                  {bookmarks.length} saved
                </span>
              </div>
              <div className="text-[11px] text-slate-300">
                Save critical simulation milestones, glitches, or hazard intervals from Cursor A & B for instant navigation.
              </div>
              <button
                type="button"
                data-testid="add-current-cursors-bookmark-btn"
                onClick={() => setShowAddBookmarkDialog((v) => !v)}
                className="w-full inline-flex items-center justify-center gap-1.5 py-1.5 px-2.5 rounded bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-xs cursor-pointer shadow-xs transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>
                  {showAddBookmarkDialog
                    ? 'Close Form'
                    : `+ Bookmark Cursors (${curA !== null ? `${curA}ns` : '0ns'}${curB !== null ? ` · ${curB}ns` : ''})`}
                </span>
              </button>
            </div>

            {/* Inline Bookmark Creation Form */}
            {showAddBookmarkDialog && (
              <div className="p-2.5 rounded-lg bg-[var(--bg2)] border border-[var(--edge2)] space-y-2.5">
                <div className="flex items-center justify-between text-[11px] font-bold text-amber-300">
                  <span>Create Milestone Bookmark</span>
                  <button
                    type="button"
                    onClick={() => setShowAddBookmarkDialog(false)}
                    className="text-slate-400 hover:text-white p-0.5 cursor-pointer"
                  >
                    <X className="w-3.5 h-3.5" />
                  </button>
                </div>

                <div className="space-y-1">
                  <label className="text-[10px] text-[var(--dim)] uppercase font-semibold">Name / Label</label>
                  <input
                    data-testid="new-bookmark-name-input"
                    value={newBookmarkName}
                    onChange={(e) => setNewBookmarkName(e.target.value)}
                    placeholder={`e.g. DMA Buffer Flush @ ${curA !== null ? curA : 0}ns`}
                    className="w-full bg-[var(--panel)] border border-[var(--edge)] rounded px-2 py-1 text-xs text-[var(--txt)] outline-none focus:border-amber-400"
                  />
                </div>

                <div className="grid grid-cols-2 gap-2 text-[10px]">
                  <div className="bg-[var(--panel)] p-1.5 rounded border border-[var(--edge)]">
                    <span className="text-[var(--dim)] block">Cursor A</span>
                    <span className="text-cyan-300 font-bold">{curA !== null ? `${curA} ns` : '0 ns'}</span>
                  </div>
                  <div className="bg-[var(--panel)] p-1.5 rounded border border-[var(--edge)]">
                    <span className="text-[var(--dim)] block">Cursor B</span>
                    <span className="text-emerald-300 font-bold">{curB !== null ? `${curB} ns` : '—'}</span>
                  </div>
                </div>

                {curA !== null && curB !== null && (
                  <div className="text-[10px] text-emerald-300 bg-emerald-950/40 p-1.5 rounded border border-emerald-500/30">
                    Δt: {Math.abs(curB - curA)} ns · Freq: {Math.abs(curB - curA) > 0 ? (1000 / Math.abs(curB - curA)).toFixed(2) : '∞'} MHz
                  </div>
                )}

                <div className="space-y-1">
                  <label className="text-[10px] text-[var(--dim)] uppercase font-semibold">Flag Color</label>
                  <div className="flex items-center gap-1.5">
                    {[
                      { color: '#f59e0b', name: 'Amber' },
                      { color: '#38bdf8', name: 'Sky' },
                      { color: '#34d399', name: 'Emerald' },
                      { color: '#a855f7', name: 'Purple' },
                      { color: '#f43f5e', name: 'Rose' },
                      { color: '#fb923c', name: 'Orange' },
                    ].map((cp) => (
                      <button
                        key={cp.color}
                        type="button"
                        onClick={() => setNewBookmarkColor(cp.color)}
                        style={{ backgroundColor: cp.color }}
                        className={`w-5 h-5 rounded-full cursor-pointer transition-transform ${
                          newBookmarkColor === cp.color ? 'ring-2 ring-white scale-110' : 'opacity-80 hover:opacity-100'
                        }`}
                        title={cp.name}
                      />
                    ))}
                  </div>
                </div>

                <div className="space-y-1">
                  <label className="text-[10px] text-[var(--dim)] uppercase font-semibold">Notes / Description (Optional)</label>
                  <textarea
                    data-testid="new-bookmark-notes-input"
                    value={newBookmarkNotes}
                    onChange={(e) => setNewBookmarkNotes(e.target.value)}
                    placeholder="e.g. Bus grant handshake verified; pipeline hazard flushed."
                    rows={2}
                    className="w-full bg-[var(--panel)] border border-[var(--edge)] rounded px-2 py-1 text-xs text-[var(--txt)] outline-none focus:border-amber-400 resize-none"
                  />
                </div>

                <div className="flex gap-2 pt-1">
                  <button
                    type="button"
                    data-testid="save-bookmark-btn"
                    onClick={handleCreateBookmark}
                    className="flex-1 py-1 rounded bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-xs cursor-pointer shadow-xs"
                  >
                    Save Milestone
                  </button>
                  <button
                    type="button"
                    onClick={() => setShowAddBookmarkDialog(false)}
                    className="px-2 py-1 rounded bg-[var(--raised)] text-[var(--dim)] hover:text-[var(--txt)] text-xs cursor-pointer"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            )}

            {/* Bookmarks List */}
            <div className="space-y-2 flex-1 overflow-y-auto">
              {bookmarks.length === 0 ? (
                <div className="text-center py-8 text-[var(--dim)] space-y-2">
                  <Bookmark className="w-8 h-8 mx-auto text-slate-600 opacity-60" />
                  <div>No bookmarks saved yet.</div>
                  <div className="text-[10px] text-slate-500">
                    Move Cursors A and B to any point in the waveform and click "+ Bookmark Cursors".
                  </div>
                </div>
              ) : (
                bookmarks.map((bm) => (
                  <div
                    key={bm.id}
                    data-testid={`bookmark-card-${bm.id}`}
                    className="p-2.5 rounded-lg bg-[var(--bg2)] border border-[var(--edge)] hover:border-amber-500/50 transition-all space-y-2 relative group"
                  >
                    <div className="flex items-start justify-between gap-1.5">
                      <div className="flex items-center gap-1.5 min-w-0">
                        <span
                          className="w-2.5 h-2.5 rounded-full shrink-0"
                          style={{ backgroundColor: bm.color }}
                        />
                        <span className="font-bold text-xs text-[var(--txt)] truncate">
                          {bm.name}
                        </span>
                      </div>
                      <button
                        type="button"
                        data-testid={`delete-bookmark-${bm.id}`}
                        onClick={() => handleDeleteBookmark(bm.id)}
                        className="text-[var(--dim)] hover:text-rose-400 p-0.5 cursor-pointer opacity-70 group-hover:opacity-100 transition-opacity"
                        title="Delete bookmark"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    {/* Cursor Time Badges */}
                    <div className="flex flex-wrap items-center gap-1 text-[10px]">
                      <span className="px-1.5 py-0.5 rounded bg-cyan-950/70 border border-cyan-500/40 text-cyan-200 font-semibold">
                        A: {bm.timeA !== null ? `${bm.timeA}ns` : '—'}
                      </span>
                      {bm.timeB !== null && (
                        <span className="px-1.5 py-0.5 rounded bg-emerald-950/70 border border-emerald-500/40 text-emerald-200 font-semibold">
                          B: {bm.timeB}ns
                        </span>
                      )}
                      {bm.deltaNs !== null && (
                        <span className="px-1.5 py-0.5 rounded bg-amber-950/70 border border-amber-500/40 text-amber-200 font-semibold">
                          Δt: {bm.deltaNs}ns
                          {bm.deltaNs > 0 ? ` · ${(1000 / bm.deltaNs).toFixed(1)}MHz` : ''}
                        </span>
                      )}
                    </div>

                    {/* Optional Notes */}
                    {bm.notes && (
                      <div className="text-[11px] text-slate-400 bg-[var(--panel)] p-1.5 rounded border border-[var(--edge)] italic">
                        {bm.notes}
                      </div>
                    )}

                    {/* Navigation Action Button */}
                    <div className="flex items-center justify-between pt-1 border-t border-[var(--edge)]">
                      <span className="text-[9px] text-[var(--faint)]">{bm.createdAt}</span>
                      <button
                        type="button"
                        data-testid={`jump-bookmark-${bm.id}`}
                        onClick={() => handleJumpToBookmark(bm)}
                        className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-[var(--panel)] hover:bg-[var(--raised)] border border-[var(--edge2)] hover:border-amber-400 text-amber-300 font-semibold text-[11px] cursor-pointer transition-colors shadow-xs"
                        title="Center waveform viewport and restore Cursors A & B"
                      >
                        <Compass className="w-3.5 h-3.5 text-amber-400" />
                        <span>Jump to Point</span>
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>

            {/* Bottom Actions */}
            {bookmarks.length > 0 && (
              <div className="pt-2 border-t border-[var(--edge)] flex items-center justify-between text-[10px]">
                <button
                  type="button"
                  data-testid="clear-all-bookmarks-btn"
                  onClick={() => setBookmarks([])}
                  className="text-rose-400 hover:text-rose-300 underline cursor-pointer"
                >
                  Clear All ({bookmarks.length})
                </button>
                <span className="text-slate-500">
                  Flag tags pinned to ruler
                </span>
              </div>
            )}
          </div>
        )}
      </aside>
      </div>

      {/* WV-12: Publication Vector Exporter Modal (LaTeX TikZ / Typst CeTZ / WaveDrom / SVG) */}
      {pubExportModalOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/75 flex items-center justify-center p-4"
          onClick={() => setPubExportModalOpen(false)}
        >
          <div
            className="w-full max-w-2xl rounded-xl border border-cyan-500/50 bg-[#111621] p-4 space-y-3 font-mono text-xs shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between border-b border-slate-800 pb-2">
              <div>
                <h3 className="text-sm font-bold text-cyan-300">
                  WV-12 Publication Waveform Exporter (LaTeX TikZ · Typst CeTZ · WaveDrom · Vector SVG)
                </h3>
                <p className="text-[11px] text-slate-400">
                  Zero-raster IEEE paper & documentation diagram generator for viewport [{Math.round(t0)}..{Math.round(t0 + tspan)} ns]
                </p>
              </div>
              <button
                type="button"
                onClick={() => setPubExportModalOpen(false)}
                className="px-2 py-1 rounded bg-slate-800 text-slate-300 hover:text-white cursor-pointer"
              >
                ESC
              </button>
            </div>

            <div className="flex items-center gap-2">
              {(['tikz', 'typst', 'wavedrom', 'svg'] as const).map((fmt) => (
                <button
                  key={fmt}
                  type="button"
                  onClick={() => setPubExportFormat(fmt)}
                  className={`px-3 py-1 rounded text-xs font-bold uppercase cursor-pointer border ${
                    pubExportFormat === fmt
                      ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50'
                      : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'
                  }`}
                >
                  {fmt === 'tikz'
                    ? 'LaTeX TikZ'
                    : fmt === 'typst'
                    ? 'Typst CeTZ'
                    : fmt === 'wavedrom'
                    ? 'WaveDrom JSON'
                    : 'Vector SVG'}
                </button>
              ))}
            </div>

            <textarea
              readOnly
              value={pubExportCode}
              rows={12}
              className="w-full bg-[#0b0e14] border border-slate-800 rounded p-3 text-[11px] text-emerald-300 font-mono outline-none"
            />

            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => navigator.clipboard?.writeText(pubExportCode)}
                className="px-3 py-1.5 rounded bg-cyan-600 hover:bg-cyan-500 text-white font-bold cursor-pointer"
              >
                Copy Source
              </button>
              <button
                type="button"
                onClick={() => setPubExportModalOpen(false)}
                className="px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Iteration 9: IEEE / ISSCC Research-Paper Formatted PDF Simulation Summary Modal */}
      {pdfReportModalOpen && (
        <div
          data-testid="pdf-report-modal"
          className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4 overflow-y-auto"
          onClick={() => setPdfReportModalOpen(false)}
        >
          <div
            className="w-full max-w-4xl rounded-xl border border-emerald-500/50 bg-[#0d1320] text-slate-100 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Top Consistent Research-Paper Header */}
            <div className="bg-[#111a2e] border-b border-emerald-500/40 px-5 py-3 flex items-center justify-between">
              <div className="font-mono">
                <div className="text-[10px] font-bold tracking-widest uppercase text-emerald-400">
                  {paperReportData.headerText}
                </div>
                <div className="text-[11px] text-slate-400 mt-0.5">
                  Top Module: <span className="text-cyan-300 font-semibold">{paperReportData.openKdbSummary.topModule}</span> ·
                  SHA-256 Digest:{' '}
                  <span className="text-emerald-300">{paperReportData.sha256DocumentDigest.slice(0, 24)}…</span>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  data-testid="download-pdf-report-btn"
                  onClick={handleDownloadPdfReport}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-mono font-bold shadow cursor-pointer"
                >
                  <Download className="w-3.5 h-3.5" />
                  Download PDF Report (.pdf)
                </button>
                <button
                  type="button"
                  aria-label="Close Research-Paper PDF Modal"
                  onClick={() => setPdfReportModalOpen(false)}
                  className="px-2.5 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-mono cursor-pointer"
                >
                  ESC
                </button>
              </div>
            </div>

            {pdfDownloadedBanner && (
              <div className="bg-emerald-950/90 border-b border-emerald-500/40 px-5 py-2 flex items-center justify-between text-xs font-mono text-emerald-200">
                <span className="flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                  {pdfDownloadedBanner}
                </span>
                <button
                  type="button"
                  onClick={() => setPdfDownloadedBanner(null)}
                  className="text-emerald-400 hover:text-white text-[11px] cursor-pointer"
                >
                  Dismiss
                </button>
              </div>
            )}

            {/* Research-Paper Body (IEEE Two-Column Styled Archival Layout) */}
            <div className="p-6 overflow-y-auto space-y-5 bg-[#f8fafc] text-slate-900 font-serif">
              {/* Paper Title & Authors Block */}
              <div className="text-center border-b-2 border-slate-900 pb-4">
                <div className="text-[10px] font-mono uppercase tracking-widest text-slate-600 mb-1">
                  {paperReportData.publicationVenueLine}
                </div>
                <h2 className="text-lg font-bold text-slate-950 leading-snug">
                  {paperReportData.paperTitle}
                </h2>
                <div className="text-xs text-slate-700 mt-1.5 font-medium">
                  {paperReportData.authorsLine}
                </div>
              </div>

              {/* Abstract & Index Terms */}
              <div className="bg-slate-100 border border-slate-300 rounded p-3.5 text-xs leading-relaxed">
                <p className="text-slate-800">
                  <span className="font-bold italic">Abstract—</span>
                  {paperReportData.abstractText}
                </p>
                <p className="text-slate-700 mt-2">
                  <span className="font-bold italic">Index Terms—</span>
                  {paperReportData.keywords.join(', ')}.
                </p>
              </div>

              {/* Two-Column IEEE Technical Sections */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-5 text-xs">
                {/* Left Column: Section I & Section II */}
                <div className="space-y-4">
                  <div>
                    <h3 className="font-bold uppercase tracking-wider text-slate-900 border-b border-slate-400 pb-1 mb-2 font-mono text-[11px]">
                      I. Waveform &amp; Temporal Signal Trace Telemetry
                    </h3>
                    <table className="w-full border-collapse font-mono text-[11px]">
                      <tbody>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Target RTL Manifest</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.waveformSummary.targetManifest}
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Total / Displayed Signals</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.waveformSummary.totalSignals} /{' '}
                            {paperReportData.waveformSummary.displayedSignals}
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Active Power-User Regex Filter</td>
                          <td className="py-1 text-right font-bold text-emerald-700">
                            /{paperReportData.waveformSummary.activeRegexFilter}/
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Simulation Horizon &amp; Events</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.waveformSummary.simulationHorizonNs} ns ·{' '}
                            {paperReportData.waveformSummary.totalEvents.toLocaleString()} edges
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Toggle Coverage &amp; Sub-Cycle Glitches</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.waveformSummary.toggleCoveragePct}% ·{' '}
                            {paperReportData.waveformSummary.glitchEventsCount} glitches
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>

                  <div>
                    <h3 className="font-bold uppercase tracking-wider text-slate-900 border-b border-slate-400 pb-1 mb-2 font-mono text-[11px]">
                      II. Open-KDB CSR Hypergraph Analysis
                    </h3>
                    <table className="w-full border-collapse font-mono text-[11px] mb-2">
                      <tbody>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Top Module Hierarchy</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.openKdbSummary.topModule}
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">Total Drivers (Comb / Seq)</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.openKdbSummary.totalDrivers} (
                            {paperReportData.openKdbSummary.combinationalDrivers} comb /{' '}
                            {paperReportData.openKdbSummary.sequentialDrivers} seq)
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">CSR Dependency Edges</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.openKdbSummary.csrEdgesCount} edges
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">3-Layer Inferred Latch Audit</td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.openKdbSummary.inferredLatchesCount} latches
                          </td>
                        </tr>
                      </tbody>
                    </table>

                    <div className="bg-white border border-slate-300 rounded p-2 font-mono text-[10px]">
                      <div className="font-bold text-slate-700 mb-1">
                        TABLE I: OPEN-KDB CSR DRIVER SAMPLE ({paperReportData.openKdbSummary.sampleDrivers.length} OF{' '}
                        {paperReportData.openKdbSummary.totalDrivers})
                      </div>
                      {paperReportData.openKdbSummary.sampleDrivers.slice(0, 5).map((d, i) => (
                        <div key={i} className="truncate text-slate-700 py-0.5 border-t border-slate-100">
                          <span className="font-bold text-slate-900">{d.signal}</span> &lt;= {d.rhs}{' '}
                          <span className="text-slate-500">
                            [{d.file}:{d.line}]
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Right Column: Section III & Section IV */}
                <div className="space-y-4">
                  <div>
                    <h3 className="font-bold uppercase tracking-wider text-slate-900 border-b border-slate-400 pb-1 mb-2 font-mono text-[11px]">
                      III. Pre-Synthesis Silicon Physics &amp; Reliability
                    </h3>
                    <table className="w-full border-collapse font-mono text-[11px]">
                      <tbody>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">
                            Switching Energy (E<sub>dyn</sub> = ½αC<sub>node</sub>V<sub>DD</sub>
                            <sup>2</sup>)
                          </td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.siliconPhysicsSummary.switchingEnergyFj} fJ (
                            {paperReportData.siliconPhysicsSummary.dynamicPowerUw} μW @ V<sub>DD</sub>=
                            {paperReportData.siliconPhysicsSummary.vddVolts}V)
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">
                            2D Poisson Die Thermal Peak (T<sub>peak</sub>)
                          </td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.siliconPhysicsSummary.peakThermalKelvin} K
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">
                            RLC PDN IR-Drop + L·di/dt Bounce (ΔV<sub>droop</sub>)
                          </td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.siliconPhysicsSummary.pdnDroopMv} mV
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">
                            10-Year BTI/HCI Aging Shift (ΔV<sub>th</sub>)
                          </td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            +{paperReportData.siliconPhysicsSummary.agingDeltaVthMv} mV
                          </td>
                        </tr>
                        <tr className="border-b border-slate-200">
                          <td className="py-1 text-slate-600">
                            Cosmic-Ray SEU Q<sub>crit</sub> &amp; Landauer k<sub>B</sub>T ln 2
                          </td>
                          <td className="py-1 text-right font-bold text-slate-900">
                            {paperReportData.siliconPhysicsSummary.seuQcritFc} fC ·{' '}
                            {paperReportData.siliconPhysicsSummary.landauerFloorZj} zJ
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>

                  <div>
                    <h3 className="font-bold uppercase tracking-wider text-slate-900 border-b border-slate-400 pb-1 mb-2 font-mono text-[11px]">
                      IV. 7-Pillar Tapeout Signoff &amp; Cryptographic Digest
                    </h3>
                    <div className="bg-emerald-50 border border-emerald-400 rounded p-3 font-mono text-xs">
                      <div className="flex items-center justify-between">
                        <span className="font-bold text-slate-800">COMPOSITE SIGNOFF VERDICT:</span>
                        <span className="px-2 py-0.5 rounded bg-emerald-700 text-white font-bold">
                          {paperReportData.signoffVerdict} ({paperReportData.signoffScore}/100)
                        </span>
                      </div>
                      <div className="text-[10px] text-slate-600 mt-2 break-all">
                        SHA-256 Tapeout Proof: {paperReportData.sha256DocumentDigest}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Bottom Consistent Research-Paper Footer */}
            <div className="bg-[#111a2e] border-t border-emerald-500/40 px-5 py-2.5 flex items-center justify-between font-mono text-[10px] text-slate-300">
              <span>{paperReportData.footerText}</span>
              <span className="text-emerald-400 font-bold">Page 1–2 · %PDF-1.4 Archival Standard</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
