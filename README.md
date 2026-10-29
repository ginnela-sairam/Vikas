# VIKAS v1.1 — Post-Duopoly Linux-Native RTL EDA Platform

**VIKAS** = XEZIM (**v1.1 topo-scheduled event kernel**, SystemVerilog/Verilog,
100% Rust) + **VIKAS STUDIO** (professional single-file HTML5 EDA workbench:
menus · 40-icon toolbar · dock panels · minimap · 4 themes · command palette)
+ Open-KDB (CSR design-graph intelligence) + Silicon-Physics twin (C++20 RK4)
+ native waveform engine with **6 innovative formats** + client-side VCD/VWF
file loading + AI-native MCP server. Zero Python in the core. Built to
leapfrog the Synopsys/Cadence duopoly as open source.

```bash
./build_linux.sh            # one command: build + test + smoke-check
./install_linux.sh          # installs to ~/.local/bin + /dev/shm arena
```

## Quick start

```bash
alias vikas="$PWD/target/release/vikas"

# zero-touch simulation: compile RTL -> auto-dump -> Open-KDB -> telemetry
vikas run examples/demo_riscv_alu_hazard.sv

# MULTI-FILE projects (NEW): industry filelist manifests + `include + UVM-aware
vikas run examples/multifile_proj/project.f
#    -> regfile/alu/soc_top across 5 files, 2 include headers, result=200 exact

# BREAKTHROUGH 13: 3-layer inferred latch detection
vikas latch examples/demo_latch_trap.sv        # exit 2 when latches found

# 3 NEW BREAKTHROUGHS (this build): mutation testing, switching power, one-command signoff
vikas mutant examples/multifile_proj/project.f --min-score 70
#    -> injects op-swaps/const mutations into the DUT, re-sims each against
#       golden, scores what your TB kills; survivors = precise TB-gap report

vikas power .vikas/vikas_wave.vwf --vdd 0.8
#    -> P = Σ ½CV²·f_tr from REAL toggle counts: hotspot ranking + clock-tree share

vikas signoff examples/nova_soc/nova.f
#    -> latch + loops + CDC + coverage + liveness + TB verdict (+ --mutant)
#       in ONE weighted tapeout-risk score; exit 0 READY / 1 AT RISK / 2 BLOCKED

# NOVA-SoC — the deep multi-file validation vehicle (26 files, 5 levels)
vikas run examples/nova_soc/nova.f        # NOVA-PASS 9/9 golden checks
vikas run examples/nova_soc/docs/manifest.files   # alternate .files manifest: same 9/9

# VIKAS STUDIO — the professional single-file HTML5 workbench
vikas run examples/nova_soc/nova.f --gui       # run + embed KDB/physics/verdict
vikas gui .vikas/vikas_wave.vwf                # from any dump (auto-detects KDB)
vikas studio examples/nova_soc/nova.f -o nova.html --no-open
#    -> one-shot studio: simulate + KDB + latch audit -> HTML workbench
#       (headless-safe: --no-open is the CI/pipeline mode)

# environment doctor (breakthrough 25): zero-arg self-check of the host
vikas doctor          # toolchain + wave-engine round-trip + parser sanity -> exit 0
vikas doctor examples/demo_latch_trap.sv       # with files: RTL root-cause bundle

#     -> File > Open also loads ANY .vcd (VCS/Icarus/Verilator dumps) and
#        native .vwf — parsed entirely in your browser, zero server, zero net

# kernel throughput suite (v1.1)
vikas bench --scale 3                          # comb/NBA/FIFO/SoC stress + events/s
#     -> qa_results/bench.json for CI tracking

# interactive TUI (branded, cursor mode, hex bus lanes — real PTY verified)
vikas tui .vikas/vikas_wave.vwf

vikas help                                     # full 59-command catalog
```

## VIKAS STUDIO — professional HTML5 workbench (verified in Chromium)

One self-contained offline HTML file (130 KB at demo scale, 283 KB at
455-signal NOVA scale) that renders like a commercial tool, not a science
project. VLM-audited against Verdi/GTKWave/ModelSim: **9/10 waveform view,
9/10 analysis hub, 8/10 design intel, zero critical visual bugs.**

**Application chrome** — title bar with live status chips + PASS/LATCH-RISK
verdict badge · menu bar (File Edit View Signals Analysis Help, all wired) ·
40-icon SVG toolbar (lucide-grade strokes, grouped, hover-tooltipped) ·
signal-tree dock with scope collapse + regex filter + one-click add ·
properties dock (activity stats, values at A/B, glitch count, quick actions) ·
console dock · status bar (t, cursors, Δt + MHz, zoom, events) · toast
notifications · **command palette (Ctrl+K)** with 37 fuzzy-searchable commands.

**Waveform engine** — value-at-cursor column · signal groups (G/U) · bus
expansion into per-bit lanes (E) · analog traces for wide buses (A) ·
industry-standard X-hatch + Z-dash 4-state rendering · glitch diamonds ·
hexagon bus transitions with value-in-segment labels · dual cursors A/B with
Δt + frequency HUD · marquee zoom (drag the time axis) · wheel zoom-at-pointer
· drag pan · **minimap navigator** with event-density histogram + draggable
viewport · per-row radix (hex/bin/dec/signed/oct/ascii) · per-row colors ·
row drag reorder · virtualized rendering for 500+ signals.

**Analysis Hub tab** — report cards computed from your real dump: toggle
coverage, switching-power ranking, X-propagation scan, glitch report, latch
audit — plus the embedded session verdict. **Design Intel tab** — full
Open-KDB browser: driver sites (file:line + RHS text), inferred latches, CDC
paths with synchronizer verdicts, FSM encodings, Tarjan SCC loops.

**Search** — three modes cycled on one box: signal regex (`add:` prefix adds
all matches to the view), boolean temporal (`valid && !ready || stall`,
evaluated at every event edge + 1-ns grid), and `goto:123ns`. Results panel
with [ ] walking.

**File > Open works on any dump** — native `.vwf` binary (VIKASVWF magic +
CRC) and **any standard VCD from any simulator** are both parsed client-side
in JavaScript (varint + 2-bit 4-state packed decode, same packing as the
native format). Export: PNG (view / all rows), SVG, CSV transitions, JSON
steps, WaveDrom timing, and re-loadable view layouts.

**Themes** — VIKAS Dark · Verdi Classic (gold) · Studio Light · Synth Neon,
persisted via localStorage. F1 opens the full keyboard/mouse reference.

**Interactions** (all verified): hover-hold tooltip (name, live value, next
change Δt, unique-value count) · click cursor A · shift-click cursor B ·
right-click context menus on rows (radix, colors, expand, remove) · Ctrl+K
palette · F1 help · +/- zoom · arrows pan · Shift+arrows edge-jump · Home/End ·
PageUp/Down · [ ] results · G/U group · E bus-expand · A analog · T theme ·
` console.

## Supported source types (multi-file, 30 kinds)

`.sv .svh .v .vh .svi .svp .svt .sva .svlib .vams .vg .vi .vo .vlog .ver .vhd .vhdl .vho .uvm .pkg .rtl .netlist .mhdl`
+ **filelists**: `.f .flist .filelist .args .scr .q .cmd .files .list`
(industry format: `+incdir+dir`, `+define+N=V`, nested `-f`, comments).

## The architecture

```text
vikas-eda/
├── crates/
│   ├── vikas-sim/       XEZIM kernel: lexer, parser, elaborator, event VM
│   │                    (IEEE delta + NBA regions, `include, filelists, params,
│   │                    unpacked memory arrays, latch inference analysis,
│   │                    expression port connections, mutation-testing engine)
│   ├── vikas-kdb/       Open-KDB: CSR driver graph, Tarjan SCC loop detection,
│   │                    latch L1 static extractor, CDC, FSM, counterfactual, SVA
│   ├── vikas-physics/   E=½CV² energy, switching-activity power, sub-threshold
│   │                    0.12V + cryo 77K engines, glitch hazards, P-Bit/Glauber,
│   │                    HDC fingerprints, E-Graph repair + C++20 RK4 twin
│   ├── vikas-wave/      4-state values, VCD + 6 native formats, parallel diff,
│   │                    bit-parallel search, FFT ENOB, bus decode, RV32I disasm
│   ├── vikas-linux/     /dev/shm arena, fork() CoW snapshots, inotify watcher,
│   │                    cache-line-aligned SPSC ring buffer, CPU topology
│   ├── vikas-wcp/       VIKAS STUDIO generator (menus, 40-icon toolbar, docks,
│   │                    minimap, 4 themes, palette, hub, client-side VCD/VWF
│   │                    loaders) + WCP \0-framed socket server, .sucl generator,
│   │                    WaveDrom/SVG export, MCP JSON-RPC 2.0 server
│   └── vikas-cli/       the `vikas` executable: 58 subcommands + TUI + editor
└── examples/            6 single-file demos + multifile_proj + nova_soc/
                         (26-file 5-level SoC: CPU+bus+7 periphs, UVM-style TB,
                         nested filelists, .v/.vh/.svh/.netlist/.vho/.uvm mix)
```

## 18 breakthroughs (the duopoly cannot do these during RTL simulation)

1. Counterfactual "why is this value wrong" solver (`vikas why`)
2. Pre-synthesis live silicon physics (energy/FO4 tracks in the dump)
3. Sub-threshold 0.12V & cryo-CMOS corner engine (`--vdd 0.12 --temp-k 77`)
4. 64-bit causality taint-dye propagation (`vikas taint`)
5. P-Bit / stochastic / entropy engine + HDC fingerprints
6. Closed-loop C++20 RK4 mixed-signal twin (SAR ADC, monotonicity verified)
7. E-Graph equality-saturation micro-architecture auto-repair
8. fork() CoW silicon time-forking (`vikas fork`)
9. Sub-cycle glitch hazard detection (VIKAS_CAUSALITY tracks)
10. /dev/shm zero-disk-wear arena + lock-free SPSC ring
11. Bit-parallel temporal query engine
12. Surfer WCP `\0`-framed protocol + native MCP server for AI agents
13. **3-layer inferred latch detection** (static L1 + live hold-state telemetry
    `VIKAS_CAUSALITY_latch_hold_*` + copy-paste SV patches, exit 2 in CI)
14. **RTL MUTATION TESTING** (`vikas mutant`) — the hardware analog of
    mutmut/cosmic-ray: op-swaps (+↔−, ==↔!=, <<↔>>, &&↔||), const-zeroing
    and off-by-one are injected into every DUT module (the testbench itself
    is never mutated), each mutant is re-simulated against golden, and the
    score tells you if your testbench is any good. Surviving mutants come
    back as a precise, actionable testbench-gap report. No commercial
    simulator offers this CLI-native. `--json` for CI quality gates,
    exit 3 under `--min-score`.
15. **Waveform-driven switching power** (`vikas power`) — P = Σ ½CV²·f_tr
    where f_tr comes from REAL toggle counts in the simulation dump, not
    assumed activity factors. Per-signal hotspot ranking, clock-tree share
    (with clock-gating verdict), window energy and fJ/cycle budget.
16. **One-command tapeout-risk signoff** (`vikas signoff`) — latch audit,
    combinational loops, CDC synchronizers, toggle coverage, liveness
    freeze detection and TB verdict (optionally + mutation score) composed
    into one weighted score out of 100 with exit codes for CI:
    ≥85 SIGNOFF READY / ≥60 AT RISK / else BLOCKED.
17. **Topo-scheduled event kernel v1.1** — continuous assigns are ordered
    once by data dependency (Kahn, stable) so acyclic combinational chains
    settle in ONE delta iteration instead of ripple-sweeping up to 64;
    the delta loop borrows its assign table with `mem::take` (zero clones),
    port-sync no longer clones the full binding vector, per-timestep
    fired-flags replace String-hashset keys, and latch lookup short-circuits
    on latch-clean designs. Measured on this repo's own stress suite:
    deep-comb events/s 407k · NBA pipelines 489k · NOVA-SoC wall
    4.76s -> 1.94s (-59%) — all goldens byte-identical (9/9 NOVA-PASS).
18. **VIKAS STUDIO** — the EDA industry's first single-file, fully-offline
    professional workbench: no server, no CDN, no install — open the HTML in
    any browser and File > Open parses ANY simulator's VCD locally. Analysis
    Hub + Design Intel give instant signoff-style insight on any dump, and
    every action is a real wired command (37 in the palette), not a mockup.

## 6 native innovative waveform formats

| ext | name | superpower |
|-----|------|-------------|
| `.vwf` | VIKAS Waveform File | binary indexed table + CRC32 integrity |
| `.vwt` | VIKAS Temporal | time-sectioned regions for partial reads |
| `.vwe` | VIKAS Encoded | delta-time + value-RLE compression |
| `.vwp` | VIKAS Physics | analog f64 lanes for physics telemetry |
| `.vwi` | VIKAS Indexed | per-signal seek index, O(1) random access |
| `.vwm` | VIKAS Memorymap | 32-byte aligned records for zero-copy mmap |

All round-trip lossless (verified by unit tests) + full VCD interop (`vikas convert`).

## Verification status (v1.1)

- `cargo test --release`: **33/33 unit tests** (kernel semantics, latch L1, mutation engine,
  switching-power formula, e-graph, SAR twin, formats)
- `test_matrix.sh`: **80/80 command checks** individually executed and captured to `qa_results/`
  (now includes the bench suite + 4 STUDIO golden checks: app completeness,
  palette wiring, KDB embedding, client-side file loaders)
- `multi_terminal_verify.sh`: **5 terminals in PARALLEL, 20/20 checks green**
  under full CPU contention — T1 NOVA 9/9 + mutation kills · T2 bench at
  2 scales (29k events/s aggregate under 5-way load, 66k unloaded) · T3
  signoff 90/100 READY + latch-trap exit 2 + CDC + FSM · T4 455-signal dump
  VCD roundtrip + 283 KB studio · T5 all 5 native formats + temporal search
- **NOVA-SoC deep multi-file project** (26 files, 5 hierarchy levels, 9 source
  extensions, nested filelists): runs on the master `.f` AND the alternate
  `.files` manifest with identical **NOVA-PASS 9/9** golden checks — CPU ISA
  loop/branch/spin-wait, UART byte recovered through the serial line, SPI
  loopback, timer IRQ, watchdog no-bark, 2-FF synchronizer, PWM activity,
  tie-cells via bus status, dual-master debug reads
- The NOVA validation campaign surfaced and fixed **9 kernel bugs** (see git
  history): late-edge pass for parent-scope `@(dut.out)` events, for-loop park
  continuation, concat item order, bound-slice ranges, port width-fit
  direction, UTF-8-safe lexer + VHDL `--` comments, expression port
  connections, 32-bit `integer`/`int`, runaway for-loop guard
- **5 terminals in PARALLEL: 41/41 checks OK** (fresh-install, NOVA deep,
  physics/cosim, 6 wave formats + GUI 455-signal + TUI PTY 5/5, env -i hardiness)
- **Clone-readiness: 18/18** — 3 fresh directories as 3 users (build-from-zero,
  prebuilt, foreign-cwd minimal-env) all reproduce identical goldens
- GUI verified in real Chromium at 455-signal scale (5.1M painted pixels,
  search narrows, hover tooltip)
- TUI verified in a real PTY on the 455-signal NOVA dump (5/5)

VIKAS v1 — open source, for the next generation of silicon engineers.
