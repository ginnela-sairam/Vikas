import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Bot,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Code2,
  Cpu,
  FileCode,
  FilePlus,
  Info,
  Maximize2,
  Minimize2,
  Play,
  Save,
  Search,
  ShieldAlert,
  Sparkles,
  Terminal,
  Wand2,
  Zap,
} from 'lucide-react';
import {
  analyzeLatches,
  computeLocalAiRtlCompletions,
  formatSignalValue,
  lintRtlSource,
  OpenKdb,
  RtlCompletionSuggestion,
  RtlLintDiagnostic,
  SignalMeta,
  SignalSteps,
  SUPPORTED_EDA_LANGUAGES_CATALOG_33,
  valAtTime,
} from '../engine/vikasEngine';

export interface ProjectFileItem {
  path: string;
  content: string;
}

interface RtlEditorIdeTabProps {
  files: ProjectFileItem[];
  activeFilePath: string;
  setActiveFilePath: (p: string) => void;
  onUpdateFileContent: (path: string, content: string) => void;
  onSaveFile: (path: string, content: string) => Promise<void>;
  onAutoPatchLatches: (path: string, content: string) => Promise<void>;
  onSimulateFile: (path: string, content: string) => Promise<void>;
  onCreateNewFile: (path: string, content: string) => void;
  signals: SignalMeta[];
  steps: SignalSteps[];
  cursorTimeNs: number;
  kdb?: OpenKdb;
}

export const RtlEditorIdeTab: React.FC<RtlEditorIdeTabProps> = ({
  files,
  activeFilePath,
  setActiveFilePath,
  onUpdateFileContent,
  onSaveFile,
  onAutoPatchLatches,
  onSimulateFile,
  onCreateNewFile,
  signals,
  steps,
  cursorTimeNs,
  kdb,
}) => {
  const [vedMode, setVedMode] = useState<'INSERT' | 'NORMAL' | 'COMMAND'>('INSERT');
  const [vedCmd, setVedCmd] = useState('');
  const [fileFilter, setFileFilter] = useState('');
  const [newFileModal, setNewFileModal] = useState(false);
  const [newFileName, setNewFileName] = useState('examples/custom_alu.sv');
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  // Layout view mode: 'SPLIT' | 'EDITOR_ONLY' | 'TRACE_ONLY'
  const [viewMode, setViewMode] = useState<'SPLIT' | 'EDITOR_ONLY' | 'TRACE_ONLY'>('SPLIT');
  
  // Problems / Diagnostics Dock state
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(true);
  const [selectedLanguage, setSelectedLanguage] = useState<string>('SystemVerilog');
  
  // Local In-Browser AI Code Completion state
  const [cursorPos, setCursorPos] = useState<number>(0);
  const [cursorLine, setCursorLine] = useState<number>(1);
  const [cursorCol, setCursorCol] = useState<number>(1);
  const [aiSuggestionsOpen, setAiSuggestionsOpen] = useState(false);
  const [activeSuggestionIdx, setActiveSuggestionIdx] = useState(0);
  const [quickInsertMenuOpen, setQuickInsertMenuOpen] = useState(false);

  // Active diagnostic tooltip popover
  const [activeDiagTooltip, setActiveDiagTooltip] = useState<RtlLintDiagnostic | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const gutterRef = useRef<HTMLDivElement>(null);
  const traceGutterRef = useRef<HTMLDivElement>(null);

  const activeFile = useMemo(
    () =>
      files.find((f) => f.path === activeFilePath) ||
      files[0] || { path: 'examples/demo_latch_trap.sv', content: '' },
    [files, activeFilePath]
  );

  // Split lines
  const lines = useMemo(() => activeFile.content.split('\n'), [activeFile.content]);

  // Real-Time RTL Linting Service (SEC-01 .. SEC-07)
  const lintDiagnostics = useMemo(
    () => lintRtlSource(activeFile.content, activeFile.path),
    [activeFile.content, activeFile.path]
  );

  // Map diagnostics to lines
  const lineDiagMap = useMemo(() => {
    const map = new Map<number, RtlLintDiagnostic[]>();
    for (const d of lintDiagnostics) {
      const list = map.get(d.line) || [];
      list.push(d);
      map.set(d.line, list);
    }
    return map;
  }, [lintDiagnostics]);

  const errorCount = useMemo(
    () => lintDiagnostics.filter((d) => d.severity === 'error').length,
    [lintDiagnostics]
  );
  const warningCount = useMemo(
    () => lintDiagnostics.filter((d) => d.severity === 'warning').length,
    [lintDiagnostics]
  );

  // Live L1 static latch audit on the currently open buffer
  const liveLatches = useMemo(
    () => analyzeLatches([{ path: activeFile.path, content: activeFile.content }]),
    [activeFile.path, activeFile.content]
  );

  // Inline live signal values at Cursor A timestamp (Feature 15)
  const inlineAnnotations = useMemo(() => {
    const map = new Map<string, string>();
    signals.forEach((sig, i) => {
      const short = sig.n.split('.').pop() || sig.n;
      if (!map.has(short)) {
        const val = valAtTime(steps[i], sig.w, cursorTimeNs);
        map.set(short, formatSignalValue(val, sig.w, 'hex'));
      }
    });
    return map;
  }, [signals, steps, cursorTimeNs]);

  const filteredFiles = useMemo(() => {
    const q = fileFilter.trim().toLowerCase();
    if (!q) return files;
    return files.filter((f) => f.path.toLowerCase().includes(q));
  }, [files, fileFilter]);

  // Extract token before cursor for Local In-Browser AI code completions
  const currentTokenPrefix = useMemo(() => {
    if (cursorPos <= 0 || cursorPos > activeFile.content.length) return '';
    const textBefore = activeFile.content.slice(0, cursorPos);
    const m = textBefore.match(/([A-Za-z0-9_]+)$/);
    return m ? m[1] : '';
  }, [activeFile.content, cursorPos]);

  // Local AI suggestions computed with sub-millisecond zero-network latency
  const aiSuggestions: RtlCompletionSuggestion[] = useMemo(() => {
    if (!aiSuggestionsOpen && currentTokenPrefix.length < 2) return [];
    return computeLocalAiRtlCompletions(
      currentTokenPrefix,
      activeFile.content,
      cursorPos,
      signals,
      kdb,
      selectedLanguage
    );
  }, [currentTokenPrefix, activeFile.content, cursorPos, signals, kdb, selectedLanguage, aiSuggestionsOpen]);

  // Synchronize textarea scroll with gutter
  const handleEditorScroll = () => {
    if (textareaRef.current && gutterRef.current) {
      gutterRef.current.scrollTop = textareaRef.current.scrollTop;
    }
  };

  // Track cursor position, line and col
  const updateCursorTracking = (e: React.SyntheticEvent<HTMLTextAreaElement>) => {
    const target = e.currentTarget;
    const pos = target.selectionStart || 0;
    setCursorPos(pos);

    const textBefore = activeFile.content.slice(0, pos);
    const lineNum = textBefore.split('\n').length;
    const lastLineStart = textBefore.lastIndexOf('\n');
    const colNum = pos - (lastLineStart === -1 ? 0 : lastLineStart + 1) + 1;
    setCursorLine(lineNum);
    setCursorCol(colNum);
  };

  // Keyboard navigation for completion dropdown & modal editor
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Trigger AI code completion on Ctrl+Space
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault();
      setAiSuggestionsOpen(true);
      setActiveSuggestionIdx(0);
      return;
    }

    // Escape closes suggestions
    if (e.key === 'Escape') {
      if (aiSuggestionsOpen) {
        e.preventDefault();
        setAiSuggestionsOpen(false);
      }
      return;
    }

    // Navigate completion list if open
    if (aiSuggestionsOpen && aiSuggestions.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setActiveSuggestionIdx((i) => (i + 1) % aiSuggestions.length);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setActiveSuggestionIdx((i) => (i - 1 + aiSuggestions.length) % aiSuggestions.length);
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        insertAiSuggestion(aiSuggestions[activeSuggestionIdx]);
        return;
      }
    }
  };

  // Insert completion suggestion into active buffer
  const insertAiSuggestion = (sug: RtlCompletionSuggestion) => {
    if (!sug) return;
    const prefixLen = currentTokenPrefix.length;
    const start = Math.max(0, cursorPos - prefixLen);
    const end = cursorPos;
    const newContent =
      activeFile.content.slice(0, start) + sug.insertText + activeFile.content.slice(end);
    onUpdateFileContent(activeFile.path, newContent);
    setAiSuggestionsOpen(false);

    // Reposition cursor after inserted text
    setTimeout(() => {
      if (textareaRef.current) {
        const newPos = start + sug.insertText.length;
        textareaRef.current.focus();
        textareaRef.current.setSelectionRange(newPos, newPos);
        setCursorPos(newPos);
      }
    }, 10);
    setStatusMsg(`[VIKAS Local-AI] Inserted ${sug.label}`);
  };

  // Quick fix application
  const applyQuickFix = (diag: RtlLintDiagnostic) => {
    if (!diag.quickFix) return;
    const targetIdx = diag.quickFix.targetLine - 1;
    if (targetIdx < 0 || targetIdx >= lines.length) return;
    const newLines = [...lines];
    newLines[targetIdx] = diag.quickFix.replacement;
    const updated = newLines.join('\n');
    onUpdateFileContent(activeFile.path, updated);
    setActiveDiagTooltip(null);
    setStatusMsg(`[VIKAS Linter] Applied quick-fix for ${diag.rule}: ${diag.title}`);
  };

  // Jump to specific line in editor
  const jumpToLine = (targetLine: number) => {
    const lineIndex = Math.max(0, targetLine - 1);
    let charOffset = 0;
    for (let i = 0; i < lineIndex && i < lines.length; i++) {
      charOffset += lines[i].length + 1;
    }
    if (textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.setSelectionRange(charOffset, charOffset + (lines[lineIndex]?.length || 0));
      // Approximate line height ~20px
      textareaRef.current.scrollTop = Math.max(0, (targetLine - 5) * 20);
      if (gutterRef.current) {
        gutterRef.current.scrollTop = textareaRef.current.scrollTop;
      }
    }
    setCursorLine(targetLine);
  };

  const handleVedCommand = async (e: React.FormEvent) => {
    e.preventDefault();
    const cmd = vedCmd.trim();
    if (cmd === ':w' || cmd === 'w') {
      await onSaveFile(activeFile.path, activeFile.content);
      setStatusMsg(`[VIKAS-VED] "${activeFile.path}" written (${lines.length}L)`);
    } else if (cmd === ':wq' || cmd === ':run' || cmd === 'run') {
      await onSaveFile(activeFile.path, activeFile.content);
      await onSimulateFile(activeFile.path, activeFile.content);
      setStatusMsg(`[VIKAS-VED] Saved & simulated "${activeFile.path}"`);
    } else if (cmd === ':patch' || cmd === 'patch') {
      await onAutoPatchLatches(activeFile.path, activeFile.content);
      setStatusMsg(`[VIKAS-VED] Applied L3 latch auto-patch`);
    }
    setVedCmd('');
    setVedMode('INSERT');
  };

  return (
    <div className="h-full w-full flex min-h-0 bg-[#090d16] text-slate-200 overflow-hidden font-mono text-xs">
      {/* Left Project File Tree Dock */}
      <aside className="w-68 bg-[#0d121f] border-r border-slate-800 flex flex-col min-h-0 shrink-0 select-none">
        <div className="flex items-center justify-between px-3 py-2 border-b border-slate-800">
          <span className="flex items-center gap-1.5 text-[11px] font-bold uppercase tracking-wider text-slate-400">
            <Code2 className="w-3.5 h-3.5 text-cyan-400" />
            RTL Project Files ({files.length})
          </span>
          <button
            type="button"
            onClick={() => setNewFileModal(true)}
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] bg-slate-800 hover:bg-slate-700 text-cyan-300 border border-slate-700 cursor-pointer"
          >
            <FilePlus className="w-3 h-3" />
            New
          </button>
        </div>

        <div className="p-2 border-b border-slate-800">
          <div className="relative">
            <Search className="w-3.5 h-3.5 text-slate-500 absolute left-2 top-2" />
            <input
              aria-label="Filter RTL files"
              value={fileFilter}
              onChange={(e) => setFileFilter(e.target.value)}
              placeholder="filter (nova_alu, latch...)"
              className="w-full bg-[#090d16] border border-slate-800 rounded pl-7 pr-2 py-1 text-xs font-mono text-slate-200 outline-none focus:border-cyan-500"
            />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5 font-mono text-xs">
          {filteredFiles.map((f) => {
            const isSelected = f.path === activeFile.path;
            const hasLatch = f.path.includes('latch_trap');
            const fileDiag = lintRtlSource(f.content, f.path);
            const errs = fileDiag.filter((d) => d.severity === 'error').length;
            const warns = fileDiag.filter((d) => d.severity === 'warning').length;

            return (
              <button
                key={f.path}
                type="button"
                onClick={() => setActiveFilePath(f.path)}
                className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-left truncate cursor-pointer transition-colors ${
                  isSelected
                    ? 'bg-cyan-950/70 text-cyan-300 font-semibold border-l-2 border-cyan-400'
                    : 'hover:bg-slate-800/60 text-slate-400 hover:text-slate-200'
                }`}
              >
                <FileCode className="w-3.5 h-3.5 shrink-0 text-cyan-400" />
                <span className="truncate flex-1">{f.path.replace(/^examples\//, '')}</span>
                {errs > 0 ? (
                  <span className="px-1.5 py-0.2 text-[9px] rounded bg-red-950/80 text-red-300 border border-red-500/50">
                    {errs} ERR
                  </span>
                ) : warns > 0 ? (
                  <span className="px-1.5 py-0.2 text-[9px] rounded bg-amber-950/80 text-amber-300 border border-amber-500/50">
                    {warns} WARN
                  </span>
                ) : hasLatch ? (
                  <span className="px-1.5 py-0.2 text-[9px] rounded bg-red-950/80 text-red-300 border border-red-500/50">
                    LATCH
                  </span>
                ) : null}
              </button>
            );
          })}
        </div>

        {/* 33 Supported Languages Quick Reference Badge */}
        <div className="p-2.5 border-t border-slate-800 bg-[#090d16] text-[10px] text-slate-400 space-y-1">
          <div className="flex items-center justify-between">
            <span className="font-bold text-slate-300">VIKAS Dialect Support</span>
            <span className="px-1.5 py-0.5 rounded bg-cyan-950 border border-cyan-500/40 text-cyan-300 font-bold">
              33 DSLs
            </span>
          </div>
          <div className="text-[10px] text-slate-500">
            SystemVerilog, VHDL, Chisel, BSV, SVA, UPF, Verilog-A, SPICE, PyMTL3, Amaranth...
          </div>
        </div>
      </aside>

      {/* Main VIKAS-VED IDE Center Area */}
      <div className="flex-1 flex flex-col min-w-0 h-full overflow-hidden">
        {/* Editor Top Control Toolbar */}
        <div className="flex items-center justify-between px-3 py-2 bg-[#0f1422] border-b border-slate-800 flex-wrap gap-2 shrink-0">
          <div className="flex items-center gap-2.5">
            <span className="font-mono text-xs font-bold text-slate-200">{activeFile.path}</span>
            <span
              onClick={() =>
                setVedMode((m) => (m === 'INSERT' ? 'NORMAL' : m === 'NORMAL' ? 'COMMAND' : 'INSERT'))
              }
              className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold cursor-pointer transition-colors ${
                vedMode === 'INSERT'
                  ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 hover:bg-cyan-500/30'
                  : vedMode === 'NORMAL'
                  ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 hover:bg-emerald-500/30'
                  : 'bg-amber-500/20 text-amber-300 border border-amber-500/40 hover:bg-amber-500/30'
              }`}
              title="Click to cycle VIKAS-VED Modal Editor state (INSERT / NORMAL / COMMAND)"
            >
              VIKAS-VED: {vedMode}
            </span>

            {/* Language Selector for Local AI Completion & Dialects */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] text-slate-400">Lang:</span>
              <select
                aria-label="Active Language Dialect"
                value={selectedLanguage}
                onChange={(e) => setSelectedLanguage(e.target.value)}
                className="bg-[#090d16] text-[11px] font-mono text-cyan-300 border border-slate-700 rounded px-1.5 py-0.5 outline-none cursor-pointer"
              >
                {SUPPORTED_EDA_LANGUAGES_CATALOG_33.map((lang) => (
                  <option key={lang.id} value={lang.name.split(' ')[0]}>
                    {lang.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Real-Time Linting Badge */}
            {errorCount > 0 ? (
              <span
                onClick={() => setDiagnosticsOpen(true)}
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-bold bg-red-950/80 text-red-300 border border-red-500/50 cursor-pointer"
              >
                <AlertCircle className="w-3.5 h-3.5 text-red-400" />
                {errorCount} Error{errorCount > 1 ? 's' : ''}
              </span>
            ) : null}
            {warningCount > 0 ? (
              <span
                onClick={() => setDiagnosticsOpen(true)}
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-bold bg-amber-950/80 text-amber-300 border border-amber-500/50 cursor-pointer"
              >
                <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                {warningCount} Warning{warningCount > 1 ? 's' : ''}
              </span>
            ) : null}
            {errorCount === 0 && warningCount === 0 ? (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-emerald-950/60 text-emerald-300 border border-emerald-500/40">
                <CheckCircle2 className="w-3.5 h-3.5" />
                Clean RTL
              </span>
            ) : null}
          </div>

          <div className="flex items-center gap-2">
            {/* View Mode Toggle */}
            <div className="flex rounded border border-slate-700 overflow-hidden text-[10px] font-mono">
              <button
                type="button"
                onClick={() => setViewMode('SPLIT')}
                className={`px-2 py-0.5 cursor-pointer ${
                  viewMode === 'SPLIT' ? 'bg-cyan-600 text-white font-bold' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                }`}
              >
                Split View
              </button>
              <button
                type="button"
                onClick={() => setViewMode('EDITOR_ONLY')}
                className={`px-2 py-0.5 cursor-pointer ${
                  viewMode === 'EDITOR_ONLY' ? 'bg-cyan-600 text-white font-bold' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                }`}
              >
                Editor Only
              </button>
              <button
                type="button"
                onClick={() => setViewMode('TRACE_ONLY')}
                className={`px-2 py-0.5 cursor-pointer ${
                  viewMode === 'TRACE_ONLY' ? 'bg-cyan-600 text-white font-bold' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                }`}
              >
                Trace Only
              </button>
            </div>

            {/* Local AI Auto-Wire Quick Menu */}
            <div className="relative">
              <button
                type="button"
                data-testid="ai-complete-trigger-btn"
                onClick={() => setQuickInsertMenuOpen((v) => !v)}
                className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs font-semibold bg-purple-950/70 hover:bg-purple-900/80 text-purple-200 border border-purple-500/50 cursor-pointer"
                title="Local In-Browser AI RTL Code Completion & Module Auto-Instantiator"
              >
                <Bot className="w-3.5 h-3.5 text-purple-400" />
                AI Complete
                <ChevronDown className="w-3 h-3 text-purple-400" />
              </button>

              {quickInsertMenuOpen && (
                <div
                  className="absolute right-0 top-full mt-1.5 w-76 rounded-lg bg-[#111624] border border-purple-500/40 shadow-2xl p-2 z-50 space-y-1"
                  onClick={() => setQuickInsertMenuOpen(false)}
                >
                  <div className="text-[10px] uppercase font-bold text-purple-300 px-2 py-1 flex items-center justify-between border-b border-slate-800">
                    <span>Local AI Code Assistant</span>
                    <span className="text-[9px] text-emerald-400">100% In-Browser</span>
                  </div>
                  <div className="text-[11px] text-slate-400 px-2 py-0.5">
                    Auto-wires ports directly to workspace signals:
                  </div>
                  <button
                    type="button"
                    onClick={() => {
                      const sugg = computeLocalAiRtlCompletions('nova_alu', activeFile.content, cursorPos, signals, kdb).find((s) => s.kind === 'module');
                      if (sugg) insertAiSuggestion(sugg);
                    }}
                    className="w-full text-left px-2 py-1.5 rounded hover:bg-slate-800 text-slate-200 text-xs flex items-center justify-between cursor-pointer"
                  >
                    <span>Auto-Wire <b>nova_alu</b></span>
                    <span className="text-[10px] text-cyan-400">Module</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      const sugg = computeLocalAiRtlCompletions('rv32i_core', activeFile.content, cursorPos, signals, kdb).find((s) => s.kind === 'module');
                      if (sugg) insertAiSuggestion(sugg);
                    }}
                    className="w-full text-left px-2 py-1.5 rounded hover:bg-slate-800 text-slate-200 text-xs flex items-center justify-between cursor-pointer"
                  >
                    <span>Auto-Wire <b>rv32i_core</b></span>
                    <span className="text-[10px] text-cyan-400">Module</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      const sugg = computeLocalAiRtlCompletions('axi_interconnect', activeFile.content, cursorPos, signals, kdb).find((s) => s.kind === 'module');
                      if (sugg) insertAiSuggestion(sugg);
                    }}
                    className="w-full text-left px-2 py-1.5 rounded hover:bg-slate-800 text-slate-200 text-xs flex items-center justify-between cursor-pointer"
                  >
                    <span>Auto-Wire <b>axi_interconnect</b></span>
                    <span className="text-[10px] text-cyan-400">Module</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      const sugg = computeLocalAiRtlCompletions('fsm_3state', activeFile.content, cursorPos, signals, kdb).find((s) => s.kind === 'snippet');
                      if (sugg) insertAiSuggestion(sugg);
                    }}
                    className="w-full text-left px-2 py-1.5 rounded hover:bg-slate-800 text-slate-200 text-xs flex items-center justify-between cursor-pointer"
                  >
                    <span>Insert <b>Glitch-Free FSM (3-State)</b></span>
                    <span className="text-[10px] text-emerald-400">Snippet</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      const sugg = computeLocalAiRtlCompletions('sva_handshake', activeFile.content, cursorPos, signals, kdb).find((s) => s.kind === 'snippet');
                      if (sugg) insertAiSuggestion(sugg);
                    }}
                    className="w-full text-left px-2 py-1.5 rounded hover:bg-slate-800 text-slate-200 text-xs flex items-center justify-between cursor-pointer"
                  >
                    <span>Insert <b>SVA Handshake Assertion</b></span>
                    <span className="text-[10px] text-purple-400">SVA</span>
                  </button>
                </div>
              )}
            </div>

            {liveLatches.length > 0 && (
              <button
                type="button"
                onClick={async () => {
                  await onAutoPatchLatches(activeFile.path, activeFile.content);
                  setStatusMsg(`Eliminated ${liveLatches.length} inferred latch(es) with SystemVerilog default initialization!`);
                }}
                className="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs font-semibold bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-400/50 cursor-pointer"
              >
                <Wand2 className="w-3.5 h-3.5" />
                Apply L3 Auto-Patch
              </button>
            )}

            <button
              type="button"
              onClick={async () => {
                await onSaveFile(activeFile.path, activeFile.content);
                setStatusMsg(`Saved ${activeFile.path} to workspace`);
              }}
              className="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 cursor-pointer"
            >
              <Save className="w-3.5 h-3.5 text-cyan-400" />
              Save (:w)
            </button>
            <button
              type="button"
              onClick={async () => {
                await onSaveFile(activeFile.path, activeFile.content);
                await onSimulateFile(activeFile.path, activeFile.content);
              }}
              className="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs font-semibold bg-cyan-500 hover:bg-cyan-400 text-slate-950 cursor-pointer"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              Simulate Live
            </button>
          </div>
        </div>

        {/* Latch Alert Banner if latches detected */}
        {liveLatches.length > 0 && (
          <div className="bg-red-950/60 border-b border-red-500/40 px-4 py-2 flex items-start justify-between gap-4 shrink-0">
            <div className="space-y-0.5 text-xs">
              <div className="font-bold text-red-300 flex items-center gap-1.5">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
                3-Layer Inferred Latch Detected in `{activeFile.path}`
              </div>
              {liveLatches.slice(0, 2).map((l, i) => (
                <div key={i} className="font-mono text-[11px] text-red-200/90">
                  • Line {l.line}: signal <b className="text-white">{l.signal}</b> ({l.missing_branch_hint}) → Shadow Track:{' '}
                  <span className="text-amber-300">VIKAS_CAUSALITY_latch_hold_{l.signal}</span>
                </div>
              ))}
            </div>
            <button
              type="button"
              onClick={() => onAutoPatchLatches(activeFile.path, activeFile.content)}
              className="shrink-0 px-3 py-1 rounded bg-red-500 hover:bg-red-400 text-slate-950 font-bold text-xs cursor-pointer"
            >
              Fix Latch Now
            </button>
          </div>
        )}

        {/* Split Editor + Live Annotated View with Sync Scroll */}
        <div className="flex-1 min-h-0 flex overflow-hidden divide-x divide-slate-800">
          {/* Editable Code Buffer with Gutter & Non-Blocking Lint Markers */}
          {(viewMode === 'SPLIT' || viewMode === 'EDITOR_ONLY') && (
            <div className="flex-1 flex flex-col min-h-0 bg-[#090d16] relative">
              <div className="px-3 py-1 bg-[#0d121f] border-b border-slate-800 text-[10.5px] font-mono text-slate-400 flex justify-between items-center shrink-0">
                <span>
                  EDIT BUFFER ({lines.length} lines · Ln {cursorLine}, Col {cursorCol})
                </span>
                <span className="text-slate-500">
                  Ctrl+Space for AI Suggestions · UTF-8 · {selectedLanguage}
                </span>
              </div>

              {/* Editor Body with Gutter */}
              <div className="flex-1 min-h-0 flex relative overflow-hidden">
                {/* Gutter: Line Numbers + Non-Blocking Violation Markers */}
                <div
                  ref={gutterRef}
                  className="w-14 bg-[#0d121f] border-r border-slate-800 flex flex-col select-none overflow-hidden shrink-0 text-right py-3 pr-2"
                >
                  {lines.map((_, i) => {
                    const lineNum = i + 1;
                    const diagsOnLine = lineDiagMap.get(lineNum);
                    const hasError = diagsOnLine?.some((d) => d.severity === 'error');
                    const hasWarning = diagsOnLine?.some((d) => d.severity === 'warning');
                    const isCursorHere = cursorLine === lineNum;

                    return (
                      <div
                        key={i}
                        className={`h-5 leading-5 flex items-center justify-end gap-1.5 ${
                          isCursorHere ? 'text-cyan-300 font-bold' : 'text-slate-600'
                        }`}
                      >
                        {/* Non-Blocking Gutter Marker */}
                        {hasError ? (
                          <span
                            onClick={() => setActiveDiagTooltip(diagsOnLine?.[0] || null)}
                            className="w-3.5 h-3.5 rounded-full bg-red-500/80 hover:bg-red-400 text-[9px] font-bold text-white flex items-center justify-center cursor-pointer shadow"
                            title={`Line ${lineNum}: ${diagsOnLine?.[0]?.message}`}
                          >
                            !
                          </span>
                        ) : hasWarning ? (
                          <span
                            onClick={() => setActiveDiagTooltip(diagsOnLine?.[0] || null)}
                            className="w-3.5 h-3.5 rounded bg-amber-500/80 hover:bg-amber-400 text-[8px] font-bold text-slate-950 flex items-center justify-center cursor-pointer"
                            title={`Line ${lineNum}: ${diagsOnLine?.[0]?.message}`}
                          >
                            ▲
                          </span>
                        ) : null}

                        <span className="text-[10.5px] font-mono">{lineNum}</span>
                      </div>
                    );
                  })}
                </div>

                {/* Textarea with Synchronized Scroll */}
                <div className="flex-1 min-h-0 relative h-full">
                  <textarea
                    ref={textareaRef}
                    aria-label="SystemVerilog RTL Code Editor"
                    value={activeFile.content}
                    onChange={(e) => onUpdateFileContent(activeFile.path, e.target.value)}
                    onScroll={handleEditorScroll}
                    onClick={updateCursorTracking}
                    onKeyUp={updateCursorTracking}
                    onSelect={updateCursorTracking}
                    onKeyDown={handleKeyDown}
                    spellCheck={false}
                    className="w-full h-full p-3 bg-transparent font-mono text-xs leading-5 text-slate-200 outline-none resize-none select-text whitespace-pre overflow-y-auto"
                  />

                  {/* Diagnostic Tooltip Popover if clicked on gutter */}
                  {activeDiagTooltip && (
                    <div
                      className="absolute z-40 max-w-md bg-[#131929] border border-red-500/50 rounded-lg p-3 shadow-2xl space-y-2 text-xs"
                      style={{
                        top: Math.min(250, Math.max(20, (activeDiagTooltip.line - 1) * 20 - (textareaRef.current?.scrollTop || 0))),
                        left: 20,
                      }}
                    >
                      <div className="flex items-center justify-between border-b border-slate-700/60 pb-1.5">
                        <span className="font-bold text-red-300 flex items-center gap-1.5">
                          <AlertTriangle className="w-3.5 h-3.5 text-red-400" />
                          [{activeDiagTooltip.rule}] {activeDiagTooltip.title}
                        </span>
                        <button
                          type="button"
                          onClick={() => setActiveDiagTooltip(null)}
                          className="text-slate-400 hover:text-white"
                        >
                          ✕
                        </button>
                      </div>
                      <p className="text-slate-300 text-[11px] leading-relaxed">
                        {activeDiagTooltip.message}
                      </p>
                      {activeDiagTooltip.quickFix && (
                        <div className="pt-1 flex justify-end">
                          <button
                            type="button"
                            onClick={() => applyQuickFix(activeDiagTooltip)}
                            className="px-2.5 py-1 rounded bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-[11px] cursor-pointer flex items-center gap-1"
                          >
                            <Wand2 className="w-3 h-3" />
                            {activeDiagTooltip.quickFix.title}
                          </button>
                        </div>
                      )}
                    </div>
                  )}

                  {/* Local AI Code Completion Floating Suggestions Menu */}
                  {(aiSuggestionsOpen || (currentTokenPrefix.length >= 2 && aiSuggestions.length > 0)) && (
                    <div
                      data-testid="local-ai-completion-dropdown"
                      className="absolute z-50 w-96 rounded-lg bg-[#111726] border border-cyan-500/50 shadow-2xl p-2 space-y-1 text-xs"
                      style={{
                        top: Math.min(320, Math.max(20, cursorLine * 20 - (textareaRef.current?.scrollTop || 0) + 10)),
                        left: Math.min(300, Math.max(20, cursorCol * 7)),
                      }}
                    >
                      <div className="flex items-center justify-between px-2 py-0.5 border-b border-slate-800 text-[10px] text-cyan-300 font-bold uppercase">
                        <span className="flex items-center gap-1">
                          <Bot className="w-3 h-3 text-cyan-400" />
                          Local AI Suggestions ({aiSuggestions.length})
                        </span>
                        <span className="text-slate-500">Tab / Enter to Insert</span>
                      </div>

                      <div className="max-h-56 overflow-y-auto space-y-0.5 font-mono">
                        {aiSuggestions.map((sug, idx) => {
                          const isSel = idx === activeSuggestionIdx;
                          return (
                            <button
                              key={`${sug.label}-${idx}`}
                              type="button"
                              onClick={() => insertAiSuggestion(sug)}
                              className={`w-full text-left px-2 py-1.5 rounded flex items-center justify-between cursor-pointer transition-colors ${
                                isSel
                                  ? 'bg-cyan-950/80 text-cyan-200 border-l-2 border-cyan-400'
                                  : 'hover:bg-slate-800 text-slate-300'
                              }`}
                            >
                              <div className="truncate flex-1">
                                <span className="font-bold text-white">{sug.label}</span>
                                <div className="text-[10px] text-slate-400 truncate">{sug.detail}</div>
                              </div>
                              <span
                                className={`ml-2 text-[9px] px-1.5 py-0.2 rounded font-bold uppercase shrink-0 ${
                                  sug.kind === 'module'
                                    ? 'bg-purple-950 text-purple-300 border border-purple-500/40'
                                    : sug.kind === 'signal'
                                    ? 'bg-cyan-950 text-cyan-300 border border-cyan-500/40'
                                    : sug.kind === 'snippet'
                                    ? 'bg-emerald-950 text-emerald-300 border border-emerald-500/40'
                                    : 'bg-slate-800 text-slate-400'
                                }`}
                              >
                                {sug.kind}
                              </span>
                            </button>
                          );
                        })}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Right Live Inline Value Annotator View */}
          {(viewMode === 'SPLIT' || viewMode === 'TRACE_ONLY') && (
            <div className="flex-1 flex flex-col min-h-0 bg-[#090d16]">
              <div className="px-3 py-1 bg-[#0d121f] border-b border-slate-800 text-[10.5px] font-mono text-cyan-300 flex justify-between items-center shrink-0">
                <span className="flex items-center gap-1.5">
                  <Sparkles className="w-3 h-3 text-cyan-400" />
                  LIVE RTL VALUE ANNOTATOR (VIKAS-VED Live Trace @ t = {cursorTimeNs} ns)
                </span>
                <span className="text-slate-500">Feature #15</span>
              </div>

              <div
                ref={traceGutterRef}
                className="flex-1 overflow-y-auto p-3 font-mono text-xs space-y-0.5 select-text"
              >
                {lines.map((line, i) => {
                  const lineNum = i + 1;
                  const latchOnLine = liveLatches.find((l) => l.line === lineNum);
                  const diagsOnLine = lineDiagMap.get(lineNum);
                  const hasError = diagsOnLine?.some((d) => d.severity === 'error');
                  const hasWarn = diagsOnLine?.some((d) => d.severity === 'warning');

                  // Extract identifiers on this line that have live signal values
                  const tokens = [...new Set(line.replace(/\/\/.*$/, '').match(/\b[A-Za-z_]\w*\b/g) || [])];
                  const annots = tokens
                    .filter((tk) => inlineAnnotations.has(tk))
                    .slice(0, 4)
                    .map((tk) => `${tk}=${inlineAnnotations.get(tk)}`);

                  return (
                    <div
                      key={i}
                      className={`flex items-baseline gap-3 px-2 py-0.5 rounded transition-colors ${
                        hasError
                          ? 'bg-red-950/30 border-l-2 border-red-500'
                          : hasWarn
                          ? 'bg-amber-950/20 border-l-2 border-amber-500'
                          : latchOnLine
                          ? 'bg-red-500/15 border-l-2 border-red-400'
                          : annots.length > 0
                          ? 'hover:bg-slate-800/50'
                          : ''
                      }`}
                    >
                      <span className="w-8 text-right text-[10px] text-slate-600 select-none shrink-0">
                        {lineNum}
                      </span>
                      <pre className="text-slate-300 whitespace-pre-wrap break-all flex-1">{line || ' '}</pre>
                      {annots.length > 0 && (
                        <span className="shrink-0 text-[10px] px-1.5 py-0.2 rounded bg-cyan-950/80 text-cyan-300 border border-cyan-500/40">
                          [{annots.join(' · ')}]
                        </span>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>

        {/* Collapsible Problems / Diagnostics Dock (Real-Time Gutter Service Companion) */}
        {diagnosticsOpen && lintDiagnostics.length > 0 && (
          <div className="h-36 bg-[#0c101c] border-t border-slate-800 flex flex-col shrink-0 select-none">
            <div className="flex items-center justify-between px-3 py-1.5 bg-[#0e1424] border-b border-slate-800 text-[11px] font-bold">
              <span className="flex items-center gap-2 text-slate-300">
                <ShieldAlert className="w-3.5 h-3.5 text-cyan-400" />
                PROBLEMS / LINT ({lintDiagnostics.length} issue{lintDiagnostics.length > 1 ? 's' : ''})
              </span>
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-red-300 font-normal">
                  {errorCount} Error{errorCount > 1 ? 's' : ''}
                </span>
                <span className="text-[10px] text-amber-300 font-normal">
                  {warningCount} Warning{warningCount > 1 ? 's' : ''}
                </span>
                <button
                  type="button"
                  onClick={() => setDiagnosticsOpen(false)}
                  className="p-0.5 text-slate-400 hover:text-white cursor-pointer"
                  title="Close Problems Drawer"
                >
                  <Minimize2 className="w-3 h-3" />
                </button>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto p-2 space-y-1 font-mono text-xs">
              {lintDiagnostics.map((diag) => (
                <div
                  key={diag.id}
                  className="flex items-center justify-between gap-3 px-2 py-1 rounded bg-[#111726] border border-slate-800/80 hover:border-slate-700 transition-colors"
                >
                  <div
                    onClick={() => jumpToLine(diag.line)}
                    className="flex items-center gap-2 flex-1 min-w-0 cursor-pointer"
                  >
                    <span
                      className={`px-1.5 py-0.2 rounded text-[9px] font-bold shrink-0 ${
                        diag.severity === 'error'
                          ? 'bg-red-950 text-red-300 border border-red-500/40'
                          : 'bg-amber-950 text-amber-300 border border-amber-500/40'
                      }`}
                    >
                      {diag.rule}
                    </span>
                    <span className="text-[11px] text-slate-400 shrink-0">
                      Line {diag.line}:{diag.col}
                    </span>
                    <span className="text-slate-200 text-xs truncate">{diag.message}</span>
                  </div>

                  {diag.quickFix && (
                    <button
                      type="button"
                      onClick={() => applyQuickFix(diag)}
                      className="shrink-0 px-2 py-0.5 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/40 text-[10px] font-bold cursor-pointer flex items-center gap-1"
                    >
                      <Wand2 className="w-3 h-3" />
                      Quick Fix
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Bottom VIKAS-VED Command Bar & Status Line */}
        <form
          onSubmit={handleVedCommand}
          className="h-8 bg-[#0c101c] border-t border-slate-800 px-3 flex items-center justify-between gap-3 font-mono text-xs shrink-0"
        >
          <div className="flex items-center gap-3">
            <span className="text-cyan-400 font-bold">VIKAS-VED</span>
            <input
              aria-label="VIKAS-VED modal command input"
              value={vedCmd}
              onChange={(e) => setVedCmd(e.target.value)}
              placeholder=":w (save) · :run (simulate) · :patch (fix latches)"
              className="bg-[#090d16] border border-slate-800 rounded px-2 py-0.5 text-xs text-slate-200 outline-none w-80 focus:border-cyan-500"
            />
            {statusMsg && <span className="text-emerald-400 text-[11px] truncate">{statusMsg}</span>}
          </div>

          <div className="flex items-center gap-3 text-[11px] text-slate-400">
            {!diagnosticsOpen && lintDiagnostics.length > 0 && (
              <button
                type="button"
                onClick={() => setDiagnosticsOpen(true)}
                className="text-amber-400 hover:underline flex items-center gap-1"
              >
                <AlertTriangle className="w-3 h-3" />
                {lintDiagnostics.length} Issues
              </button>
            )}
            <span>Ln {cursorLine}, Col {cursorCol}</span>
            <span>{lines.length} Lines</span>
          </div>
        </form>
      </div>

      {/* Create New File Modal */}
      {newFileModal && (
        <div className="fixed inset-0 z-50 bg-black/65 flex items-center justify-center p-4">
          <div className="bg-[#111624] border border-slate-700 rounded-xl max-w-md w-full p-5 space-y-4 shadow-2xl">
            <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
              <FilePlus className="w-4 h-4 text-cyan-400" />
              Create New SystemVerilog Module
            </h3>
            <div className="space-y-1.5">
              <label className="text-xs text-slate-400 block">File Path (inside examples/)</label>
              <input
                aria-label="New file path"
                value={newFileName}
                onChange={(e) => setNewFileName(e.target.value)}
                className="w-full bg-[#090d16] border border-slate-700 rounded px-3 py-1.5 font-mono text-xs text-slate-200 outline-none"
              />
            </div>
            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setNewFileModal(false)}
                className="px-3 py-1.5 rounded text-xs bg-slate-800 text-slate-300 hover:text-white cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => {
                  const safePath = newFileName.startsWith('examples/')
                    ? newFileName
                    : `examples/${newFileName}`;
                  const modName =
                    safePath
                      .split('/')
                      .pop()
                      ?.replace(/\.\w+$/, '') || 'custom_mod';
                  const template = `// Created in VIKAS-VED Studio IDE (Iteration 10 Sovereign EDA)\nmodule ${modName}(\n    input  logic        clk,\n    input  logic        rst_n,\n    input  logic [15:0] a,\n    input  logic [15:0] b,\n    output logic [15:0] y\n);\n    always_ff @(posedge clk or negedge rst_n) begin\n        if (!rst_n) begin\n            y <= 16'd0;\n        end else begin\n            y <= a + b;\n        end\n    end\nendmodule\n`;
                  onCreateNewFile(safePath, template);
                  setNewFileModal(false);
                }}
                className="px-3.5 py-1.5 rounded text-xs font-semibold bg-cyan-500 text-slate-950 cursor-pointer"
              >
                Create & Open
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
