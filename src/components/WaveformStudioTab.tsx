import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  Activity,
  ChevronDown,
  ChevronRight,
  Crosshair,
  Download,
  Expand,
  FolderOpen,
  Layers,
  Maximize2,
  Minimize2,
  Play,
  Plus,
  Ruler,
  Search,
  Sparkles,
  Trash2,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import {
  formatSignalValue,
  SigKind,
  SignalMeta,
  SignalSteps,
  valAtTime,
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
}) => {
  const [rows, setRows] = useState<WaveRowItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [treeFilter, setTreeFilter] = useState('');
  const [collapsedScopes, setCollapsedScopes] = useState<Set<string>>(new Set());
  const [t0, setT0] = useState(0);
  const [tspan, setTspan] = useState(Math.max(endNs, 100));
  const [searchQuery, setSearchQuery] = useState('');
  const [searchMode, setSearchMode] = useState<'signal' | 'expr' | 'goto'>('signal');
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

  // Group signals by hierarchical scope
  const scopes = useMemo(() => {
    const map = new Map<string, number[]>();
    let rx: RegExp | null = null;
    if (treeFilter.trim()) {
      try {
        rx = new RegExp(treeFilter.trim(), 'i');
      } catch {
        rx = null;
      }
    }
    signals.forEach((s, idx) => {
      if (rx && !rx.test(s.n)) return;
      const parts = s.n.split('.');
      const scope = parts.length > 1 ? parts.slice(0, -1).join('.') : 'top';
      const list = map.get(scope) || [];
      list.push(idx);
      map.set(scope, list);
    });
    return [...map.entries()];
  }, [signals, treeFilter]);

  const inViewSet = useMemo(() => new Set(rows.map((r) => r.si)), [rows]);

  const selectedRow = useMemo(
    () => rows.find((r) => r.id === selectedId) || rows[0] || null,
    [rows, selectedId]
  );

  // Compute vertical layout of rows (accounting for bus bit expansion and analog height)
  const layoutItems = useMemo(() => {
    const out: {
      row: WaveRowItem;
      kind: 'sig' | 'bit';
      bit?: number;
      y: number;
      h: number;
    }[] = [];
    let y = 28; // top ruler height
    for (const r of rows) {
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
  }, [rows, signals]);

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
  }, [layoutItems, signals, steps, t0, tspan, curA, curB, defRadix, selectedRow, viewportSize]);

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
      const bh = (counts[i] / maxC) * (H - 8);
      ctx.fillStyle = 'rgba(34, 211, 238, 0.32)';
      ctx.fillRect(i * barW, H - bh - 2, Math.max(1, barW - 1), bh);
    }

    // Current viewport box
    const vx0 = (t0 / maxEnd) * W;
    const vw = Math.max(12, (tspan / maxEnd) * W);
    ctx.fillStyle = 'rgba(56, 189, 248, 0.16)';
    ctx.strokeStyle = '#22d3ee';
    ctx.lineWidth = 1.5;
    ctx.fillRect(vx0, 2, vw, H - 4);
    ctx.strokeRect(vx0, 2, vw, H - 4);
  }, [steps, endNs, t0, tspan, viewportSize]);

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

  const handleFit = () => {
    setT0(0);
    setTspan(Math.max(endNs, 100));
  };

  const handleRunSearch = () => {
    const q = searchQuery.trim();
    if (!q) return;
    if (searchMode === 'goto' || /^goto:\s*\d+/i.test(q)) {
      const m = q.match(/\d+/);
      if (m) {
        const targetT = Math.min(endNs, parseInt(m[0], 10));
        setCurA(targetT);
        setT0(Math.max(0, Math.min(endNs - tspan, targetT - tspan / 2)));
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
      if (additions.length > 0) setRows((prev) => [...prev, ...additions]);
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

        {/* Search & Temporal Query Box */}
        <div className="flex items-center bg-[var(--bg2)] border border-[var(--edge2)] rounded px-2 py-0.5 ml-1 flex-1 max-w-md">
          <Search className="w-3.5 h-3.5 text-[var(--dim)] mr-1.5 shrink-0" />
          <input
            aria-label="Search signal or boolean expression"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleRunSearch();
            }}
            placeholder="search signal · add:uart · goto:1200"
            className="bg-transparent text-xs font-mono text-[var(--txt)] outline-none flex-1 min-w-0"
          />
          <button
            type="button"
            onClick={() =>
              setSearchMode((m) => (m === 'signal' ? 'expr' : m === 'expr' ? 'goto' : 'signal'))
            }
            className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[var(--panel)] text-[var(--accent)] border border-[var(--edge)] ml-1 cursor-pointer"
          >
            {searchMode.toUpperCase()}
          </button>
          <button
            type="button"
            onClick={handleRunSearch}
            aria-label="Execute Search"
            className="p-1 text-[var(--dim)] hover:text-[var(--accent)] cursor-pointer"
          >
            <Play className="w-3 h-3" />
          </button>
        </div>

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

          <div className="p-2 border-b border-[var(--edge)]">
            <input
              aria-label="Filter signals in hierarchy tree"
              value={treeFilter}
              onChange={(e) => setTreeFilter(e.target.value)}
              placeholder="filter signals (regex ok)..."
              className="w-full bg-[var(--bg2)] border border-[var(--edge2)] rounded px-2.5 py-1 text-xs font-mono text-[var(--txt)] outline-none focus:border-[var(--accent)]"
            />
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

        {/* Center Waveform Canvas + Minimap */}
        <div className="flex-1 flex flex-col min-w-0 bg-[var(--bg)] relative">
          <div ref={wrapRef} className="flex-1 overflow-y-auto overflow-x-hidden relative">
            <canvas
              ref={canvasRef}
              data-testid="waveform-canvas"
              onClick={handleCanvasClick}
              onMouseMove={handleCanvasMouseMove}
              onMouseLeave={() => setHoverInfo(null)}
              onWheel={(e) => {
                e.preventDefault();
                handleZoom(e.deltaY < 0 ? 0.8 : 1.25);
              }}
              className="block cursor-crosshair"
            />
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

        {/* Right Signal Properties & Cross-Probe Dock */}
        <aside className="w-72 bg-[var(--panel)] border-l border-[var(--edge)] flex flex-col min-h-0 shrink-0">
          <div className="px-3 py-1.5 border-b border-[var(--edge)] text-[11px] font-bold tracking-wider uppercase text-[var(--dim)]">
            Signal Inspector
          </div>
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
                        Cross-Probe & Display
                      </div>
                      <button
                        type="button"
                        onClick={() => onJumpToSource(sig.n)}
                        className="w-full flex items-center justify-center gap-1.5 px-3 py-1.5 rounded bg-[var(--sel)] hover:bg-[var(--sel2)] text-[var(--txt)] border border-[var(--accent)]/40 font-medium cursor-pointer"
                      >
                        <Sparkles className="w-3.5 h-3.5 text-[var(--accent)]" />
                        Cross-Probe Driver in RTL IDE
                      </button>

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
                  </>
                );
              })()
            ) : (
              <div className="text-[var(--dim)] text-center py-8">
                Select any signal row in the tree or waveform canvas to inspect live values and cross-probe RTL.
              </div>
            )}
          </div>
        </aside>
      </div>
    </div>
  );
};
