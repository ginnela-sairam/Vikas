//! # vikas-cli — the `vikas` executable: 55 subcommands across simulation,
//! Open-KDB debug, physics, waveform analysis, Linux-native machinery,
//! AI/MCP servers, GUI/TUI, and project tooling. Zero Python.

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
    Quickrun { files: Vec<PathBuf> },
    Watch { files: Vec<PathBuf> },
    Latch { files: Vec<PathBuf>, json: bool },
    Help,
    Version,
    HelpFormats,
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
    Power { dump: PathBuf, vdd: f64, cnode_ff: f64, freq_mhz: f64, max_ns: u64, json: bool },
    Signoff { files: Vec<PathBuf>, top: Option<String>, max_ns: u64, mutant: bool, json: bool },
}

fn parse_args(args: &[String]) -> Result<Cmd> {
    let mut it = args.iter();
    let cmd = it.next().unwrap_or(&"help".into()).clone();
    let rest: Vec<String> = it.cloned().collect();
    let files = |r: &[String]| -> Vec<PathBuf> {
        const VALUED: &[&str] = &[
            "--top", "--max-ns", "-o", "--time", "--from", "--to", "--array",
            "--dump", "--addr", "--vdd", "--temp-k", "--bus",
            "--max-mutants", "--max", "--min-score", "--cnode", "--freq-mhz",
        ];
        let mut out = Vec::new();
        let mut skip_next = false;
        for s in r {
            if skip_next {
                skip_next = false;
                continue;
            }
            if VALUED.contains(&s.as_str()) {
                skip_next = true;
                continue;
            }
            if !s.starts_with('-') {
                out.push(PathBuf::from(s));
            }
        }
        out
    };
    let flagv = |r: &[String], name: &str| -> Option<String> {
        r.iter().position(|s| s == name).and_then(|i| r.get(i + 1)).cloned()
    };
    let numv = |r: &[String], name: &str, d: u64| -> u64 {
        flagv(r, name).and_then(|v| v.parse().ok()).unwrap_or(d)
    };
    match cmd.as_str() {
        "run" => Ok(Cmd::Run {
            files: files(&rest),
            top: flagv(&rest, "--top"),
            max_ns: numv(&rest, "--max-ns", 100_000),
            gui: rest.contains(&"--gui".into()),
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
            out: flagv(&rest, "-o").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("vikas_gui.html")),
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
            max_mutants: flagv(&rest, "--max-mutants").or_else(|| flagv(&rest, "--max"))
                .and_then(|v| v.parse().ok()).unwrap_or(200) as usize,
            min_score: flagv(&rest, "--min-score").and_then(|v| v.parse().ok()).unwrap_or(70.0),
            json: rest.contains(&"--json".into()),
        }),
        "power" => Ok(Cmd::Power {
            // positional RTL source or dump; flags must not be mistaken for a path
            dump: files(&rest).first().cloned().unwrap_or_else(|| PathBuf::from(".vikas/vikas_wave.vwf")),
            max_ns: numv(&rest, "--max-ns", 50_000),
            vdd: flagv(&rest, "--vdd").and_then(|v| v.parse().ok()).unwrap_or(0.8),
            cnode_ff: flagv(&rest, "--cnode").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            freq_mhz: flagv(&rest, "--freq-mhz").and_then(|v| v.parse().ok()).unwrap_or(100.0),
            json: rest.contains(&"--json".into()),
        }),
        "signoff" => Ok(Cmd::Signoff {
            files: files(&rest),
            top: flagv(&rest, "--top"),
            max_ns: numv(&rest, "--max-ns", 50_000),
            mutant: rest.contains(&"--mutant".into()),
            json: rest.contains(&"--json".into()),
        }),
        _ => {
            bail!("unknown command `{}` — try `vikas help`", cmd);
        }
    }
}

fn main() {
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
            println!("vikas 1.0.0 — XEZIM kernel, Open-KDB, physics twin, 6 native wave formats");
            Ok(())
        }
        Cmd::HelpFormats | Cmd::InfoFormats => {
            help::formats();
            Ok(())
        }
        Cmd::Run { files, top, max_ns, gui } => cmds::cmd_run(files, top, max_ns, gui),
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
        Cmd::Power { dump, vdd, cnode_ff, freq_mhz, max_ns, json } => cmds::cmd_power(dump, vdd, cnode_ff, freq_mhz, max_ns, json),
        Cmd::Signoff { files, top, max_ns, mutant, json } => cmds::cmd_signoff(files, top, max_ns, mutant, json),
    }
}

pub fn sim_and_dump(files: &[PathBuf], top: Option<&str>, max_ns: u64) -> Result<(vikas_sim::vm::SimResult, Vec<String>)> {
    let (mods, top_name, per_file, consts) = parse_project(files, top)?;
    let mut vm = vikas_sim::vm::Vm::new(mods, &top_name, &consts);
    let res = vm.run(max_ns);
    Ok((res, per_file))
}
