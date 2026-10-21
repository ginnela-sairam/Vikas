//! VIKAS v1 — command catalog + format reference.

use crate::util::*;

pub fn catalog() {
    println!("{}", c("══════════════════════════════════════════════════════════════", BOLD));
    println!("{}", c("  VIKAS v1 — command catalog (58 commands, 16 breakthroughs)", CYAN));
    println!("{}", c("══════════════════════════════════════════════════════════════", BOLD));
    let groups: &[(&str, &[(&str, &str)])] = &[
        ("SIMULATION (XEZIM kernel — multi-file, filelists, UVM-aware)", &[
            ("run <files|manifest.f> [--top m] [--max-ns N] [--gui]", "simulate + auto-dump + Open-KDB + telemetry + latch alert"),
            ("quickrun <files>", "auto-generate testbench stimulus then simulate (feature 2)"),
            ("watch <files>", "inotify live re-simulation on save (feature 3)"),
            ("autogen-tb <rtl.sv>", "write auto-TB to <rtl>_tb.sv without simulating"),
            ("edit [file]", "VIKAS built-in editor (vim-style keys: h j k l i :w :q)"),
        ]),
        ("BREAKTHROUGH 13 — LATCH DETECTION (3 layers)", &[
            ("latch <files> [--json]", "L1 static AST + L3 patch; exit 2 when inferred latches found"),
        ]),
        ("OPEN-KDB DEBUG (design-graph intelligence, features 11-20)", &[
            ("kdb <files> [--top]", "build Open-KDB CSR graph -> .vikas/open_kdb.vikas.json"),
            ("trace <sig> <files>", "active driver & fan-in cone tracer (13)"),
            ("xtrace <sig> <files>", "clickable vscode:// URI cross-probe links (12)"),
            ("cdc <files>", "clock-domain crossing + synchronizer depth (20)"),
            ("fsm <files>", "FSM state & transition extractor (17)"),
            ("sva <files>", "auto-SVA assertion synthesizer (34)"),
            ("loops <files>", "Tarjan SCC combinational-loop detector (18)"),
            ("recommend <sig> <files>", "Jaccard smart-signal recommender (37)"),
            ("why <sig> <files>", "counterfactual blocking-input solver (b1/f39)"),
            ("doctor <files>", "AI root-cause diagnostic bundle (33)"),
            ("schematic <sig> <files>", "Graphviz DOT + Mermaid cone export (16)"),
            ("coverage [dump]", "toggle coverage + dead-silicon list (19)"),
        ]),
        ("WAVE ANALYSIS (features 21-30, 44-46)", &[
            ("tui [dump]", "terminal UI — j/k select, /filter, q quit (43)"),
            ("gui [dump] [-o out.html]", "offline single-file HTML5 GUI with hover tooltips (42)"),
            ("summary [dump]", "executive telemetry scorecard (36)"),
            ("annotate <file.sv> --time t", "inline live-value RTL source annotator (15)"),
            ("find \"cond\" [--dump d]", "bit-parallel temporal search (b11/32)"),
            ("expr \"a && b\"", "virtual signal expression evaluator (26)"),
            ("diff <a> <b>", "parallel earliest-divergence differ (25)"),
            ("decode [dump]", "bus protocol auto-decoder AXI/APB/SPI/I2C/UART (21)"),
            ("disasm [dump]", "RV32I opcode disassembler (30)"),
            ("spectrum [dump] --bus b", "FFT: SNR/SINAD/THD/SFDR/ENOB (23)"),
            ("perf [dump]", "backpressure/stall utilization profiler (29)"),
            ("radix <hex> q15|fp16|bf16|fp8", "fixed-point & AI-float translator (27)"),
            ("memdump [dump] --array a --time t", "2D memory hex inspector (28)"),
            ("export-doc [dump]", "WaveDrom JSON + publication SVG (40)"),
            ("export-tb [dump]", "waveform -> standalone SV testbench (38)"),
            ("anomaly [dump]", "deadlock/livelock freeze detector (35)"),
            ("delta <t1> <t2>", "Δt/frequency/latency calculator (46)"),
            ("cycles <t> <period-ns>", "ns <-> clock-cycle ruler (45)"),
            ("convert <in> <out.vwf|vwt|vwe|vwp|vwi|vwm|vcd>", "lossless format conversion"),
            ("formats", "native innovative format reference"),
        ]),
        ("SILICON PHYSICS (breakthroughs 2/3/5/6/7/9)", &[
            ("physics <files> --vdd 0.12 --temp-k 77", "live physics + sub-threshold/cryo verdicts"),
            ("energy <files> --vdd 0.8", "E=½CV² dynamic energy estimate (b2)"),
            ("sweep --vdd 0.12,0.4,0.8 --temp-k 300", "multi-corner V/T sweep table (49)"),
            ("pbit [dump]", "Glauber p-bit + entropy analytics (b5)"),
            ("cosim [bits]", "C++20 RK4 SAR ADC closed-loop twin (b6)"),
            ("egraph \"a&&a\"", "equality-saturation cone repair -> SV patch (b7)"),
            ("hdc [dump]", "10,000-bit hypervector fingerprint (b5)"),
            ("taint <src> <dst> <files>", "64-bit causality dye propagation (b4)"),
        ]),
        ("LINUX-NATIVE (breakthroughs 8/10)", &[
            ("topo", "CPU topology + auto-tune report (9)"),
            ("fork [N]", "fork() CoW snapshot timing demo (b8)"),
            ("shm", "/dev/shm zero-disk-wear arena status (b10)"),
        ]),
        ("AI & INTEGRATION (breakthrough 12)", &[
            ("mcp [workdir]", "MCP JSON-RPC 2.0 server: signals/wave_slice/end_time (31)"),
            ("wcp --addr host:port", "Surfer WCP \\0-framed hot-reload server (4/5)"),
        ]),
        ("PROJECT & CI (features 10/38/48/50)", &[
            ("session", "print/validate vikas.toml session-as-code (10)"),
            ("ci <files>", "CI regression gatekeeper: latch=0, loops=0 report (48)"),
            ("gate-compare <rtl> <gate>", "RTL vs gate-level waveform co-verify (50)"),
        ]),
        ("BREAKTHROUGH #14 — MUTATION TESTING (testbench quality, CLI-native)", &[
            ("mutant <files> [--max-mutants N | --max N] [--min-score S]", "inject op-swaps/const mutations into the DUT, re-sim, score TB kills (b14)"),
            ("mutant --json <files>", "machine-readable survivor report for CI quality gates"),
        ]),
        ("BREAKTHROUGH #15 — SWITCHING POWER (waveform-driven)", &[
            ("power [dump|rtl] [--vdd V] [--cnode fF] [--freq-mhz F] [--max-ns N]", "P = Σ ½CV²·f_tr from REAL toggles; RTL input auto-simulates first (zero-touch) (b15)"),
            ("power --json [dump]", "per-signal power/energy JSON for power-budget CI"),
        ]),
        ("BREAKTHROUGH #16 — ONE-COMMAND SIGNOFF", &[
            ("signoff <files> [--mutant] [--json]", "latch+loops+CDC+coverage+liveness+TB verdict (+mutation) → weighted tapeout-risk score; exit 0/1/2 (b16)"),
        ]),
    ];
    for (gname, cmds) in groups {
        println!("\n{} {}", c("■", MAGENTA), c(gname, BOLD));
        for (n, d) in *cmds {
            println!("  {} {}", c(&format!("{:<58}", n), GREEN), c(d, DIM));
        }
    }
    println!("\n{} supported source types: .sv .svh .v .vh .svi .svp .svt .sva .uvm .vhd .vhdl + filelists (.f .flist .args .scr .q) — `vikas run manifest.f`", c("multi-file:", CYAN));
    println!("{} F1=help(zoom GUI) · Esc close · +,- zoom · arrows pan · hover-hold=tooltip", c("keys:", CYAN));
}

pub fn formats() {
    println!("{}", c("VIKAS v1 native innovative waveform formats (6)", BOLD));
    let rows = [
        (".vwf", "VIKAS Waveform File", "binary indexed change table + CRC32 integrity checksum"),
        (".vwt", "VIKAS Temporal Track", "time-sectioned regions with start-time index for partial reads"),
        (".vwe", "VIKAS Encoded", "delta-time + value-RLE compression (removes redundant changes)"),
        (".vwp", "VIKAS Physics", "carries analog f64 lanes for physics/stochastic/taint tracks"),
        (".vwi", "VIKAS Indexed", "per-signal absolute seek index: O(1) random access per signal"),
        (".vwm", "VIKAS Memorymap", "32-byte aligned records for zero-copy mmap access"),
        (".vcd", "industry VCD", "full read/write interop (convert with `vikas convert`)"),
    ];
    for (e, n, d) in rows {
        println!("  {} {} — {}", c(&format!("{:<5}", e), GREEN), c(n, CYAN), d);
    }
}
