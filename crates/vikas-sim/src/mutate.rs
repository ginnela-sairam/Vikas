//! VIKAS v1 — BREAKTHROUGH #14: RTL mutation testing engine.
//!
//! Industry-first for a CLI EDA tool: systematically injects single-point
//! design mutations (operator swaps, constant zeroing, off-by-one) into the
//! DUT (every module reachable from the top EXCEPT the testbench itself),
//! re-simulates each mutant against the golden run, and scores how many the
//! testbench KILLS. Surviving mutants are precise, actionable testbench-gap
//! reports — the hardware analog of software mutation testing (mutmut /
//! cosmic-ray), which no major commercial simulator offers CLI-native.

use crate::parser::{Expr, ModuleDef, Stmt};
use crate::vm::SimResult;
use std::collections::{HashMap, HashSet};

/// Mutation operator table: op -> replacement.
pub fn swap_op(op: &str) -> Option<&'static str> {
    Some(match op {
        "+" => "-",
        "-" => "+",
        "&" => "|",
        "|" => "&",
        "&&" => "||",
        "||" => "&&",
        "^" => "&",
        "==" => "!=",
        "!=" => "==",
        "<" => ">=",
        ">=" => "<",
        ">" => "<=",
        "<=" => ">",
        "<<" => ">>",
        ">>" => "<<",
        _ => return None,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum MutKind {
    OpSwap,
    ConstZero,
    ConstOffByOne,
}

impl MutKind {
    pub fn label(&self) -> &'static str {
        match self {
            MutKind::OpSwap => "op-swap",
            MutKind::ConstZero => "const-zero",
            MutKind::ConstOffByOne => "off-by-one",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MutSite {
    pub module: String,
    pub kind: MutKind,
    /// human description, e.g. `a + b` -> `a - b`
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Killed,
    Survived,
    /// mutant diverged so much the sim never reached $finish inside max_ns
    Hang,
}

#[derive(Clone, Debug)]
pub struct MutantResult {
    pub site: MutSite,
    pub verdict: Verdict,
    /// first divergence description (killed/hang)
    pub first_diff: Option<String>,
    pub wall_ms: u128,
}

/// Modules reachable from `top` through instantiation (the DUT closure —
/// the top module itself is the testbench and is never mutated).
pub fn dut_closure(mods: &[ModuleDef], top: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut stack = vec![top.to_string()];
    while let Some(m) = stack.pop() {
        if let Some(md) = mods.iter().find(|x| x.name == m) {
            for inst in &md.instances {
                if out.insert(inst.module.clone()) {
                    stack.push(inst.module.clone());
                }
            }
        }
    }
    out
}

/// Count total mutation sites (deterministic depth-first order).
pub fn count_sites(mods: &[ModuleDef], dut: &HashSet<String>) -> usize {
    let mut k = 0usize;
    let mut tmp = mods.to_vec();
    visit(&mut tmp, dut, &mut |_, _, _| {
        k += 1;
    });
    k
}

/// Apply the k-th mutation (0-based, same deterministic order as count_sites).
/// Returns the site description, or None if k is out of range.
pub fn apply_kth(mods: &mut [ModuleDef], dut: &HashSet<String>, k: usize) -> Option<MutSite> {
    let mut k = k;
    let mut hit: Option<MutSite> = None;
    visit(mods, dut, &mut |m, e, kind| {
        if hit.is_some() {
            return;
        }
        if k > 0 {
            k -= 1;
            return;
        }
        match kind {
            MutKind::OpSwap => {
                let prep = match e {
                    Expr::Bin(op, l, r) => {
                        let before = format!("({} {} {})", expr_text(l), op, expr_text(r));
                        Some((op.to_string(), (**l).clone(), (**r).clone(), before))
                    }
                    _ => None,
                };
                if let Some((op, l2, r2, before)) = prep {
                    let newop = swap_op(&op).unwrap_or(op.as_str()).to_string();
                    *e = Expr::Bin(newop.clone(), Box::new(l2), Box::new(r2));
                    hit = Some(MutSite {
                        module: m.to_string(),
                        kind: MutKind::OpSwap,
                        detail: format!("`{}` op `{}` -> `{}`", before, op, newop),
                    });
                }
            }
            MutKind::ConstZero => {
                if let Expr::Num(v, line) = e {
                    let old = *v;
                    *e = Expr::Num(0, *line);
                    hit = Some(MutSite {
                        module: m.to_string(),
                        kind: MutKind::ConstZero,
                        detail: format!("literal {} -> 0", old),
                    });
                }
            }
            MutKind::ConstOffByOne => {
                if let Expr::Num(v, line) = e {
                    let old = *v;
                    *e = Expr::Num(old.wrapping_add(1), *line);
                    hit = Some(MutSite {
                        module: m.to_string(),
                        kind: MutKind::ConstOffByOne,
                        detail: format!("literal {} -> {}", old, old.wrapping_add(1)),
                    });
                }
            }
        }
    });
    hit
}

/// Deterministic visitor over every mutable expression node in DUT modules.
/// Callback args: (module name, mutable expr ref, mutation kind at this node).
/// A Bin node with a swappable operator is ONE OpSwap site; a Num node with a
/// non-zero value yields TWO sites (ConstZero then ConstOffByOne).
fn visit(
    mods: &mut [ModuleDef],
    dut: &HashSet<String>,
    f: &mut dyn FnMut(&str, &mut Expr, MutKind),
) {
    // NOTE: the closure receives each site once per mutation at that node.
    // We simulate "two sites per Num" by calling f twice with the two kinds.
    for m in mods.iter_mut() {
        if !dut.contains(&m.name) {
            continue;
        }
        let mname = m.name.clone();
        for stmt in m.signals.iter_mut() {
            if let Stmt::Decl { init: Some(e), .. } = stmt {
                visit_expr(e, &mname, f);
            }
        }
        for a in m.assigns.iter_mut() {
            visit_expr(&mut a.1, &mname, f);
        }
        for ab in m.always.iter_mut() {
            let mut body = std::mem::take(&mut ab.body);
            for st in body.iter_mut() {
                visit_stmt(st, &mname, f);
            }
            ab.body = body;
        }
    }
}

fn visit_stmt(s: &mut Stmt, m: &str, f: &mut dyn FnMut(&str, &mut Expr, MutKind)) {
    match s {
        Stmt::Assign { lhs: _, rhs, .. } => visit_expr(rhs, m, f),
        Stmt::If { cond, then, els } => {
            visit_expr(cond, m, f);
            for st in then.iter_mut() { visit_stmt(st, m, f); }
            for st in els.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::Case { subject, arms, default } => {
            visit_expr(subject, m, f);
            for (labels, body) in arms.iter_mut() {
                for e in labels.iter_mut() { visit_expr(e, m, f); }
                for st in body.iter_mut() { visit_stmt(st, m, f); }
            }
            if let Some(body) = default {
                for st in body.iter_mut() { visit_stmt(st, m, f); }
            }
        }
        Stmt::For { init, cond, step, body } => {
            visit_expr(init, m, f);
            visit_expr(cond, m, f);
            visit_expr(step, m, f);
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::While { cond, body } => {
            visit_expr(cond, m, f);
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::Repeat { count, body } => {
            visit_expr(count, m, f);
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::Delay { body, .. } => {
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::Event { body, .. } => {
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::Wait { cond, body } => {
            visit_expr(cond, m, f);
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::Block(body) => {
            for st in body.iter_mut() { visit_stmt(st, m, f); }
        }
        Stmt::FCall(_, args) | Stmt::SysTask(_, args) => {
            for a in args.iter_mut() { visit_expr(a, m, f); }
        }
        _ => {}
    }
}

fn visit_expr(e: &mut Expr, m: &str, f: &mut dyn FnMut(&str, &mut Expr, MutKind)) {
    match e {
        Expr::Bin(op, l, r) => {
            let swappable = swap_op(op).is_some();
            if swappable {
                f(m, e, MutKind::OpSwap);
            }
            // re-borrow AFTER a possible mutation (the swap replaces children
            // with clones) so nested sites keep their deterministic order
            if let Expr::Bin(_, l, r) = e {
                visit_expr(l, m, f);
                visit_expr(r, m, f);
            }
        }
        Expr::Un(_, x) => visit_expr(x, m, f),
        Expr::Index(b, i) => {
            visit_expr(b, m, f);
            visit_expr(i, m, f);
        }
        Expr::Slice(b, h, l) => {
            visit_expr(b, m, f);
            visit_expr(h, m, f);
            visit_expr(l, m, f);
        }
        Expr::Call(_, args) => {
            for a in args.iter_mut() { visit_expr(a, m, f); }
        }
        Expr::Concat(items) => {
            for it in items.iter_mut() { visit_expr(it, m, f); }
        }
        Expr::Replicate(n, v) => {
            visit_expr(n, m, f);
            visit_expr(v, m, f);
        }
        Expr::Ternary(c, a, b) => {
            visit_expr(c, m, f);
            visit_expr(a, m, f);
            visit_expr(b, m, f);
        }
        Expr::Num(v, _) => {
            if *v != 0 {
                f(m, e, MutKind::ConstZero);
                f(m, e, MutKind::ConstOffByOne);
            }
        }
        _ => {}
    }
}

/// Small expression renderer for site reports.
pub fn expr_text(e: &Expr) -> String {
    match e {
        Expr::Num(v, _) => v.to_string(),
        Expr::Str(s) => format!("\"{}\"", s),
        Expr::Ident(n) => n.clone(),
        Expr::Bin(op, l, r) => format!("({} {} {})", expr_text(l), op, expr_text(r)),
        Expr::Un(op, x) => format!("{}{}", op, expr_text(x)),
        Expr::Index(b, i) => format!("{}[{}]", expr_text(b), expr_text(i)),
        Expr::Slice(b, h, l) => format!("{}[{}:{}]", expr_text(b), expr_text(h), expr_text(l)),
        Expr::Call(n, args) => format!("{}({})", n, args.iter().map(expr_text).collect::<Vec<_>>().join(", ")),
        Expr::Concat(items) => format!("{{{}}}", items.iter().map(expr_text).collect::<Vec<_>>().join(",")),
        Expr::Replicate(n, v) => format!("{{{}{{{}}}}}", expr_text(n), expr_text(v)),
        Expr::Ternary(c, a, b) => format!("({} ? {} : {})", expr_text(c), expr_text(a), expr_text(b)),
    }
}

/// Full mutation-testing campaign. Returns per-mutant results and the score
/// in [0,1]: killed / (killed + survived). Hangs count as killed (the design
/// observably changed); rejected/non-viable mutants don't exist in this
/// scheme (the kernel always runs; divergent mutants hang into max_ns).
pub fn run_mutation_testing(
    mods: &[ModuleDef],
    top: &str,
    consts: &HashMap<String, u64>,
    max_ns: u64,
    max_mutants: usize,
) -> anyhow::Result<(Vec<MutantResult>, f64)> {
    let dut = dut_closure(mods, top);
    let total = count_sites(mods, &dut);
    if total == 0 {
        anyhow::bail!("no mutable sites found under top `{}`", top);
    }
    // golden run
    let golden = {
        let mut vm = crate::vm::Vm::new(mods.to_vec(), top, consts);
        vm.quiet = true;
        vm.run(max_ns)
    };
    let n = total.min(max_mutants);
    let mut results = Vec::with_capacity(n);
    let mut killed = 0usize;
    let mut survived = 0usize;
    for k in 0..n {
        let mut mmods = mods.to_vec();
        let site = match apply_kth(&mut mmods, &dut, k) {
            Some(s) => s,
            None => break,
        };
        let mut vm = crate::vm::Vm::new(mmods, top, consts);
        vm.quiet = true;
        let res = vm.run(max_ns);
        let (verdict, diff) = compare(&golden, &res, max_ns);
        match verdict {
            Verdict::Killed | Verdict::Hang => killed += 1,
            Verdict::Survived => survived += 1,
        }
        results.push(MutantResult { site, verdict, first_diff: diff, wall_ms: res.wall_ms });
    }
    let score = if killed + survived == 0 { 0.0 } else { killed as f64 / (killed + survived) as f64 };
    Ok((results, score))
}

/// Equivalence oracle: printed output first (testbench verdicts), then the
/// full waveform (per-signal change timelines), then completion time.
fn compare(golden: &SimResult, mutant: &SimResult, max_ns: u64) -> (Verdict, Option<String>) {
    // 1. printed output differences are the strongest TB signal
    for (i, g) in golden.printed.iter().enumerate() {
        match mutant.printed.get(i) {
            Some(m) if m == g => {}
            Some(m) => return (Verdict::Killed, Some(format!("print[{}]: `{}` -> `{}`", i, g.trim(), m.trim()))),
            None => return (Verdict::Killed, Some(format!("print[{}]: `{}` -> (missing)", i, g.trim()))),
        }
    }
    if mutant.printed.len() > golden.printed.len() {
        let extra = mutant.printed[golden.printed.len()..].iter().map(|s| s.trim()).collect::<Vec<_>>().join(" | ");
        return (Verdict::Killed, Some(format!("extra prints: {}", extra)));
    }
    // 2. waveform: per-signal change timelines
    let gsteps = golden.dump.steps();
    let msteps = mutant.dump.steps();
    let mut names: Vec<&String> = golden.dump.signals.iter().map(|s| &s.name).collect();
    names.sort();
    for name in names {
        let gi = golden.dump.signals.iter().position(|s| &s.name == name);
        let mi = mutant.dump.signals.iter().position(|s| &s.name == name);
        let g = gi.map(|i| &gsteps[i]).map(|v| v.as_slice()).unwrap_or(&[]);
        let m = match mi {
            Some(i) if i < msteps.len() => msteps[i].as_slice(),
            _ => &[],
        };
        if g.len() != m.len() {
            return (Verdict::Killed, Some(format!("{}: {} transitions -> {}", name, g.len(), m.len())));
        }
        for (a, b) in g.iter().zip(m.iter()) {
            if a.0 != b.0 || a.1 != b.1 {
                return (Verdict::Killed, Some(format!("{} @ t={}: {} -> {}", name, a.0, a.1.to_hex_str(), b.1.to_hex_str())));
            }
        }
    }
    // 3. completion: golden finished early, mutant slammed into max_ns -> hang
    if golden.dump.end_time() + 100 < max_ns && mutant.dump.end_time() >= max_ns {
        return (Verdict::Hang, Some(format!("no $finish within {} ns (golden ended at {})", max_ns, golden.dump.end_time())));
    }
    (Verdict::Survived, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulate_source;

    const ALU_TB: &str = r#"
        module adder(
            input  logic [7:0] a, b,
            output logic [7:0] sum,
            output logic       eq
        );
            assign sum = a + b;
            assign eq  = (a == b);
        endmodule
        module tb;
            logic [7:0] a, b, sum;
            logic eq;
            adder dut(.a(a), .b(b), .sum(sum), .eq(eq));
            logic pass;
            initial begin
                a = 8'd10; b = 8'd20; pass = 0;
                #10;
                if (sum == 8'd30) pass = 1;
                if (sum != 8'd30) $display("TB-FAIL sum=%0d", sum);
                if (eq != 1'b0)   $display("TB-FAIL eq=%0d", eq);
                a = 8'd7; b = 8'd7;
                #10;
                if (eq != 1'b1)   $display("TB-FAIL eq2=%0d", eq);
                if (pass) $display("TB-PASS");
                $finish;
            end
        endmodule
    "#;

    #[test]
    fn mutation_kills_op_swap_and_reports_score() {
        // simulate_source gives us a SimResult, but the campaign needs the
        // parsed mods: re-parse via the same path.
        let lx = crate::lexer::Lexer::lex(ALU_TB, "t");
        let mut p = crate::parser::Parser::new(&lx.toks, &lx.lines);
        let mods = p.parse_program();
        let consts = p.consts.clone();
        let (results, score) = run_mutation_testing(&mods, "tb", &consts, 200, 64).unwrap();
        assert!(!results.is_empty(), "must find mutable sites in the DUT");
        // DUT only: the TB's own `+ 1`-style sites must NOT be mutated
        for r in &results {
            assert_ne!(r.site.module, "tb", "testbench must never be mutated");
        }
        // a + b -> a - b MUST be killed (10+20=30 vs 10-20=236)
        assert!(results.iter().any(|r| r.verdict == Verdict::Killed && r.site.detail.contains("+")),
            "op-swap of the adder must be killed");
        assert!(score > 0.0);
    }

    #[test]
    fn dut_closure_excludes_tb() {
        let lx = crate::lexer::Lexer::lex(ALU_TB, "t");
        let mut p = crate::parser::Parser::new(&lx.toks, &lx.lines);
        let mods = p.parse_program();
        let c = dut_closure(&mods, "tb");
        assert!(c.contains("adder"));
        assert!(!c.contains("tb"));
    }

    #[test]
    fn deterministic_site_counting() {
        let lx = crate::lexer::Lexer::lex(ALU_TB, "t");
        let mut p = crate::parser::Parser::new(&lx.toks, &lx.lines);
        let mods = p.parse_program();
        let c = dut_closure(&mods, "tb");
        let n1 = count_sites(&mods, &c);
        let n2 = count_sites(&mods, &c);
        assert_eq!(n1, n2);
        // DUT has exactly two swappable operators (`a + b`, `a == b`);
        // the TB's constants must NOT be counted (TB is never mutated)
        assert_eq!(n1, 2, "expected exactly the 2 DUT op-swap sites, got {}", n1);
    }

    #[test]
    fn survivor_is_detected_when_tb_blind() {
        // TB never checks `eq` -> the == -> != swap must SURVIVE
        let blind = r#"
        module cmp(input logic [3:0] a, output logic y);
            assign y = (a == 4'd5);
        endmodule
        module tb;
            logic [3:0] a;
            logic y;
            cmp dut(.a(a), .y(y));
            initial begin
                a = 4'd1;
                #10;
                $display("TB-DONE");
                $finish;
            end
        endmodule
        "#;
        let lx = crate::lexer::Lexer::lex(blind, "t");
        let mut p = crate::parser::Parser::new(&lx.toks, &lx.lines);
        let mods = p.parse_program();
        let consts = p.consts.clone();
        let (results, _score) = run_mutation_testing(&mods, "tb", &consts, 100, 64).unwrap();
        assert!(results.iter().any(|r| r.verdict == Verdict::Survived),
            "an unchecked comparison must survive (testbench gap report)");
    }

    #[test]
    fn printed_capture_works() {
        let res = simulate_source("module t; initial begin $display(\"HELLO %0d\", 42); $finish; end endmodule", None, 50).unwrap();
        assert!(res.printed.iter().any(|l| l.contains("HELLO 42")), "printed={:?}", res.printed);
    }
}
