import React, { useMemo, useState } from 'react';
import {
  AlertTriangle,
  CheckCircle2,
  Code2,
  FileCode,
  FilePlus,
  Play,
  Save,
  ShieldAlert,
  Sparkles,
  Wand2,
} from 'lucide-react';
import {
  analyzeLatches,
  formatSignalValue,
  SignalMeta,
  SignalSteps,
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
}) => {
  const [vimMode, setVimMode] = useState<'INSERT' | 'NORMAL' | 'COMMAND'>('INSERT');
  const [vimCmd, setVimCmd] = useState('');
  const [fileFilter, setFileFilter] = useState('');
  const [newFileModal, setNewFileModal] = useState(false);
  const [newFileName, setNewFileName] = useState('examples/custom_alu.sv');
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const activeFile = useMemo(
    () => files.find((f) => f.path === activeFilePath) || files[0] || { path: 'examples/demo_latch_trap.sv', content: '' },
    [files, activeFilePath]
  );

  // Live L1 static latch audit on the currently open buffer
  const liveLatches = useMemo(
    () => analyzeLatches([{ path: activeFile.path, content: activeFile.content }]),
    [activeFile.path, activeFile.content]
  );

  // Inline live signal values at Cursor A timestamp (Feature 15: Verdi nTrace inline annotator)
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

  const lines = useMemo(() => activeFile.content.split('\n'), [activeFile.content]);

  const handleVimCommand = async (e: React.FormEvent) => {
    e.preventDefault();
    const cmd = vimCmd.trim();
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
    setVimCmd('');
    setVimMode('INSERT');
  };

  return (
    <div className="flex flex-1 min-h-0 bg-[var(--bg)]">
      {/* Left Project File Tree */}
      <aside className="w-72 bg-[var(--panel)] border-r border-[var(--edge)] flex flex-col min-h-0 shrink-0">
        <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--edge)]">
          <span className="flex items-center gap-1.5 text-[11px] font-bold uppercase tracking-wider text-[var(--dim)]">
            <Code2 className="w-3.5 h-3.5 text-[var(--accent)]" />
            RTL Project Files ({files.length})
          </span>
          <button
            type="button"
            onClick={() => setNewFileModal(true)}
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--accent)] border border-[var(--edge2)] cursor-pointer"
          >
            <FilePlus className="w-3 h-3" />
            New File
          </button>
        </div>

        <div className="p-2 border-b border-[var(--edge)]">
          <input
            aria-label="Filter RTL files"
            value={fileFilter}
            onChange={(e) => setFileFilter(e.target.value)}
            placeholder="filter files (nova_alu, latch...)"
            className="w-full bg-[var(--bg2)] border border-[var(--edge2)] rounded px-2.5 py-1 text-xs font-mono text-[var(--txt)] outline-none focus:border-[var(--accent)]"
          />
        </div>

        <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5 font-mono text-xs">
          {filteredFiles.map((f) => {
            const isSelected = f.path === activeFile.path;
            const hasLatch = f.path.includes('latch_trap');
            return (
              <button
                key={f.path}
                type="button"
                onClick={() => setActiveFilePath(f.path)}
                className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-left truncate cursor-pointer ${
                  isSelected
                    ? 'bg-[var(--sel)] text-[var(--accent)] font-semibold border-l-2 border-[var(--accent)]'
                    : 'hover:bg-[var(--raised)] text-[var(--dim)]'
                }`}
              >
                <FileCode className="w-3.5 h-3.5 shrink-0 text-[var(--accent2)]" />
                <span className="truncate flex-1">{f.path.replace(/^examples\//, '')}</span>
                {hasLatch && (
                  <span className="px-1.5 py-0.2 text-[9px] rounded bg-[var(--bad)]/20 text-[var(--bad)] border border-[var(--bad)]/40">
                    LATCH
                  </span>
                )}
              </button>
            );
          })}
        </div>
      </aside>

      {/* Center VIKAS-VED Editor + Live Value & Latch Annotator */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Editor Toolbar */}
        <div className="flex items-center justify-between px-4 py-2 bg-[var(--panel)] border-b border-[var(--edge)] flex-wrap gap-2">
          <div className="flex items-center gap-2.5">
            <span className="font-mono text-xs font-bold text-[var(--txt)]">{activeFile.path}</span>
            <span
              onClick={() =>
                setVimMode((m) => (m === 'INSERT' ? 'NORMAL' : m === 'NORMAL' ? 'COMMAND' : 'INSERT'))
              }
              className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold cursor-pointer ${
                vimMode === 'INSERT'
                  ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'
                  : vimMode === 'NORMAL'
                  ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40'
                  : 'bg-amber-500/20 text-amber-300 border border-amber-500/40'
              }`}
              title="Click to cycle VIKAS-VED Modal Editor state (INSERT / NORMAL / COMMAND)"
            >
              VIKAS-VED: {vimMode}
            </span>
            {liveLatches.length > 0 ? (
              <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded text-xs font-bold bg-red-500/20 text-red-300 border border-red-500/40">
                <ShieldAlert className="w-3.5 h-3.5" />
                [VIKAS LATCH ALERT] {liveLatches.length} Inferred Latch(es)
              </span>
            ) : (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-emerald-500/15 text-emerald-300 border border-emerald-500/30">
                <CheckCircle2 className="w-3.5 h-3.5" />
                L1/L2/L3 Latch-Clean
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
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
              className="inline-flex items-center gap-1.5 px-3 py-1 rounded text-xs bg-[var(--panel2)] hover:bg-[var(--raised)] text-[var(--txt)] border border-[var(--edge2)] cursor-pointer"
            >
              <Save className="w-3.5 h-3.5 text-[var(--accent)]" />
              Save File (:w)
            </button>
            <button
              type="button"
              onClick={async () => {
                await onSaveFile(activeFile.path, activeFile.content);
                await onSimulateFile(activeFile.path, activeFile.content);
              }}
              className="inline-flex items-center gap-1.5 px-3.5 py-1 rounded text-xs font-semibold bg-cyan-500 hover:bg-cyan-400 text-slate-950 cursor-pointer"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              Compile & Simulate Live
            </button>
          </div>
        </div>

        {/* Latch Alert Banner if latches detected */}
        {liveLatches.length > 0 && (
          <div className="bg-red-950/60 border-b border-red-500/40 px-4 py-2.5 flex items-start justify-between gap-4">
            <div className="space-y-1 text-xs">
              <div className="font-bold text-red-300 flex items-center gap-1.5">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
                BREAKTHROUGH #13 — 3-Layer Inferred Latch Detected in `{activeFile.path}`
              </div>
              {liveLatches.map((l, i) => (
                <div key={i} className="font-mono text-[11px] text-red-200/90">
                  • Line {l.line}: signal <b className="text-white">{l.signal}</b> inside `{l.always_block_type}` ({l.missing_branch_hint}) → Shadow Track:{' '}
                  <span className="text-amber-300">VIKAS_CAUSALITY_latch_hold_{l.signal}</span>
                </div>
              ))}
            </div>
            <button
              type="button"
              onClick={() => onAutoPatchLatches(activeFile.path, activeFile.content)}
              className="shrink-0 px-3 py-1.5 rounded bg-red-500 hover:bg-red-400 text-slate-950 font-bold text-xs cursor-pointer"
            >
              Fix Latch Now
            </button>
          </div>
        )}

        {/* Split Editor + Live Annotated View (Feature 15: Verdi nTrace inline values) */}
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 min-h-0 divide-x divide-[var(--edge)]">
          {/* Editable Code Buffer */}
          <div className="flex flex-col min-h-0 bg-[#090d16]">
            <div className="px-3 py-1 bg-[var(--panel2)] border-b border-[var(--edge)] text-[10.5px] font-mono text-[var(--dim)] flex justify-between">
              <span>EDIT BUFFER ({lines.length} lines)</span>
              <span>UTF-8 · SystemVerilog 2017</span>
            </div>
            <textarea
              aria-label="SystemVerilog RTL Code Editor"
              value={activeFile.content}
              onChange={(e) => onUpdateFileContent(activeFile.path, e.target.value)}
              spellCheck={false}
              className="flex-1 w-full p-4 bg-transparent font-mono text-xs leading-relaxed text-[var(--txt)] outline-none resize-none select-text"
            />
          </div>

          {/* Live Inline Value Annotator (`vikas annotate --time <curA>`) */}
          <div className="flex flex-col min-h-0 bg-[var(--bg)]">
            <div className="px-3 py-1 bg-[var(--panel2)] border-b border-[var(--edge)] text-[10.5px] font-mono text-[var(--accent)] flex justify-between items-center">
              <span className="flex items-center gap-1.5">
                <Sparkles className="w-3 h-3" />
                LIVE RTL VALUE ANNOTATOR (Verdi nTrace @ t = {cursorTimeNs} ns)
              </span>
              <span className="text-[var(--dim)]">Feature #15</span>
            </div>
            <div className="flex-1 overflow-y-auto p-3 font-mono text-xs space-y-0.5 select-text">
              {lines.map((line, i) => {
                const lineNum = i + 1;
                const latchOnLine = liveLatches.find((l) => l.line === lineNum);
                // Extract identifiers on this line that have live signal values
                const tokens = [...new Set(line.replace(/\/\/.*$/, '').match(/\b[A-Za-z_]\w*\b/g) || [])];
                const annots = tokens
                  .filter((tk) => inlineAnnotations.has(tk))
                  .slice(0, 4)
                  .map((tk) => `${tk}=${inlineAnnotations.get(tk)}`);

                return (
                  <div
                    key={i}
                    className={`flex items-baseline gap-3 px-2 py-0.5 rounded ${
                      latchOnLine
                        ? 'bg-red-500/15 border-l-2 border-red-400'
                        : annots.length > 0
                        ? 'hover:bg-[var(--raised)]/60'
                        : ''
                    }`}
                  >
                    <span className="w-8 text-right text-[10px] text-[var(--faint)] select-none shrink-0">
                      {lineNum}
                    </span>
                    <pre className="text-[var(--txt)] whitespace-pre-wrap break-all flex-1">{line || ' '}</pre>
                    {annots.length > 0 && (
                      <span className="shrink-0 text-[10px] px-1.5 py-0.2 rounded bg-cyan-500/15 text-cyan-300 border border-cyan-500/30">
                        [{annots.join(' · ')}]
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* VIM Command Bar & Status */}
        <form
          onSubmit={handleVimCommand}
          className="h-8 bg-[var(--panel)] border-t border-[var(--edge)] px-3 flex items-center gap-3 font-mono text-xs"
        >
          <span className="text-[var(--accent)] font-bold">VIKAS-VED</span>
          <input
            aria-label="Vim command input"
            value={vimCmd}
            onChange={(e) => setVimCmd(e.target.value)}
            placeholder=":w (save) · :run (simulate) · :patch (fix inferred latches)"
            className="bg-[var(--bg2)] border border-[var(--edge)] rounded px-2 py-0.5 text-xs text-[var(--txt)] outline-none w-80"
          />
          {statusMsg && <span className="text-emerald-400 text-[11px]">{statusMsg}</span>}
        </form>
      </div>

      {/* Create New File Modal */}
      {newFileModal && (
        <div className="fixed inset-0 z-50 bg-black/65 flex items-center justify-center p-4">
          <div className="bg-[var(--panel2)] border border-[var(--edge2)] rounded-xl max-w-md w-full p-5 space-y-4 shadow-2xl">
            <h3 className="text-sm font-bold text-[var(--txt)] flex items-center gap-2">
              <FilePlus className="w-4 h-4 text-[var(--accent)]" />
              Create New SystemVerilog Module
            </h3>
            <div className="space-y-1.5">
              <label className="text-xs text-[var(--dim)] block">File Path (inside examples/)</label>
              <input
                aria-label="New file path"
                value={newFileName}
                onChange={(e) => setNewFileName(e.target.value)}
                className="w-full bg-[var(--bg2)] border border-[var(--edge2)] rounded px-3 py-1.5 font-mono text-xs text-[var(--txt)] outline-none"
              />
            </div>
            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setNewFileModal(false)}
                className="px-3 py-1.5 rounded text-xs bg-[var(--panel)] text-[var(--dim)] hover:text-[var(--txt)] cursor-pointer"
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
                  const template = `// Created in VIKAS-VED Studio IDE\nmodule ${modName}(\n    input  logic        clk,\n    input  logic        rst_n,\n    input  logic [15:0] a,\n    input  logic [15:0] b,\n    output logic [15:0] y\n);\n    always_ff @(posedge clk) begin\n        if (!rst_n) y <= 16'd0;\n        else        y <= a + b;\n    end\nendmodule\n`;
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
