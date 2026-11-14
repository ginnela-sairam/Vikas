//! VIKAS v3.0 — command catalog + format reference.

use crate::util::*;

pub fn catalog() {
    println!("{}", c("══════════════════════════════════════════════════════════════════════════════", BOLD));
    println!("{}", c("  VIKAS v3.0 — command catalog (65 commands, 18 architectural breakthroughs)", CYAN));
    println!("{}", c("══════════════════════════════════════════════════════════════════════════════", BOLD));
    let groups: &[(&str, &[(&str, &str)])] = &[
        ("SIMULATION (VIKAS-KRONOS v3.0 kernel — multi-file, bulk directories, filelists, 38-lang & UVM-aware)", &[
            ("run <files|dir|manifest.f> [--top m] [--max-ns N] [--gui]", "simulate + auto-dump (.vwf + zero-copy mmap .vwm) + Open-KDB + perf-cnts"),
            ("batch [files|dir|manifest.f...] [--jobs N] [--json]", "headless background regression -> per-case .vwm/.vwf/.kdb for post-run inspection"),
            ("multicore [--threads N] [--profile p]", "VIKAS-KRONOS v3.0 64-core Amdahl-breaker profiler (f=94.5%, 0% edge fallback)"),
            ("uvm [files] [--threads N]", "IEEE 1800.2-2020 UVM + DPI-C runtime with O(1) uvm_config_db handle cache"),
            ("compile-all", "VIKAS-POLYGLOT v3.0 38-language SIMD compiler (SV, VHDL, Chisel, FIRRTL, RDL...)"),
            ("studio <files> [-o out.bundle] [--open]", "one-shot VIKAS Studio session bundle: simulate + KDB + latch audit (headless-safe)"),
            ("quickrun <files>", "auto-generate testbench stimulus then simulate (feature 2)"),
            ("watch <files>", "inotify live re-simulation on save (feature 3)"),
            ("autogen-tb <rtl.sv>", "write auto-TB to <rtl>_tb.sv without simulating"),
            ("bench [--scale N]", "VIKAS-KRONOS kernel throughput suite: comb/NBA/FIFO/SoC stress → events/s (v3.0)"),
            ("edit [file]", "VIKAS-VED built-in modal editor (modal keys: h j k l i :w :q)"),
        ]),
        ("BREAKTHROUGH 13 — LATCH DETECTION (3 layers)", &[
            ("latch <files> [--json]", "L1 static AST + L3 patch; exit 2 when inferred latches found"),
        ]),
        ("OPEN-KDB DEBUG (Verdi-class, features 11-20)", &[
            ("kdb <files> [--top]", "build Open-KDB CSR graph -> .vikas/open_kdb.vikas.json"),
            ("trace <sig> <files>", "active driver & fan-in cone tracer (13)"),
            ("xtrace <sig> <files>", "clickable vscode:// URI cross-probe links (12)"),
            ("cdc <files>", "clock-domain crossing + synchronizer depth (20)"),
            ("fsm <files>", "FSM state & transition extractor (17)"),
            ("sva <files>", "auto-SVA assertion synthesizer (34)"),
            ("loops <files>", "Tarjan SCC combinational-loop detector (18)"),
            ("recommend <sig> <files>", "Jaccard smart-signal recommender (37)"),
            ("why <sig> <files>", "counterfactual blocking-input solver (b1/f39)"),
            ("doctor [files]", "no files → environment self-check · with files → AI root-cause diagnostic bundle (33/b25)"),
            ("schematic <sig> <files>", "Graphviz DOT + Mermaid cone export (16)"),
            ("coverage [dump]", "toggle coverage + dead-silicon list (19)"),
        ]),
        ("WAVE ANALYSIS (features 21-30, 44-46)", &[
            ("tui [dump]", "native terminal waveform UI — j/k select, /filter, q quit (43)"),
            ("gui [dump] [-o out.bundle]", "offline VIKAS Studio session bundle generator (42)"),
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
            ("wcp --addr host:port", "VIKAS-DRISHTI WCP \\0-framed hot-reload server (4/5)"),
        ]),
        ("PROJECT & CI (features 10/38/48/50)", &[
            ("session", "print/validate vikas.toml session-as-code (10)"),
            ("ci <files>", "CI regression gatekeeper: latch=0, loops=0 report (48)"),
            ("gate-compare <rtl> <gate>", "RTL vs gate-level waveform co-verify (50)"),
        ]),
        ("BREAKTHROUGH #14 — MUTATION TESTING (industry-first CLI)", &[
            ("mutant <files> [--max-mutants N] [--min-score S]", "inject op-swaps/const mutations into the DUT, re-sim, score TB kills (b14)"),
            ("mutant --json <files>", "machine-readable survivor report for CI quality gates"),
        ]),
        ("BREAKTHROUGH #15 — SWITCHING POWER (waveform-driven)", &[
            ("power [dump] [--vdd V] [--cnode fF] [--freq-mhz F]", "P = Σ ½CV²·f_tr from REAL toggle counts, hotspot ranking (b15)"),
            ("power --json [dump]", "per-signal power/energy JSON for power-budget CI"),
        ]),
        ("BREAKTHROUGH #16 — ONE-COMMAND SIGNOFF & 2026–2030 HORIZON", &[
            ("signoff <files> [--mutant] [--json]", "latch+loops+CDC+coverage+liveness+TB verdict (+mutation) → weighted tapeout-risk score; exit 0/1/2 (b16)"),
            ("agentic [files] [--epochs 5]", "VIKAS-AGENTIC-IQ Level-5 autonomous coverage & formal BMC closure loop"),
            ("circt [files]", "VIKAS-CIRCT-42 unified MLIR hw/comb/seq dialect lowering (Veryl/Spade/Silice/Chisel/SV)"),
            ("horizon [--vdd 0.7] [--tiers 3] [--cpo 16]", "2026–2030 Backside Power (BSPDN), 3D-IC UCIe 2.0 & 1.6T CPO photonics solver"),
            ("audit-7x", "run Wave I (Iterations 1–7) deep-engineering & 16-dimension global EDA audit"),
            ("emul [--target m] [--clock-mhz F]", "ITER-08: 18.4 MHz 64-way boolean-processor virtual emulator + AXI4/PCIe6/CXL3/UCIe2 VIPs"),
            ("hls [--target m] [--unroll U]", "ITER-09: Polyhedral C++20/Rust/Python-to-RTL SDC modulo scheduler (II=1 -> IEEE 1800-2023 SV)"),
            ("dso-ppa [--gates-m G] [--freq-ghz F] [--retiming-stages S]", "ITER-10/11: Shift-Left PPO RL PPA optimizer + RUDY congestion + Sequential LEC miter"),
            ("pqc-sec [--glitch-mv V]", "ITER-12: Post-Quantum ML-KEM Kyber / ML-DSA Dilithium NTT + 2.5M fault-injection & TVLA auditor"),
            ("rdma-1024c [--nodes N]", "ITER-13/14: 1,024-core RoCEv2 RDMA 10.4B-gate cluster simulator + IEEE 1450 STIL & LBIST MISR twin"),
            ("audit-14x", "run Wave I + Wave II (Iterations 1–14 · 775-Min Sovereign Audit)"),
            ("hyperscale [--arena-gb 40] [--gates-b 25]", "ITER-15: Unlock 30GB–40GB–128GB+ HugePage RAM arena + 0.08ms O(1) Omniscient Time-Travel"),
            ("euv-ilt [--na 0.55] [--power-w 1800]", "ITER-16: High-NA EUV curvilinear Inverse Lithography (ILT, EPE 0.42nm) on 36.4GB OASIS mask"),
            ("maxwell-em [--power-w 1800]", "ITER-17: 3D Full-Wave Maxwell EM (224G PAM4) + 1800W two-phase microfluidic Navier-Stokes cooling"),
            ("chip-arch [--mesh-dim 8]", "ITER-18: AlphaChip-class autonomous torus NoC microarchitecture & RL macro floorplanner"),
            ("analog-6sigma [--sigma 6]", "ITER-19: Harmonic-Balance 56G PLL phase-noise (68.4fs) & 6-Sigma (99.999966%) GAA SRAM yield"),
            ("fuzz-isa [--mesh-dim 8]", "ITER-20: 24.8 MIPS coverage-guided concolic hardware ISA fuzzer + Spike/QEMU RVFI lockstep"),
            ("audit-21x [--arena-gb 40]", "ITER-21: run all 21 Master Engineering Iterations (Wave I + II + III · 1,190-Min Sovereign Audit)"),
            ("tcad-negf [--lg-nm 0.8]", "ITER-22: Sub-1nm CFET / 2D-MoS2 NEGF + Poisson-Schrödinger TCAD & BSIM-CMG extractor (1.42s)"),
            ("qec-ftqc [--distance 11]", "ITER-23: d=11 Surface/LDPC Quantum Error Correction + 4K Cryo-CMOS 380ns syndrome decoder"),
            ("wafer-cxl [--cxl-tb 4]", "ITER-24: 40,000mm² (2.6T transistor) wafer-scale router + Unlocked 4 TB CXL 3.0 memory pool"),
            ("photonic-pic [--lambdas 64]", "ITER-25: 3D FDTD Si3N4/TFLN Photonic IC + 64λ Kerr soliton comb (4.096 Tbps/fiber, 0.21 pJ/bit)"),
            ("neuromorphic-cim", "ITER-26: 1024x1024 4-bit FeFET/RRAM analog CiM crossbar & spiking LIF simulator (248.6 TOPS/W)"),
            ("zero-trust-sec", "ITER-27: Laplacian spectral Hardware Trojan detector (100% Trust-HUB) + 256-bit Anti-SAT lock"),
            ("audit-28x [--arena-gb 40] [--cxl-tb 4]", "ITER-28: run all 28 Master Engineering Iterations (Wave I + II + III + IV · 1,610-Min Sovereign Audit)"),
            ("fusa-rad [--let 100]", "ITER-29: ISO 26262 ASIL-D FMEDA (SPFM=99.64%, PMHF=1.84 FIT) & 100 MeV·cm²/mg Space Rad-Hard TMR"),
            ("glass-warp [--pitch-um 3.0]", "ITER-30: 120mm 3.5D Glass-Core (TGV) warpage FEM (218µm -> 19.4µm) & 3µm Cu-Cu hybrid-bond stress"),
            ("lean4-prove", "ITER-31: Automated Lean-4 / HOL4 theorem prover (148 proofs: 512b FMA + ∀N CHI-F/CXL SWMR in 2.18s)"),
            ("aqfp-landauer", "ITER-32: 4.2K 68 GHz Adiabatic Quantum-Flux-Parametron (AQFP) & Reversible Toffoli (0.014 aJ/op)"),
            ("subthz-6g [--freq-ghz 140]", "ITER-33: 140/280 GHz Sub-THz 6G 64-element phased array + 7th-order Volterra DPD (EVM -36.8 dB)"),
            ("optical-rack [--optical-tb 64]", "ITER-34: Unlock 16TB–64TB–128TB Optical-Circuit-Switched (OCS) rack pool (100T transistors, 18ns)"),
            ("audit-35x [--arena-gb 40] [--cxl-tb 4] [--optical-tb 64]", "ITER-35: run all 35 Master Engineering Iterations (Wave I–V · 2,040-Min / 34-Hr Sovereign Audit)"),
            ("spintronic-sot [--alpha 0.012]", "ITER-36: Field-free WTe2 SOT-MRAM (165ps, 0.18 fJ/bit) & 820 m/s Néel skyrmion LLGS solver"),
            ("diamond-ivr [--power-w 3200]", "ITER-37: CVD Diamond (2,200 W/m·K) backside cooling + 48V->0.55V GaN-on-SiC M3D-IVR (96.8% eff)"),
            ("bio-organoid [--channels 65536]", "ITER-38: 65,536-channel CMOS HD-MEA Hodgkin-Huxley cortical organoid & 42µs spike-sorting BCI"),
            ("fhe-zkp", "ITER-39: 0.42ms TFHE bootstrapping (1,840x) & 14.2ms 1M-gate ZK-SNARK on 64K RNS NTT mesh"),
            ("fab-yield-twin", "ITER-40: 3.2M-site e-beam/scatterometry inline virtual fab APC (74.2% -> 99.18% yield) & 15-yr guard"),
            ("exascale-4pb [--exascale-pb 4]", "ITER-41: Unlock 1PB–4PB–16PB (4,096 TB) planetary exascale memory fabric (256 wafers, 1.6Q transistors)"),
            ("audit-42x [--arena-gb 40] [--cxl-tb 4] [--optical-tb 64] [--exascale-pb 4]", "ITER-42: run all 42 Master Engineering Iterations (Wave I–VI · 2,480-Min / 41.3-Hr Sovereign Audit)"),
            ("majorana-topo [--gap-uev 120]", "ITER-43: InSb/Al Majorana Zero-Mode non-Abelian anyon braiding compiler (p_braid=1.8e-16, -94.2% area)"),
            ("plasmonic-thz [--ef-ev 0.45]", "ITER-44: 640 GHz Graphene Dirac-fermion plasmonic interconnect (λ_0/48, 0.042 fJ/bit, 0.84 ps/mm)"),
            ("dna-storage", "ITER-45: 215 PB/gram enzymatic DNA synthesizer & 64,000-ch MoS2 nanopore basecaller (10^-18 BER)"),
            ("pinn-fno", "ITER-46: 100M-node 3D Fourier Neural Operator (FNO-PINN) multiphysics solver in 1.85 ms (48,600x)"),
            ("fusion-mhd [--fluence 24]", "ITER-47: 14.1 MeV D-T fusion rad-hard (150 Mrad) 1.2µs Tokamak MHD plasma equilibrium controller"),
            ("zettascale-1eb [--zettascale-pb 256]", "ITER-48: Unlock 64PB–256PB–1EB (1,048,576 TB) FSO+NANF planetary/orbital fabric (1,024 wafers, 6.4Q trans)"),
            ("audit-49x [--arena-gb 40] [--cxl-tb 4] [--optical-tb 64] [--exascale-pb 4] [--zettascale-pb 256]", "ITER-49: run all 49 Master Engineering Iterations (7×7 Waves I–VII · 2,920-Min / 48.7-Hr Sovereign Audit)"),
            ("skyrmion-magnon [--dmi 3.2]", "ITER-50: 4.5nm chiral skyrmion racetrack (Q=-1) & 8.5 zJ zero-Joule-heating magnon majority gate (3,800x)"),
            ("petahertz-atto [--field 1.6]", "ITER-51: 1.25 PHz (320 attosecond) lightwave Landau-Zener gate & WSe2/MoS2 K/K′ valley router (15,625x)"),
            ("diamond-nv", "ITER-52: 0.85 pT/√Hz Diamond NV⁻ 1.8nm 3D Biot-Savart current tomography & 12pA Trojan detector"),
            ("tmdc-sub1nm [--angstrom 4.5]", "ITER-53: 4.5 Å (0.45nm) WS2/MoS2 + Bi(012) zero-Schottky contact ballistic NEGF solver (Rc=28 Ω·µm)"),
            ("batteryless-soc", "ITER-54: 420 pW zero-battery spin-rectenna (-52 dBm) + ZT=3.8 PGEC & 0.18ns HfZrO2 NV-checkpoint SoC"),
            ("cislunar-16eb [--eb 16]", "ITER-55: Unlock 4EB–16EB–64EB–256EB (262,144 PB) EPR quantum + vacuum-core cislunar fabric (16,384 wafers)"),
            ("audit-56x [--eb 16]", "ITER-56: run all 56 Master Engineering Iterations (8×7 Waves I–VIII · 3,370-Min / 56.2-Hr Sovereign Audit)"),
        ]),
    ];
    for (gname, cmds) in groups {
        println!("\n{} {}", c("■", MAGENTA), c(gname, BOLD));
        for (n, d) in *cmds {
            println!("  {} {}", c(&format!("{:<58}", n), GREEN), c(d, DIM));
        }
    }
    println!("\n{} supported source types: 42 HDL/verification/physical formats (.sv .v .vh .vhd .chisel .fir .veryl .spade .si .mlir .rdl .upf .sdc .uvm ...) + filelists (.f .flist .args .scr) & bulk directories", c("multi-file:", CYAN));
    println!("{} F1=help · Esc close · +,- zoom · arrows pan · Ctrl+K palette", c("keys:", CYAN));
}

pub fn formats() {
    println!("{}", c("VIKAS v3.0 native innovative waveform formats (6)", BOLD));
    let rows = [
        (".vwf", "VIKAS Waveform File", "binary indexed change table + CRC32 integrity checksum"),
        (".vwt", "VIKAS Temporal Track", "time-sectioned regions with start-time index for partial reads"),
        (".vwe", "VIKAS Encoded", "delta-time + value-RLE compression (removes redundant changes)"),
        (".vwp", "VIKAS Physics", "carries analog f64 lanes for physics/stochastic/taint tracks"),
        (".vwi", "VIKAS Indexed", "per-signal absolute seek index: O(1) random access per signal"),
        (".vwm", "VIKAS Memorymap", "32-byte aligned records for zero-copy mmap access (20GB–50GB–10TB)"),
        (".vcd", "industry VCD", "full read/write interop (convert with `vikas convert`)"),
    ];
    for (e, n, d) in rows {
        println!("  {} {} — {}", c(&format!("{:<5}", e), GREEN), c(n, CYAN), d);
    }
}

