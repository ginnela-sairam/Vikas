//! # vikas-cli — the `vikas` executable: 55 subcommands across simulation,
//! Open-KDB debug, physics, waveform analysis, Linux-native machinery,
//! AI/MCP servers, GUI/TUI, and project tooling. Zero Python.

mod bench;
mod cmds;
mod help;
mod tui;
mod util;

use anyhow::{bail, Result};
use std::path::PathBuf;
use util::*;

#[derive(Debug)]
enum Cmd {
    Run { files: Vec<PathBuf>, top: Option<String>, max_ns: u64, gui: bool },
    Studio { files: Vec<PathBuf>, top: Option<String>, max_ns: u64, out: Option<PathBuf>, open: bool },
    Quickrun { files: Vec<PathBuf> },
    Watch { files: Vec<PathBuf> },
    Latch { files: Vec<PathBuf>, json: bool },
    Help,
    Version,
    HelpFormats,
    Bench { scale: u32, dir: Option<PathBuf> },
    // KDB / debug
    Kdb { files: Vec<PathBuf>, top: Option<String> },
    Trace { signal: String, files: Vec<PathBuf> },
    Xtrace { signal: String, files: Vec<PathBuf> },
    Cdc { files: Vec<PathBuf> },
    Fsm { files: Vec<PathBuf> },
    Sva { files: Vec<PathBuf> },
    Loops { files: Vec<PathBuf> },
    Recommend { signal: String, files: Vec<PathBuf> },
    Why { signal: String, files: Vec<PathBuf> },
    Doctor { files: Vec<PathBuf> },
    Schematic { signal: String, files: Vec<PathBuf> },
    Coverage { dump: PathBuf },
    // wave analysis
    Tui { dump: PathBuf },
    Gui { dump: PathBuf, out: PathBuf },
    Summary { dump: PathBuf },
    Annotate { file: PathBuf, time: u64 },
    Find { cond: String, dump: PathBuf },
    Expr { formula: String, dump: PathBuf },
    Diff { a: PathBuf, b: PathBuf },
    Decode { dump: PathBuf },
    Disasm { dump: PathBuf },
    Spectrum { dump: PathBuf, bus: String },
    Perf { dump: PathBuf },
    Radix { value: u64, kind: String },
    Memdump { dump: PathBuf, array: String, time: u64 },
    ExportDoc { dump: PathBuf, from: u64, to: u64 },
    ExportTb { dump: PathBuf },
    InfoFormats,
    Convert { input: PathBuf, output: PathBuf },
    Anomaly { dump: PathBuf },
    Delta { a: u64, b: u64 },
    Cycles { time: u64, period: u64 },
    // physics
    Physics { files: Vec<PathBuf>, vdd: f64, temp: f64 },
    Energy { files: Vec<PathBuf>, vdds: String },
    Sweep { vdds: String, temp: f64 },
    Pbit { dump: PathBuf },
    Cosim { bits: u32 },
    Egraph { expr: String },
    Hdc { dump: PathBuf },
    Taint { src: String, dst: String, files: Vec<PathBuf> },
    // linux
    Topo,
    Fork { n: u32 },
    Shm,
    // servers
    Mcp { workdir: PathBuf },
    Wcp { addr: String },
    // project
    Session,
    Ci { files: Vec<PathBuf> },
    GateCompare { rtl: PathBuf, gate: PathBuf },
    AutogenTb { file: PathBuf },
    Editor { file: Option<PathBuf> },
    // breakthroughs 14-16 (this build)
    Mutant { files: Vec<PathBuf>, top: Option<String>, max_ns: u64, max_mutants: usize, min_score: f64, json: bool },
    Power { dump: PathBuf, vdd: f64, cnode_ff: f64, freq_mhz: f64, json: bool },
    Signoff { files: Vec<PathBuf>, top: Option<String>, max_ns: u64, mutant: bool, json: bool },
    Multicore { threads: u32, profile: String },
    Uvm { files: Vec<PathBuf>, threads: u32 },
    CompileAll,
    Batch { files: Vec<PathBuf>, jobs: u32, json: bool },
    Agentic { files: Vec<PathBuf>, epochs: u32 },
    Circt { files: Vec<PathBuf> },
    Horizon { vdd: f64, tiers: u32, cpo: u32 },
    Audit7x,
    Emul { target: String, clock_mhz: u32 },
    Hls { target: String, unroll: u32 },
    DsoPpa { gates_m: f64, freq_ghz: f64, retiming_stages: u32 },
    PqcSec { glitch_mv: u32 },
    Rdma1024c { nodes: u32 },
    Audit14x,
    Hyperscale { arena_gb: u32, gates_b: f64 },
    EuvIlt { na: f64, power_w: u32 },
    MaxwellEm { power_w: u32 },
    ChipArch { mesh_dim: u32 },
    Analog6Sigma { sigma: u32 },
    FuzzIsa { mesh_dim: u32 },
    Audit21x { arena_gb: u32 },
    TcadNegf { lg_nm: f64 },
    QecFtqc { distance: u32 },
    WaferCxl { cxl_tb: f64 },
    PhotonicPic { lambdas: u32 },
    NeuromorphicCim,
    ZeroTrustSec,
    Audit28x { arena_gb: u32, cxl_tb: f64 },
    FusaRad { let_mev: u32 },
    GlassWarp { pitch_um: f64 },
    Lean4Prove,
    AqfpLandauer,
    Subthz6g { freq_ghz: u32 },
    OpticalRack { optical_tb: u32 },
    Audit35x { arena_gb: u32, cxl_tb: f64, optical_tb: u32 },
    SpintronicSot { alpha: f64 },
    DiamondIvr { power_w: u32 },
    BioOrganoid { channels: u32 },
    FheZkp,
    FabYieldTwin,
    Exascale4pb { exascale_pb: u32 },
    Audit42x { arena_gb: u32, cxl_tb: f64, optical_tb: u32, exascale_pb: u32 },
    MajoranaTopo { gap_uev: u32 },
    PlasmonicThz { ef_ev: f64 },
    DnaStorage,
    PinnFno,
    FusionMhd { fluence: u32 },
    Zettascale1eb { zettascale_pb: u32 },
    Audit49x { arena_gb: u32, cxl_tb: f64, optical_tb: u32, exascale_pb: u32, zettascale_pb: u32 },
    SkyrmionMagnon { dmi: f64 },
    PetahertzAtto { field: f64 },
    DiamondNv,
    TmdcSub1nm { angstrom: f64 },
    BatterylessSoc,
    Cislunar16eb { cislunar_eb: u32 },
    Audit56x { arena_gb: u32, cxl_tb: f64, optical_tb: u32, exascale_pb: u32, zettascale_pb: u32, cislunar_eb: u32 },
}

fn parse_args(args: &[String]) -> Result<Cmd> {
    // Breakthrough NB-27 / AP-23: If invoked directly with commercial simulator flags
    // (e.g. `vikas -f project.f -top soc_tb +incdir+inc -full64 -kdb`), route to `run`.
    let first = args.first().map(|s| s.as_str()).unwrap_or("help");
    let implicit_run = first.starts_with("-f")
        || first.starts_with("-F")
        || first.starts_with("+incdir+")
        || first.starts_with("+define+")
        || first == "-top"
        || first == "-s"
        || first == "-full64"
        || first.ends_with(".f")
        || first.ends_with(".sv")
        || first.ends_with(".v");

    let (cmd, rest): (String, Vec<String>) = if implicit_run {
        ("run".to_string(), args.to_vec())
    } else {
        let mut it = args.iter();
        let c = it.next().unwrap_or(&"help".into()).clone();
        (c, it.cloned().collect())
    };

    let files = |r: &[String]| -> Vec<PathBuf> {
        const VALUED: &[&str] = &[
            "--top", "-top", "-s", "--max-ns", "-o", "--time", "--from", "--to", "--array",
            "--dump", "--addr", "--vdd", "--temp-k", "--bus",
            "--max-mutants", "--min-score", "--cnode", "--freq-mhz",
            "--threads", "--jobs", "--profile", "--epochs", "--tiers", "--cpo",
            "--target", "--clock-mhz", "--unroll", "--gates-m", "--freq-ghz", "--retiming-stages",
            "--glitch-mv", "--nodes", "--arena-gb", "--gates-b", "--na", "--power-w", "--mesh-dim", "--sigma",
            "--lg-nm", "--distance", "--cxl-tb", "--lambdas",
            "--let", "--pitch-um", "--optical-tb",
            "--alpha", "--channels", "--exascale-pb",
            "--gap-uev", "--ef-ev", "--fluence", "--zettascale-pb",
            "--dmi", "--field", "--angstrom", "--eb", "--cislunar-eb",
            "-y", "-l", "-do",
        ];
        let mut out = Vec::new();
        let mut skip_next = false;
        let mut take_file_next = false;
        for s in r {
            if skip_next {
                skip_next = false;
                continue;
            }
            if take_file_next {
                take_file_next = false;
                out.push(PathBuf::from(s));
                continue;
            }
            if s == "-f" || s == "-F" || s == "-v" {
                take_file_next = true;
                continue;
            }
            if VALUED.contains(&s.as_str()) {
                skip_next = true;
                continue;
            }
            if !s.starts_with('-') && !s.starts_with('+') {
                out.push(PathBuf::from(s));
            }
        }
        out
    };
    let flagv = |r: &[String], name: &str| -> Option<String> {
        r.iter().position(|s| s == name).and_then(|i| r.get(i + 1)).cloned()
    };
    let top_flag = |r: &[String]| -> Option<String> {
        flagv(r, "--top").or_else(|| flagv(r, "-top")).or_else(|| flagv(r, "-s"))
    };
    let numv = |r: &[String], name: &str, d: u64| -> u64 {
        flagv(r, name).and_then(|v| v.parse().ok()).unwrap_or(d)
    };
    match cmd.as_str() {
        "run" | "vcs" | "xrun" => Ok(Cmd::Run {
            files: files(&rest),
            top: top_flag(&rest),
            max_ns: numv(&rest, "--max-ns", 100_000),
            gui: rest.contains(&"--gui".into()),
        }),
        "studio" => Ok(Cmd::Studio {
            files: files(&rest),
            top: top_flag(&rest),
            max_ns: numv(&rest, "--max-ns", 100_000),
            out: flagv(&rest, "-o").map(PathBuf::from),
            open: rest.contains(&"--open".into()),
        }),
        "quickrun" => Ok(Cmd::Quickrun { files: files(&rest) }),
        "watch" => Ok(Cmd::Watch { files: files(&rest) }),
        "latch" => Ok(Cmd::Latch { files: files(&rest), json: rest.contains(&"--json".into()) }),
        "help" | "--help" | "-h" => Ok(Cmd::Help),
        "version" | "--version" | "-V" => Ok(Cmd::Version),
        "help-formats" => Ok(Cmd::HelpFormats),
        "kdb" => Ok(Cmd::Kdb { files: files(&rest), top: flagv(&rest, "--top") }),
        "trace" => Ok(Cmd::Trace { signal: rest.first().cloned().unwrap_or_default(), files: files(&rest[1..]) }),
        "xtrace" => Ok(Cmd::Xtrace { signal: rest.first().cloned().unwrap_or_default(), files: files(&rest[1..]) }),
        "cdc" => Ok(Cmd::Cdc { files: files(&rest) }),
        "fsm" => Ok(Cmd::Fsm { files: files(&rest) }),
        "sva" | "gen-sva" => Ok(Cmd::Sva { files: files(&rest) }),
        "loops" => Ok(Cmd::Loops { files: files(&rest) }),
        "recommend" => Ok(Cmd::Recommend { signal: rest.first().cloned().unwrap_or_default(), files: files(&rest[1..]) }),
        "why" => Ok(Cmd::Why { signal: rest.first().cloned().unwrap_or_default(), files: files(&rest[1..]) }),
        "doctor" => Ok(Cmd::Doctor { files: files(&rest) }),
        "schematic" => Ok(Cmd::Schematic { signal: rest.first().cloned().unwrap_or_default(), files: files(&rest[1..]) }),
        "coverage" => Ok(Cmd::Coverage { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "tui" => Ok(Cmd::Tui { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "gui" | "html-report" => Ok(Cmd::Gui {
            dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")),
            out: flagv(&rest, "-o").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("vikas_gui.bundle")),
        }),
        "summary" => Ok(Cmd::Summary { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "annotate" => Ok(Cmd::Annotate {
            file: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("top.sv")),
            time: numv(&rest, "--time", 0),
        }),
        "find" => Ok(Cmd::Find { cond: rest.first().cloned().unwrap_or_default(), dump: flagv(&rest, "--dump").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "expr" => Ok(Cmd::Expr { formula: rest.first().cloned().unwrap_or_default(), dump: flagv(&rest, "--dump").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "diff" => {
            let f = files(&rest);
            if f.len() < 2 {
                bail!("usage: vikas diff <gold.vwf> <current.vwf>");
            }
            Ok(Cmd::Diff { a: f[0].clone(), b: f[1].clone() })
        }
        "decode" => Ok(Cmd::Decode { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "disasm" => Ok(Cmd::Disasm { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "spectrum" => Ok(Cmd::Spectrum {
            dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")),
            bus: flagv(&rest, "--bus").unwrap_or_else(|| "adc_out".into()),
        }),
        "perf" => Ok(Cmd::Perf { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "radix" => Ok(Cmd::Radix { value: rest.first().and_then(|v| v.parse().ok()).unwrap_or(0), kind: rest.get(1).cloned().unwrap_or_else(|| "q15".into()) }),
        "memdump" => Ok(Cmd::Memdump {
            dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")),
            array: flagv(&rest, "--array").unwrap_or_else(|| "mem".into()),
            time: numv(&rest, "--time", 0),
        }),
        "export-doc" => Ok(Cmd::ExportDoc { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")), from: numv(&rest, "--from", 0), to: numv(&rest, "--to", 1000) }),
        "export-tb" => Ok(Cmd::ExportTb { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "formats" => Ok(Cmd::InfoFormats),
        "convert" => {
            let f = files(&rest);
            if f.len() < 2 {
                bail!("usage: vikas convert <in.vcd> <out.vwf|vwt|vwe|vwp|vwi|vwm>");
            }
            Ok(Cmd::Convert { input: f[0].clone(), output: f[1].clone() })
        }
        "anomaly" | "scan" => Ok(Cmd::Anomaly { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "delta" => Ok(Cmd::Delta { a: rest.first().and_then(|v| v.parse().ok()).unwrap_or(0), b: rest.get(1).and_then(|v| v.parse().ok()).unwrap_or(0) }),
        "cycles" => Ok(Cmd::Cycles { time: rest.first().and_then(|v| v.parse().ok()).unwrap_or(0), period: rest.get(1).and_then(|v| v.parse().ok()).unwrap_or(10) }),
        "physics" => Ok(Cmd::Physics { files: files(&rest), vdd: flagv(&rest, "--vdd").and_then(|v| v.parse().ok()).unwrap_or(0.8), temp: flagv(&rest, "--temp-k").and_then(|v| v.parse().ok()).unwrap_or(300.0) }),
        "energy" => Ok(Cmd::Energy { files: files(&rest), vdds: flagv(&rest, "--vdd").unwrap_or_else(|| "0.8".into()) }),
        "sweep" => Ok(Cmd::Sweep { vdds: flagv(&rest, "--vdd").unwrap_or_else(|| "0.12,0.4,0.8".into()), temp: flagv(&rest, "--temp-k").and_then(|v| v.parse().ok()).unwrap_or(300.0) }),
        "pbit" => Ok(Cmd::Pbit { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "cosim" => Ok(Cmd::Cosim { bits: rest.first().and_then(|v| v.parse().ok()).unwrap_or(12) }),
        "egraph" => Ok(Cmd::Egraph { expr: rest.first().cloned().unwrap_or_else(|| "a&&a".into()) }),
        "hdc" => Ok(Cmd::Hdc { dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")) }),
        "taint" => Ok(Cmd::Taint { src: rest.first().cloned().unwrap_or_default(), dst: rest.get(1).cloned().unwrap_or_default(), files: files(&rest[2..]) }),
        "topo" => Ok(Cmd::Topo),
        "fork" => Ok(Cmd::Fork { n: rest.first().and_then(|v| v.parse().ok()).unwrap_or(1) }),
        "shm" => Ok(Cmd::Shm),
        "mcp" => Ok(Cmd::Mcp { workdir: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }),
        "wcp" => Ok(Cmd::Wcp { addr: flagv(&rest, "--addr").unwrap_or_else(|| "127.0.0.1:45984".into()) }),
        "session" => Ok(Cmd::Session),
        "ci" => Ok(Cmd::Ci { files: files(&rest) }),
        "gate-compare" => {
            let f = files(&rest);
            if f.len() < 2 {
                bail!("usage: vikas gate-compare <rtl.vwf> <gate.vwf>");
            }
            Ok(Cmd::GateCompare { rtl: f[0].clone(), gate: f[1].clone() })
        }
        "autogen-tb" => Ok(Cmd::AutogenTb { file: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("top.sv")) }),
        "edit" | "ved" => Ok(Cmd::Editor { file: rest.first().map(PathBuf::from) }),
        "mutant" | "mutation" => Ok(Cmd::Mutant {
            files: files(&rest),
            top: flagv(&rest, "--top"),
            max_ns: numv(&rest, "--max-ns", 20_000),
            max_mutants: numv(&rest, "--max-mutants", 200) as usize,
            min_score: flagv(&rest, "--min-score").and_then(|v| v.parse().ok()).unwrap_or(70.0),
            json: rest.contains(&"--json".into()),
        }),
        "power" => Ok(Cmd::Power {
            dump: rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")),
            vdd: flagv(&rest, "--vdd").and_then(|v| v.parse().ok()).unwrap_or(0.8),
            cnode_ff: flagv(&rest, "--cnode").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            freq_mhz: flagv(&rest, "--freq-mhz").and_then(|v| v.parse().ok()).unwrap_or(100.0),
            json: rest.contains(&"--json".into()),
        }),
        "bench" => Ok(Cmd::Bench {
            scale: numv(&rest, "--scale", 3) as u32,
            dir: flagv(&rest, "--out").map(PathBuf::from),
        }),
        "signoff" => Ok(Cmd::Signoff {
            files: files(&rest),
            top: flagv(&rest, "--top"),
            max_ns: numv(&rest, "--max-ns", 50_000),
            mutant: rest.contains(&"--mutant".into()),
            json: rest.contains(&"--json".into()),
        }),
        "multicore" | "kronos-mc" => Ok(Cmd::Multicore {
            threads: numv(&rest, "--threads", 16) as u32,
            profile: flagv(&rest, "--profile").unwrap_or_else(|| "dual_dut_1.45m".into()),
        }),
        "uvm" => Ok(Cmd::Uvm {
            files: files(&rest),
            threads: numv(&rest, "--threads", 16) as u32,
        }),
        "compile-all" | "polyglot" => Ok(Cmd::CompileAll),
        "batch" | "bg" | "jobs" | "regress" => Ok(Cmd::Batch {
            files: files(&rest),
            jobs: numv(&rest, "--jobs", 16) as u32,
            json: rest.contains(&"--json".into()),
        }),
        "agentic" | "agentic-iq" => Ok(Cmd::Agentic {
            files: files(&rest),
            epochs: numv(&rest, "--epochs", 5) as u32,
        }),
        "circt" | "mlir" => Ok(Cmd::Circt {
            files: files(&rest),
        }),
        "horizon" | "bspdn" => Ok(Cmd::Horizon {
            vdd: flagv(&rest, "--vdd").and_then(|v| v.parse().ok()).unwrap_or(0.7),
            tiers: numv(&rest, "--tiers", 3) as u32,
            cpo: numv(&rest, "--cpo", 16) as u32,
        }),
        "audit-7x" | "iterations" => Ok(Cmd::Audit7x),
        "emul" | "veloce-emul" => Ok(Cmd::Emul {
            target: flagv(&rest, "--target").unwrap_or_else(|| "nova_tensor_attention_hls_core".into()),
            clock_mhz: numv(&rest, "--clock-mhz", 1200) as u32,
        }),
        "hls" | "catapult-hls" => Ok(Cmd::Hls {
            target: flagv(&rest, "--target").unwrap_or_else(|| "nova_tensor_attention_hls_core".into()),
            unroll: numv(&rest, "--unroll", 8) as u32,
        }),
        "dso-ppa" | "conformal-lec" => Ok(Cmd::DsoPpa {
            gates_m: flagv(&rest, "--gates-m").and_then(|v| v.parse().ok()).unwrap_or(12.5),
            freq_ghz: flagv(&rest, "--freq-ghz").and_then(|v| v.parse().ok()).unwrap_or(2.4),
            retiming_stages: numv(&rest, "--retiming-stages", 4) as u32,
        }),
        "pqc-sec" | "pqc" => Ok(Cmd::PqcSec {
            glitch_mv: numv(&rest, "--glitch-mv", 220) as u32,
        }),
        "rdma-1024c" | "rdma" => Ok(Cmd::Rdma1024c {
            nodes: numv(&rest, "--nodes", 16) as u32,
        }),
        "audit-14x" | "wave2" => Ok(Cmd::Audit14x),
        "hyperscale" | "unlock-40gb" => Ok(Cmd::Hyperscale {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
            gates_b: flagv(&rest, "--gates-b").and_then(|v| v.parse().ok()).unwrap_or(25.0),
        }),
        "euv-ilt" | "culitho" => Ok(Cmd::EuvIlt {
            na: flagv(&rest, "--na").and_then(|v| v.parse().ok()).unwrap_or(0.55),
            power_w: numv(&rest, "--power-w", 1800) as u32,
        }),
        "maxwell-em" | "microfluidic" => Ok(Cmd::MaxwellEm {
            power_w: numv(&rest, "--power-w", 1800) as u32,
        }),
        "chip-arch" | "alphachip" => Ok(Cmd::ChipArch {
            mesh_dim: numv(&rest, "--mesh-dim", 8) as u32,
        }),
        "analog-6sigma" | "harmonic" => Ok(Cmd::Analog6Sigma {
            sigma: numv(&rest, "--sigma", 6) as u32,
        }),
        "fuzz-isa" | "concolic" => Ok(Cmd::FuzzIsa {
            mesh_dim: numv(&rest, "--mesh-dim", 8) as u32,
        }),
        "audit-21x" | "wave3" => Ok(Cmd::Audit21x {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
        }),
        "tcad-negf" | "negf" | "cfet-tcad" => Ok(Cmd::TcadNegf {
            lg_nm: flagv(&rest, "--lg-nm").and_then(|v| v.parse().ok()).unwrap_or(0.8),
        }),
        "qec-ftqc" | "quantum-qec" => Ok(Cmd::QecFtqc {
            distance: numv(&rest, "--distance", 11) as u32,
        }),
        "wafer-cxl" | "cxl-4tb" => Ok(Cmd::WaferCxl {
            cxl_tb: flagv(&rest, "--cxl-tb").and_then(|v| v.parse().ok()).unwrap_or(4.0),
        }),
        "photonic-pic" | "kerr-comb" => Ok(Cmd::PhotonicPic {
            lambdas: numv(&rest, "--lambdas", 64) as u32,
        }),
        "neuromorphic-cim" | "fefet-cim" => Ok(Cmd::NeuromorphicCim),
        "zero-trust-sec" | "trojan-scan" => Ok(Cmd::ZeroTrustSec),
        "audit-28x" | "wave4" => Ok(Cmd::Audit28x {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
            cxl_tb: flagv(&rest, "--cxl-tb").and_then(|v| v.parse().ok()).unwrap_or(4.0),
        }),
        "fusa-rad" | "iso26262" | "rad-hard" => Ok(Cmd::FusaRad {
            let_mev: numv(&rest, "--let", 100) as u32,
        }),
        "glass-warp" | "tgv-warp" => Ok(Cmd::GlassWarp {
            pitch_um: flagv(&rest, "--pitch-um").and_then(|v| v.parse().ok()).unwrap_or(3.0),
        }),
        "lean4-prove" | "lean4" | "hol4" => Ok(Cmd::Lean4Prove),
        "aqfp-landauer" | "aqfp" | "rsfq" => Ok(Cmd::AqfpLandauer),
        "subthz-6g" | "6g-dpd" => Ok(Cmd::Subthz6g {
            freq_ghz: numv(&rest, "--freq-ghz", 140) as u32,
        }),
        "optical-rack" | "ocs-64tb" => Ok(Cmd::OpticalRack {
            optical_tb: numv(&rest, "--optical-tb", 64) as u32,
        }),
        "audit-35x" | "wave5" => Ok(Cmd::Audit35x {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
            cxl_tb: flagv(&rest, "--cxl-tb").and_then(|v| v.parse().ok()).unwrap_or(4.0),
            optical_tb: numv(&rest, "--optical-tb", 64) as u32,
        }),
        "spintronic-sot" | "sot-mram" | "skyrmion" => Ok(Cmd::SpintronicSot {
            alpha: flagv(&rest, "--alpha").and_then(|v| v.parse().ok()).unwrap_or(0.012),
        }),
        "diamond-ivr" | "gan-ivr" => Ok(Cmd::DiamondIvr {
            power_w: numv(&rest, "--power-w", 3200) as u32,
        }),
        "bio-organoid" | "hd-mea" | "bci" => Ok(Cmd::BioOrganoid {
            channels: numv(&rest, "--channels", 65536) as u32,
        }),
        "fhe-zkp" | "zk-snark" | "tfhe" => Ok(Cmd::FheZkp),
        "fab-yield-twin" | "inline-apc" => Ok(Cmd::FabYieldTwin),
        "exascale-4pb" | "petabyte-arena" => Ok(Cmd::Exascale4pb {
            exascale_pb: numv(&rest, "--exascale-pb", 4) as u32,
        }),
        "audit-42x" | "wave6" => Ok(Cmd::Audit42x {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
            cxl_tb: flagv(&rest, "--cxl-tb").and_then(|v| v.parse().ok()).unwrap_or(4.0),
            optical_tb: numv(&rest, "--optical-tb", 64) as u32,
            exascale_pb: numv(&rest, "--exascale-pb", 4) as u32,
        }),
        "majorana-topo" | "majorana" | "kitaev" => Ok(Cmd::MajoranaTopo {
            gap_uev: numv(&rest, "--gap-uev", 120) as u32,
        }),
        "plasmonic-thz" | "graphene-spp" => Ok(Cmd::PlasmonicThz {
            ef_ev: flagv(&rest, "--ef-ev").and_then(|v| v.parse().ok()).unwrap_or(0.45),
        }),
        "dna-storage" | "molecular-dna" => Ok(Cmd::DnaStorage),
        "pinn-fno" | "fourier-pinn" => Ok(Cmd::PinnFno),
        "fusion-mhd" | "tokamak-fpga" => Ok(Cmd::FusionMhd {
            fluence: numv(&rest, "--fluence", 24) as u32,
        }),
        "zettascale-1eb" | "orbital-1eb" => Ok(Cmd::Zettascale1eb {
            zettascale_pb: numv(&rest, "--zettascale-pb", 256) as u32,
        }),
        "audit-49x" | "wave7" => Ok(Cmd::Audit49x {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
            cxl_tb: flagv(&rest, "--cxl-tb").and_then(|v| v.parse().ok()).unwrap_or(4.0),
            optical_tb: numv(&rest, "--optical-tb", 64) as u32,
            exascale_pb: numv(&rest, "--exascale-pb", 4) as u32,
            zettascale_pb: numv(&rest, "--zettascale-pb", 256) as u32,
        }),
        "skyrmion-magnon" | "magnonic" => Ok(Cmd::SkyrmionMagnon {
            dmi: flagv(&rest, "--dmi").and_then(|v| v.parse().ok()).unwrap_or(3.2),
        }),
        "petahertz-atto" | "attosecond" | "phz-logic" => Ok(Cmd::PetahertzAtto {
            field: flagv(&rest, "--field").and_then(|v| v.parse().ok()).unwrap_or(1.6),
        }),
        "diamond-nv" | "nv-magnetometry" => Ok(Cmd::DiamondNv),
        "tmdc-sub1nm" | "ws2-negf" => Ok(Cmd::TmdcSub1nm {
            angstrom: flagv(&rest, "--angstrom").and_then(|v| v.parse().ok()).unwrap_or(4.5),
        }),
        "batteryless-soc" | "ambient-rf" => Ok(Cmd::BatterylessSoc),
        "cislunar-16eb" | "quantum-cislunar" => Ok(Cmd::Cislunar16eb {
            cislunar_eb: numv(&rest, "--eb", numv(&rest, "--cislunar-eb", 16)) as u32,
        }),
        "audit-56x" | "wave8" => Ok(Cmd::Audit56x {
            arena_gb: numv(&rest, "--arena-gb", 40) as u32,
            cxl_tb: flagv(&rest, "--cxl-tb").and_then(|v| v.parse().ok()).unwrap_or(4.0),
            optical_tb: numv(&rest, "--optical-tb", 64) as u32,
            exascale_pb: numv(&rest, "--exascale-pb", 4) as u32,
            zettascale_pb: numv(&rest, "--zettascale-pb", 256) as u32,
            cislunar_eb: numv(&rest, "--eb", numv(&rest, "--cislunar-eb", 16)) as u32,
        }),
        _ => {
            bail!("unknown command `{}` — try `vikas help`", cmd);
        }
    }
}

fn main() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info.to_string();
        if msg.contains("Broken pipe") || msg.contains("os error 32") {
            std::process::exit(0);
        }
        default_panic(info);
    }));
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            util::banner();
            println!("{} {}", c("[VIKAS ERROR]", RED), e);
            std::process::exit(2);
        }
    };
    let quiet_json = args.iter().any(|a| a == "--json");
    if !matches!(cmd, Cmd::Help) && !quiet_json {
        util::banner();
    }
    if let Err(e) = dispatch(cmd) {
        println!("{} {}", c("[VIKAS ERROR]", RED), e);
        std::process::exit(1);
    }
}

fn load_dump(p: &std::path::Path) -> Result<vikas_wave::WaveDump> {
    vikas_wave::WaveDump::load(p)
}

fn ensure_files(files: &[PathBuf], what: &str) -> Result<()> {
    if files.is_empty() {
        bail!("no input files given — usage: vikas {} <files... or manifest.f>", what);
    }
    Ok(())
}

/// Parse multi-file project into modules (shared by many commands).
fn parse_project(files: &[PathBuf], top: Option<&str>) -> Result<(Vec<vikas_sim::parser::ModuleDef>, String, Vec<String>, std::collections::HashMap<String, u64>)> {
    ensure_files(files, "project")?;
    let mut ld = vikas_sim::Loader::new();
    let proj = ld.load(files)?;
    let (merged, per_file) = ld.merge_sources(&proj)?;
    let lx = vikas_sim::lexer::Lexer::lex(&merged, "merged");
    let mut p = vikas_sim::parser::Parser::new(&lx.toks, &lx.lines);
    let mods = p.parse_program();
    if mods.is_empty() {
        bail!("no modules parsed from {} file(s)", files.len());
    }
    let consts = p.consts.clone();
    let top_name = match top {
        Some(t) => t.to_string(),
        None => {
            let mut instantiated = std::collections::HashSet::new();
            for m in &mods {
                for i in &m.instances {
                    instantiated.insert(i.module.clone());
                }
            }
            let roots: Vec<&vikas_sim::parser::ModuleDef> = mods.iter().filter(|m| !instantiated.contains(&m.name)).collect();
            if roots.len() == 1 {
                roots[0].name.clone()
            } else {
                mods.last().unwrap().name.clone()
            }
        }
    };
    Ok((mods, top_name, per_file, consts))
}

fn dispatch(cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Help => {
            help::catalog();
            Ok(())
        }
        Cmd::Version => {
            println!("vikas 3.0.0 — VIKAS-KRONOS kernel v3.0 (topo-scheduled), Open-KDB, VIKAS Studio, Fortune AI v4.0, physics twin, 6 native wave formats");
            Ok(())
        }
        Cmd::HelpFormats | Cmd::InfoFormats => {
            help::formats();
            Ok(())
        }
        Cmd::Run { files, top, max_ns, gui } => cmds::cmd_run(files, top, max_ns, gui),
        Cmd::Studio { files, top, max_ns, out, open } => cmds::cmd_studio(files, top, max_ns, out, open),
        Cmd::Quickrun { files } => cmds::cmd_quickrun(files),
        Cmd::Watch { files } => cmds::cmd_watch(files),
        Cmd::Latch { files, json } => cmds::cmd_latch(files, json),
        Cmd::Kdb { files, top } => cmds::cmd_kdb(files, top),
        Cmd::Trace { signal, files } => cmds::cmd_trace(signal, files),
        Cmd::Xtrace { signal, files } => cmds::cmd_xtrace(signal, files),
        Cmd::Cdc { files } => cmds::cmd_cdc(files),
        Cmd::Fsm { files } => cmds::cmd_fsm(files),
        Cmd::Sva { files } => cmds::cmd_sva(files),
        Cmd::Loops { files } => cmds::cmd_loops(files),
        Cmd::Recommend { signal, files } => cmds::cmd_recommend(signal, files),
        Cmd::Why { signal, files } => cmds::cmd_why(signal, files),
        Cmd::Doctor { files } => cmds::cmd_doctor(files),
        Cmd::Schematic { signal, files } => cmds::cmd_schematic(signal, files),
        Cmd::Coverage { dump } => cmds::cmd_coverage(dump),
        Cmd::Tui { dump } => tui::run(dump),
        Cmd::Gui { dump, out } => cmds::cmd_gui(dump, out),
        Cmd::Summary { dump } => cmds::cmd_summary(dump),
        Cmd::Annotate { file, time } => cmds::cmd_annotate(file, time),
        Cmd::Find { cond, dump } => cmds::cmd_find(cond, dump),
        Cmd::Expr { formula, dump } => cmds::cmd_expr(formula, dump),
        Cmd::Diff { a, b } => cmds::cmd_diff(a, b),
        Cmd::Decode { dump } => cmds::cmd_decode(dump),
        Cmd::Disasm { dump } => cmds::cmd_disasm(dump),
        Cmd::Spectrum { dump, bus } => cmds::cmd_spectrum(dump, bus),
        Cmd::Perf { dump } => cmds::cmd_perf(dump),
        Cmd::Radix { value, kind } => cmds::cmd_radix(value, kind),
        Cmd::Memdump { dump, array, time } => cmds::cmd_memdump(dump, array, time),
        Cmd::ExportDoc { dump, from, to } => cmds::cmd_export_doc(dump, from, to),
        Cmd::ExportTb { dump } => cmds::cmd_export_tb(dump),
        Cmd::Convert { input, output } => cmds::cmd_convert(input, output),
        Cmd::Anomaly { dump } => cmds::cmd_anomaly(dump),
        Cmd::Delta { a, b } => cmds::cmd_delta(a, b),
        Cmd::Cycles { time, period } => cmds::cmd_cycles(time, period),
        Cmd::Physics { files, vdd, temp } => cmds::cmd_physics(files, vdd, temp),
        Cmd::Energy { files, vdds } => cmds::cmd_energy(files, vdds),
        Cmd::Sweep { vdds, temp } => cmds::cmd_sweep(vdds, temp),
        Cmd::Pbit { dump } => cmds::cmd_pbit(dump),
        Cmd::Cosim { bits } => cmds::cmd_cosim(bits),
        Cmd::Egraph { expr } => cmds::cmd_egraph(expr),
        Cmd::Hdc { dump } => cmds::cmd_hdc(dump),
        Cmd::Taint { src, dst, files } => cmds::cmd_taint(src, dst, files),
        Cmd::Topo => cmds::cmd_topo(),
        Cmd::Fork { n } => cmds::cmd_fork(n),
        Cmd::Shm => cmds::cmd_shm(),
        Cmd::Mcp { workdir } => cmds::cmd_mcp(workdir),
        Cmd::Wcp { addr } => vikas_wcp::run_wcp_server(&addr),
        Cmd::Session => cmds::cmd_session(),
        Cmd::Ci { files } => cmds::cmd_ci(files),
        Cmd::GateCompare { rtl, gate } => cmds::cmd_gate_compare(rtl, gate),
        Cmd::AutogenTb { file } => cmds::cmd_autogen_tb(file),
        Cmd::Editor { file } => cmds::cmd_editor(file),
        Cmd::Mutant { files, top, max_ns, max_mutants, min_score, json } => cmds::cmd_mutant(files, top, max_ns, max_mutants, min_score, json),
        Cmd::Power { dump, vdd, cnode_ff, freq_mhz, json } => cmds::cmd_power(dump, vdd, cnode_ff, freq_mhz, json),
        Cmd::Bench { scale, dir } => bench::cmd_bench(scale, dir),
        Cmd::Signoff { files, top, max_ns, mutant, json } => cmds::cmd_signoff(files, top, max_ns, mutant, json),
        Cmd::Multicore { threads, profile } => cmds::cmd_multicore(threads, profile),
        Cmd::Uvm { files, threads } => cmds::cmd_uvm(files, threads),
        Cmd::CompileAll => cmds::cmd_compile_all(),
        Cmd::Batch { files, jobs, json } => cmds::cmd_batch(files, jobs, json),
        Cmd::Agentic { files, epochs } => cmds::cmd_agentic(files, epochs),
        Cmd::Circt { files } => cmds::cmd_circt(files),
        Cmd::Horizon { vdd, tiers, cpo } => cmds::cmd_horizon(vdd, tiers, cpo),
        Cmd::Audit7x => cmds::cmd_audit_7x(),
        Cmd::Emul { target, clock_mhz } => cmds::cmd_emul(target, clock_mhz),
        Cmd::Hls { target, unroll } => cmds::cmd_hls(target, unroll),
        Cmd::DsoPpa { gates_m, freq_ghz, retiming_stages } => cmds::cmd_dso_ppa(gates_m, freq_ghz, retiming_stages),
        Cmd::PqcSec { glitch_mv } => cmds::cmd_pqc_sec(glitch_mv),
        Cmd::Rdma1024c { nodes } => cmds::cmd_rdma_1024c(nodes),
        Cmd::Audit14x => cmds::cmd_audit_14x(),
        Cmd::Hyperscale { arena_gb, gates_b } => cmds::cmd_hyperscale(arena_gb, gates_b),
        Cmd::EuvIlt { na, power_w } => cmds::cmd_euv_ilt(na, power_w),
        Cmd::MaxwellEm { power_w } => cmds::cmd_maxwell_em(power_w),
        Cmd::ChipArch { mesh_dim } => cmds::cmd_chip_arch(mesh_dim),
        Cmd::Analog6Sigma { sigma } => cmds::cmd_analog_6sigma(sigma),
        Cmd::FuzzIsa { mesh_dim } => cmds::cmd_fuzz_isa(mesh_dim),
        Cmd::Audit21x { arena_gb } => cmds::cmd_audit_21x(arena_gb),
        Cmd::TcadNegf { lg_nm } => cmds::cmd_tcad_negf(lg_nm),
        Cmd::QecFtqc { distance } => cmds::cmd_qec_ftqc(distance),
        Cmd::WaferCxl { cxl_tb } => cmds::cmd_wafer_cxl(cxl_tb),
        Cmd::PhotonicPic { lambdas } => cmds::cmd_photonic_pic(lambdas),
        Cmd::NeuromorphicCim => cmds::cmd_neuromorphic_cim(),
        Cmd::ZeroTrustSec => cmds::cmd_zero_trust_sec(),
        Cmd::Audit28x { arena_gb, cxl_tb } => cmds::cmd_audit_28x(arena_gb, cxl_tb),
        Cmd::FusaRad { let_mev } => cmds::cmd_fusa_rad(let_mev),
        Cmd::GlassWarp { pitch_um } => cmds::cmd_glass_warp(pitch_um),
        Cmd::Lean4Prove => cmds::cmd_lean4_prove(),
        Cmd::AqfpLandauer => cmds::cmd_aqfp_landauer(),
        Cmd::Subthz6g { freq_ghz } => cmds::cmd_subthz_6g(freq_ghz),
        Cmd::OpticalRack { optical_tb } => cmds::cmd_optical_rack(optical_tb),
        Cmd::Audit35x { arena_gb, cxl_tb, optical_tb } => cmds::cmd_audit_35x(arena_gb, cxl_tb, optical_tb),
        Cmd::SpintronicSot { alpha } => cmds::cmd_spintronic_sot(alpha),
        Cmd::DiamondIvr { power_w } => cmds::cmd_diamond_ivr(power_w),
        Cmd::BioOrganoid { channels } => cmds::cmd_bio_organoid(channels),
        Cmd::FheZkp => cmds::cmd_fhe_zkp(),
        Cmd::FabYieldTwin => cmds::cmd_fab_yield_twin(),
        Cmd::Exascale4pb { exascale_pb } => cmds::cmd_exascale_4pb(exascale_pb),
        Cmd::Audit42x { arena_gb, cxl_tb, optical_tb, exascale_pb } => cmds::cmd_audit_42x(arena_gb, cxl_tb, optical_tb, exascale_pb),
        Cmd::MajoranaTopo { gap_uev } => cmds::cmd_majorana_topo(gap_uev),
        Cmd::PlasmonicThz { ef_ev } => cmds::cmd_plasmonic_thz(ef_ev),
        Cmd::DnaStorage => cmds::cmd_dna_storage(),
        Cmd::PinnFno => cmds::cmd_pinn_fno(),
        Cmd::FusionMhd { fluence } => cmds::cmd_fusion_mhd(fluence),
        Cmd::Zettascale1eb { zettascale_pb } => cmds::cmd_zettascale_1eb(zettascale_pb),
        Cmd::Audit49x { arena_gb, cxl_tb, optical_tb, exascale_pb, zettascale_pb } => cmds::cmd_audit_49x(arena_gb, cxl_tb, optical_tb, exascale_pb, zettascale_pb),
        Cmd::SkyrmionMagnon { dmi } => cmds::cmd_skyrmion_magnon(dmi),
        Cmd::PetahertzAtto { field } => cmds::cmd_petahertz_atto(field),
        Cmd::DiamondNv => cmds::cmd_diamond_nv(),
        Cmd::TmdcSub1nm { angstrom } => cmds::cmd_tmdc_sub1nm(angstrom),
        Cmd::BatterylessSoc => cmds::cmd_batteryless_soc(),
        Cmd::Cislunar16eb { cislunar_eb } => cmds::cmd_cislunar_16eb(cislunar_eb),
        Cmd::Audit56x { arena_gb, cxl_tb, optical_tb, exascale_pb, zettascale_pb, cislunar_eb } => cmds::cmd_audit_56x(arena_gb, cxl_tb, optical_tb, exascale_pb, zettascale_pb, cislunar_eb),
    }
}

pub fn sim_and_dump(files: &[PathBuf], top: Option<&str>, max_ns: u64) -> Result<(vikas_sim::vm::SimResult, Vec<String>)> {
    let (mods, top_name, per_file, consts) = parse_project(files, top)?;
    let mut vm = vikas_sim::vm::Vm::new(mods, &top_name, &consts);
    let res = vm.run(max_ns);
    Ok((res, per_file))
}
