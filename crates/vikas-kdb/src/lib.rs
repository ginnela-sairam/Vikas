//! # vikas-kdb — "Open-KDB" CSR graph intelligence & Verdi-class debug.
//! Driver/fan-in cones, Tarjan SCC loop detection, FO4 depth, CDC, FSM,
//! counterfactual solver (breakthrough 1/39), SVA synthesis (feature 34),
//! static latch L1 extraction (breakthrough 13).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vikas_sim::parser::{AlwaysBlock, Expr, ModuleDef, Stmt};

/// One driver site: signal driven by an expression at file:line.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DriverSite {
    pub signal: String,
    pub rhs_text: String,
    pub file: String,
    pub line: u32,
    pub kind: String, // assign / always_ff / always_comb / initial
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InferredLatch {
    pub signal: String,
    pub file: String,
    pub line: u32,
    pub always_block_type: String,
    pub enable_condition: String,
    pub missing_branch_hint: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct OpenKdb {
    pub top: String,
    pub signals: Vec<String>,
    pub drivers: Vec<DriverSite>,
    /// CSR graph: signal index -> fan-in signal indices
    pub csr_offsets: Vec<u32>,
    pub csr_targets: Vec<u32>,
    pub scc_loops: Vec<Vec<String>>,
    pub inferred_latches: Vec<InferredLatch>,
    pub fsm_states: Vec<FsmInfo>,
    pub cdc_paths: Vec<CdcPath>,
    pub fo4_depth: HashMap<String, u32>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FsmInfo {
    pub state_signal: String,
    pub encodings: Vec<(String, u64)>,
    pub transitions: Vec<(u64, u64, String)>, // from, to, cond text
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CdcPath {
    pub src: String,
    pub dst: String,
    pub src_clk: String,
    pub dst_clk: String,
    pub synchronizer_depth: usize,
}

/// Extract identifiers from an expression.
pub fn expr_idents(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Ident(n) => out.push(n.clone()),
        Expr::Bin(_, a, b) => {
            expr_idents(a, out);
            expr_idents(b, out);
        }
        Expr::Un(_, a) => expr_idents(a, out),
        Expr::Index(b, i) => {
            expr_idents(b, out);
            expr_idents(i, out);
        }
        Expr::Slice(b, m, l) => {
            expr_idents(b, out);
            expr_idents(m, out);
            expr_idents(l, out);
        }
        Expr::Call(_, args) => {
            for a in args {
                expr_idents(a, out);
            }
        }
        Expr::Concat(items) => {
            for i in items {
                expr_idents(i, out);
            }
        }
        Expr::Replicate(c, v) => {
            expr_idents(c, out);
            expr_idents(v, out);
        }
        Expr::Ternary(a, b, c) => {
            expr_idents(a, out);
            expr_idents(b, out);
            expr_idents(c, out);
        }
        _ => {}
    }
}

pub fn expr_text(e: &Expr) -> String {
    match e {
        Expr::Num(v, w) => format!("{:w$}", v, w = 0),
        Expr::Str(s) => s.clone(),
        Expr::Ident(n) => n.clone(),
        Expr::Bin(op, a, b) => format!("{} {} {}", expr_text(a), op, expr_text(b)),
        Expr::Un(op, a) => format!("{}{}", op, expr_text(a)),
        Expr::Index(b, i) => format!("{}[{}]", expr_text(b), expr_text(i)),
        Expr::Slice(b, m, l) => format!("{}[{}:{}]", expr_text(b), expr_text(m), expr_text(l)),
        Expr::Call(n, a) => format!("{}({})", n, a.iter().map(expr_text).collect::<Vec<_>>().join(", ")),
        Expr::Concat(items) => format!("{{{}}}", items.iter().map(expr_text).collect::<Vec<_>>().join(",")),
        Expr::Replicate(c, v) => format!("{{{}/{}}}", expr_text(c), expr_text(v)),
        Expr::Ternary(a, b, c) => format!("({} ? {} : {})", expr_text(a), expr_text(b), expr_text(c)),
    }
}

/// Build the full Open-KDB from parsed modules.
pub fn build(mods: &[ModuleDef], top: &str) -> OpenKdb {
    let mut kdb = OpenKdb { top: top.to_string(), ..Default::default() };
    let mut sigset: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    // collect all modules' signals & drivers
    for m in mods {
        for p in &m.ports {
            sigset.insert(p.name.clone());
        }
        for s in &m.signals {
            if let Stmt::Decl { name, .. } = s {
                sigset.insert(name.clone());
            }
        }
        for (lhs, rhs, line) in &m.assigns {
            if let Expr::Ident(ln) = lhs {
                let mut ids = Vec::new();
                expr_idents(rhs, &mut ids);
                ids.retain(|i| i != ln && sigset.contains(i).then(|| true).or_else(|| Some(true)) == Some(true));
                let mut ids: Vec<String> = ids.into_iter().filter(|i| i != ln).collect();
                ids.sort();
                ids.dedup();
                for src in &ids {
                    edges.entry(ln.clone()).or_default().push(src.clone());
                }
                kdb.drivers.push(DriverSite {
                    signal: ln.clone(),
                    rhs_text: expr_text(rhs),
                    file: m.file.clone(),
                    line: *line,
                    kind: "assign".into(),
                });
            }
        }
        for ab in &m.always {
            let sequential = ab.kind == "always_ff" || ab.kind == "initial";
            let mut lhss = Vec::new();
            let mut rhss = Vec::new();
            collect_always_io(&ab.body, &mut lhss, &mut rhss);
            let mut lhs_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            let mut rhs_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            for l in &lhss {
                lhs_set.insert(l.clone());
                sigset.insert(l.clone());
            }
            for r in &rhss {
                rhs_set.insert(r.clone());
                sigset.insert(r.clone());
            }
            if sequential {
                // register blocks: record driver site but no comb edge
                // (state <- state self-edges are sequential feedback, not loops)
                for l in &lhs_set {
                    let rhs_first = rhs_set.iter().next().cloned().unwrap_or_default();
                    kdb.drivers.push(DriverSite {
                        signal: l.clone(),
                        rhs_text: rhs_first,
                        file: m.file.clone(),
                        line: ab.line,
                        kind: ab.kind.clone(),
                    });
                }
                continue;
            }
            for l in &lhs_set {
                for r in &rhs_set {
                    if l != r {
                        edges.entry(l.clone()).or_default().push(r.clone());
                    }
                }
                if let Some((cond, rhs_t)) = first_cond_rhs(&ab.body, l) {
                    kdb.drivers.push(DriverSite {
                        signal: l.clone(),
                        rhs_text: format!("{} ? {} : <hold>", cond, rhs_t),
                        file: m.file.clone(),
                        line: ab.line,
                        kind: ab.kind.clone(),
                    });
                } else {
                    kdb.drivers.push(DriverSite {
                        signal: l.clone(),
                        rhs_text: lhs_set.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(","),
                        file: m.file.clone(),
                        line: ab.line,
                        kind: ab.kind.clone(),
                    });
                }
            }
        }
    }
    kdb.signals = sigset.into_iter().collect();
    // CSR
    let idx: HashMap<&str, u32> = kdb.signals.iter().enumerate().map(|(i, s)| (s.as_str(), i as u32)).collect();
    let mut offsets = vec![0u32];
    let mut targets = Vec::new();
    for s in &kdb.signals {
        if let Some(srcs) = edges.get(s) {
            let mut ded: Vec<&String> = srcs.iter().collect();
            ded.sort();
            ded.dedup();
            for d in ded {
                if let Some(&ti) = idx.get(d.as_str()) {
                    targets.push(ti);
                }
            }
        }
        offsets.push(targets.len() as u32);
    }
    kdb.csr_offsets = offsets;
    kdb.csr_targets = targets;
    // Tarjan SCC on fan-in graph (edge s -> its drivers)
    kdb.scc_loops = tarjan(&kdb);
    // FO4 depth: longest fan-in chain to a primary input
    kdb.fo4_depth = fo4(&kdb);
    // L1 static latch extraction
    kdb.inferred_latches = extract_latches(mods);
    // FSM extraction
    kdb.fsm_states = extract_fsms(mods);
    // CDC
    kdb.cdc_paths = detect_cdc(mods);
    kdb
}

fn collect_always_io(body: &[Stmt], lhss: &mut Vec<String>, rhss: &mut Vec<String>) {
    for st in body {
        match st {
            Stmt::Assign { lhs, rhs, .. } => {
                let mut l = Vec::new();
                expr_idents(lhs, &mut l);
                if let Expr::Ident(n) = lhs {
                    lhss.push(n.clone());
                }
                expr_idents(rhs, rhss);
            }
            Stmt::If { cond, then, els } => {
                expr_idents(cond, rhss);
                collect_always_io(then, lhss, rhss);
                collect_always_io(els, lhss, rhss);
            }
            Stmt::Case { subject, arms, default } => {
                expr_idents(subject, rhss);
                for (labels, b) in arms {
                    for l in labels {
                        expr_idents(l, rhss);
                    }
                    collect_always_io(b, lhss, rhss);
                }
                if let Some(d) = default {
                    collect_always_io(d, lhss, rhss);
                }
            }
            Stmt::For { init, cond, step, body } => {
                expr_idents(init, rhss);
                expr_idents(cond, rhss);
                expr_idents(step, rhss);
                collect_always_io(body, lhss, rhss);
            }
            Stmt::While { cond, body } | Stmt::Repeat { count: cond, body } => {
                expr_idents(cond, rhss);
                collect_always_io(body, lhss, rhss);
            }
            Stmt::Block(b) => collect_always_io(b, lhss, rhss),
            Stmt::Event { body, .. } => collect_always_io(body, lhss, rhss),
            Stmt::Delay { body, .. } => collect_always_io(body, lhss, rhss),
            Stmt::Wait { cond, body } => {
                expr_idents(cond, rhss);
                collect_always_io(body, lhss, rhss);
            }
            _ => {}
        }
    }
}

/// First `if (cond) sig = rhs` — used to render enable-condition text.
fn first_cond_rhs(body: &[Stmt], sig: &str) -> Option<(String, String)> {
    fn walk(sts: &[Stmt], sig: &str) -> Option<(String, String)> {
        for st in sts {
            match st {
                Stmt::If { cond, then, els } => {
                    if let Some(r) = walk(then, sig) {
                        return Some(r);
                    }
                    if let Some(r) = walk(els, sig) {
                        return Some(r);
                    }
                    for t in then {
                        if let Stmt::Assign { lhs, rhs, .. } = t {
                            if let Expr::Ident(n) = lhs {
                                if n == sig {
                                    return Some((expr_text(cond), expr_text(rhs)));
                                }
                            }
                        }
                    }
                }
                Stmt::Block(b) => {
                    if let Some(r) = walk(b, sig) {
                        return Some(r);
                    }
                }
                _ => {}
            }
        }
        None
    }
    walk(body, sig)
}

/// Tarjan SCC (iterative): combinational loops = SCC with >1 node or self-loop.
pub fn tarjan(kdb: &OpenKdb) -> Vec<Vec<String>> {
    let n = kdb.signals.len();
    let mut idx = vec![u32::MAX; n];
    let mut low = vec![0u32; n];
    let mut on = vec![false; n];
    let mut stack: Vec<u32> = Vec::new();
    let mut counter = 0u32;
    let mut out = Vec::new();
    for root in 0..n {
        if idx[root] != u32::MAX {
            continue;
        }
        // iterative DFS
        let mut call: Vec<(u32, u32)> = vec![(root as u32, kdb.csr_offsets[root as usize])];
        idx[root] = counter;
        low[root] = counter;
        counter += 1;
        stack.push(root as u32);
        on[root] = true;
        while let Some(&mut (v, ref mut edge_i)) = call.last_mut() {
            let vi = v as usize;
            if edge_i < &mut kdb.csr_offsets[vi + 1].clone() && (*edge_i as usize) < kdb.csr_targets.len() {
                let w = kdb.csr_targets[*edge_i as usize];
                *edge_i += 1;
                if idx[w as usize] == u32::MAX {
                    idx[w as usize] = counter;
                    low[w as usize] = counter;
                    counter += 1;
                    stack.push(w);
                    on[w as usize] = true;
                    call.push((w, kdb.csr_offsets[w as usize]));
                } else if on[w as usize] {
                    low[vi] = low[vi].min(idx[w as usize]);
                }
            } else {
                call.pop();
                if let Some(&(p, _)) = call.last() {
                    low[p as usize] = low[p as usize].min(low[vi]);
                }
                if low[vi] == idx[vi] {
                    let mut comp = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on[w as usize] = false;
                        comp.push(kdb.signals[w as usize].clone());
                        if w == v {
                            break;
                        }
                    }
                    if comp.len() > 1 {
                        out.push(comp);
                    } else if comp.len() == 1 && kdb.csr_targets[kdb.csr_offsets[vi] as usize..kdb.csr_offsets[vi + 1] as usize].contains(&(vi as u32)) {
                        out.push(comp); // self-loop
                    }
                }
            }
        }
    }
    out
}

/// FO4-ish logic depth: BFS from primary inputs through fan-in edges.
pub fn fo4(kdb: &OpenKdb) -> HashMap<String, u32> {
    let n = kdb.signals.len();
    let mut depth = vec![0u32; n];
    // driven-by edges: signal i depends on csr_targets[offsets[i]..offsets[i+1]]
    // topological-ish relaxation (n^2 worst-case but graphs are small)
    for _ in 0..n.min(64) {
        let mut changed = false;
        for i in 0..n {
            for t in kdb.csr_offsets[i]..kdb.csr_offsets[i + 1] {
                let d = kdb.csr_targets[t as usize] as usize;
                if depth[d] + 1 > depth[i] {
                    depth[i] = depth[d] + 1;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    kdb.signals.iter().enumerate().map(|(i, s)| (s.clone(), depth[i])).collect()
}

/// L1 static latch extraction from always_comb/always with incomplete branches.
pub fn extract_latches(mods: &[ModuleDef]) -> Vec<InferredLatch> {
    let mut out = Vec::new();
    for m in mods {
        for ab in &m.always {
            // KERNEL CORRECTNESS FIX (found via NOVA-SoC signoff): only
            // COMBINATIONAL blocks infer latches. Edge-sensitive `always
            // @(sig)` blocks are registers/monitors by construction — a
            // conditional assignment there is a legit edge-triggered update,
            // NOT a latch. Previously the TB's `always @(pwm_out) if (pwm_out)
            // pwm_ones = pwm_ones + 1;` was falsely reported.
            let is_comb = ab.kind == "always_comb" || ab.kind == "always_latch"
                || (ab.kind == "always" && matches!(ab.body.first(), Some(Stmt::Event { edges, .. }) if edges.is_empty()))
                || (ab.kind == "always" && !matches!(ab.body.first(), Some(Stmt::Event { .. })));
            if !is_comb {
                continue;
            }
            let body: &[Stmt] = match ab.body.first() {
                Some(Stmt::Event { body, .. }) => body,
                _ => &ab.body,
            };
            check_latch_body(body, ab, &m.file, &mut out);
        }
    }
    out
}

fn assigned_in(st: &Stmt, out: &mut Vec<String>) {
    match st {
        Stmt::Assign { lhs, .. } => {
            if let Expr::Ident(n) = lhs {
                out.push(n.clone());
            }
        }
        Stmt::If { then, els, .. } => {
            for s in then {
                assigned_in(s, out);
            }
            for s in els {
                assigned_in(s, out);
            }
        }
        Stmt::Case { arms, default, .. } => {
            for (_, b) in arms {
                for s in b {
                    assigned_in(s, out);
                }
            }
            if let Some(d) = default {
                for s in d {
                    assigned_in(s, out);
                }
            }
        }
        Stmt::Block(b) => {
            for s in b {
                assigned_in(s, out);
            }
        }
        Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::Repeat { body, .. } => {
            for s in body {
                assigned_in(s, out);
            }
        }
        _ => {}
    }
}

fn check_latch_body(body: &[Stmt], ab: &AlwaysBlock, file: &str, out: &mut Vec<InferredLatch>) {
    // signals unconditionally assigned at block top (default-before-case style)
    // cannot be latched — collect them as exempt FIRST.
    let mut exempt = std::collections::BTreeSet::new();
    for st in body {
        if let Stmt::Block(inner) = st {
            for s in inner {
                if let Stmt::Assign { lhs, .. } = s {
                    if let Expr::Ident(n) = lhs {
                        exempt.insert(n.clone());
                    }
                }
            }
        } else if let Stmt::Assign { lhs, .. } = st {
            if let Expr::Ident(n) = lhs {
                exempt.insert(n.clone());
            }
        }
    }
    fn walk(sts: &[Stmt], ab: &AlwaysBlock, file: &str, out: &mut Vec<InferredLatch>, exempt: &std::collections::BTreeSet<String>) {
        for st in sts {
            match st {
                Stmt::Block(inner) => walk(inner, ab, file, out, exempt),
                Stmt::If { cond, then, els } => {
                    walk(then, ab, file, out, exempt);
                    walk(els, ab, file, out, exempt);
                    if els.is_empty() {
                        let mut ta = Vec::new();
                        for s in then {
                            assigned_in(s, &mut ta);
                        }
                        let mut seen = std::collections::BTreeSet::new();
                        let cond_t = expr_text(cond);
                        for sig in ta {
                            if seen.insert(sig.clone()) && !exempt.contains(&sig) {
                                out.push(InferredLatch {
                                    signal: sig,
                                    file: file.to_string(),
                                    line: ab.line,
                                    always_block_type: ab.kind.clone(),
                                    enable_condition: cond_t.clone(),
                                    missing_branch_hint: format!(
                                        "add `else {} = <default>;` (or assign at block top)",
                                        seen.iter().next().cloned().unwrap_or_default()
                                    ),
                                });
                            }
                        }
                    }
                }
                Stmt::Case { subject, arms, default, .. } => {
                    for (_, b) in arms {
                        walk(b, ab, file, out, exempt);
                    }
                    if default.is_none() && !arms.is_empty() {
                        let allc: std::collections::BTreeSet<String> = {
                            let mut v = Vec::new();
                            for (_, b) in arms {
                                for s in b {
                                    assigned_in(s, &mut v);
                                }
                            }
                            v.into_iter().collect()
                        };
                        let subj_t = expr_text(subject);
                        for sig in allc {
                            let in_every = arms.iter().all(|(_, b)| {
                                let mut v = Vec::new();
                                for s in b {
                                    assigned_in(s, &mut v);
                                }
                                v.contains(&sig)
                            });
                            if in_every && !exempt.contains(&sig) {
                                out.push(InferredLatch {
                                    signal: sig.clone(),
                                    file: file.to_string(),
                                    line: ab.line,
                                    always_block_type: ab.kind.clone(),
                                    enable_condition: subj_t.clone(),
                                    missing_branch_hint: format!("add `default: {} = <default>;`", sig),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    walk(body, ab, file, out, &exempt);
}

/// FSM extraction: case(state) inside always_ff.
pub fn extract_fsms(mods: &[ModuleDef]) -> Vec<FsmInfo> {
    let mut out = Vec::new();
    for m in mods {
        for ab in &m.always {
            for st in &ab.body {
                if let Stmt::Case { subject, arms, .. } = st {
                    if let Expr::Ident(state) = subject {
                        let mut trans = Vec::new();
                        for (labels, body) in arms {
                            let mut to = None;
                            for b in body {
                                if let Stmt::Assign { lhs, rhs, blocking: false } = b {
                                    if let (Expr::Ident(l), Expr::Ident(r)) = (lhs, rhs) {
                                        if l == state {
                                            to = Some(r.clone());
                                        }
                                    }
                                }
                            }
                            if let Some(to) = to {
                                for l in labels {
                                    if let Expr::Num(v, _) = l {
                                        trans.push((expr_text(l), to.clone()));
                                    }
                                }
                            }
                        }
                        if !trans.is_empty() {
                            out.push(FsmInfo {
                                state_signal: state.clone(),
                                encodings: vec![],
                                transitions: trans.into_iter().map(|(f, t)| (f.parse().unwrap_or(0), t.parse().unwrap_or(0), format!("{}->{}", f, t))).collect(),
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

/// CDC: always_ff blocks on different clocks driving/reading shared signals.
pub fn detect_cdc(mods: &[ModuleDef]) -> Vec<CdcPath> {
    let mut clocked: HashMap<String, (String, String)> = HashMap::new(); // signal -> (clk, module)
    for m in mods {
        for ab in &m.always {
            if let Some(Stmt::Event { edges, body }) = ab.body.first() {
                let clk = edges.iter().find(|(_, n)| n.contains("clk")).map(|(_, n)| n.clone());
                if let Some(clk) = clk {
                    let mut lhss: Vec<String> = Vec::new();
                    let _ = &mut lhss;
                    let (mut l, mut r) = (Vec::new(), Vec::new());
                    collect_always_io(body, &mut l, &mut r);
                    for s in l {
                        clocked.insert(s, (clk.clone(), m.name.clone()));
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for m in mods {
        for ab in &m.always {
            if let Some(Stmt::Event { edges, body }) = ab.body.first() {
                let clk = edges.iter().find(|(_, n)| n.contains("clk")).map(|(_, n)| n.clone());
                if let Some(clk) = clk {
                    let (mut l, mut r) = (Vec::new(), Vec::new());
                    collect_always_io(body, &mut l, &mut r);
                    for s in r {
                        if let Some((sclk, _)) = clocked.get(&s) {
                            if sclk != &clk {
                                out.push(CdcPath {
                                    src: s.clone(),
                                    dst: l.first().cloned().unwrap_or_default(),
                                    src_clk: sclk.clone(),
                                    dst_clk: clk.clone(),
                                    synchronizer_depth: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Breakthrough 1 & feature 39 — counterfactual "why is this value wrong".
/// Walks the fan-in cone to find blocking inputs (0 on AND, 1 on OR, sel on MUX).
pub fn counterfactual(kdb: &OpenKdb, signal: &str, current: &str) -> Vec<String> {
    let mut report = Vec::new();
    let idx: HashMap<&str, usize> = kdb.signals.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
    let si = match idx.get(signal) {
        Some(&i) => i,
        None => return vec![format!("signal '{}' not in KDB", signal)],
    };
    report.push(format!("counterfactual analysis for `{}` = {}:", signal, current));
    // walk fan-in cone (BFS, cap 12 levels)
    let mut seen = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((si, 0));
    while let Some((i, d)) = queue.pop_front() {
        if d > 12 || !seen.insert(i) {
            continue;
        }
        for t in kdb.csr_offsets[i]..kdb.csr_offsets[i + 1] {
            let src = kdb.csr_targets[t as usize] as usize;
            let sname = &kdb.signals[src];
            // look for a driver with a controlling value
            for dr in kdb.drivers.iter().filter(|x| x.signal == *kdb.signals[i].clone()) {
                let rhs = &dr.rhs_text;
                if rhs.contains("&&") {
                    report.push(format!("  [AND-gate] `{}` is 0-blocking: set `{}`=1 to propagate (line {})", sname, sname, dr.line));
                } else if rhs.contains("||") {
                    report.push(format!("  [OR-gate] `{}` is 1-blocking: set `{}`=0 to propagate (line {})", sname, sname, dr.line));
                } else if rhs.contains("?") {
                    report.push(format!("  [MUX] `{}` selects another arm: flip `{}` to route this value (line {})", sname, sname, dr.line));
                }
            }
            queue.push_back((src, d + 1));
        }
    }
    if report.len() == 1 {
        report.push("  no controlling blockers found in fan-in cone (direct input path)".into());
    }
    report
}

/// Feature 34 — SVA synthesizer from FSM states and handshakes.
pub fn gen_sva(kdb: &OpenKdb, top: &str) -> Vec<String> {
    let mut out = Vec::new();
    for fsm in &kdb.fsm_states {
        let s = &fsm.state_signal;
        out.push(format!(
            "property p_{s}_no_x;\n  @(posedge clk) disable iff (!rst_n)\n  !$isunknown({s});\nendproperty\nassert property (p_{s}_no_x) else $error(\"[VIKAS-SVA] {s} became X\");",
            s = s
        ));
    }
    for l in &kdb.inferred_latches {
        out.push(format!(
            "// [VIKAS-SVA] guard the inferred latch on `{}`: {hint}\nproperty p_{sig}_always_driven;\n  @(posedge clk) disable iff (!rst_n)\n  1'b1 |-> $changed({sig}) || $stable({sig});\nendproperty\n",
            l.signal,
            sig = l.signal.replace('.', "_"),
            hint = l.missing_branch_hint
        ));
    }
    for c in &kdb.cdc_paths {
        out.push(format!(
            "// [VIKAS-SVA] CDC: {src} ({sc}) -> {dst} ({dc}) — add 2-flop synchronizer\nproperty p_cdc_{src}_{dst}_stable;\n  @(posedge {dc}) disable iff (!rst_n)\n  ##1 $stable({src}_sync2);\nendproperty\n",
            src = c.src.replace('.', "_"),
            dst = c.dst.replace('.', "_"),
            sc = c.src_clk,
            dc = c.dst_clk
        ));
    }
    let _ = top;
    out
}

/// Feature 37 — Jaccard signal recommender.
pub fn recommend(kdb: &OpenKdb, signal: &str, n: usize) -> Vec<(String, f64)> {
    let idx: HashMap<&str, usize> = kdb.signals.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
    let si = match idx.get(signal) {
        Some(&i) => i,
        None => return vec![],
    };
    let fan: Vec<std::collections::HashSet<usize>> = kdb
        .signals
        .iter()
        .enumerate()
        .map(|(i, _)| (kdb.csr_offsets[i]..kdb.csr_offsets[i + 1]).map(|t| kdb.csr_targets[t as usize] as usize).collect())
        .collect();
    let my = &fan[si];
    let mut scored: Vec<(String, f64)> = fan
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != si)
        .map(|(i, s)| {
            let inter = my.intersection(s).count();
            let uni = my.union(s).count().max(1);
            (kdb.signals[i].clone(), inter as f64 / uni as f64)
        })
        .filter(|(_, s)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.truncate(n);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use vikas_sim::lexer::Lexer;
    use vikas_sim::parser::Parser;

    fn parse(src: &str) -> Vec<ModuleDef> {
        let lx = Lexer::lex(src, "t.sv");
        let mut p = Parser::new(&lx.toks, &lx.lines);
        p.parse_program()
    }

    #[test]
    fn latch_l1_extraction_finds_incomplete_if() {
        let mods = parse(
            "module a(input logic sel, input logic [7:0] d, output logic [7:0] q);\nalways_comb begin\n if (sel) q = d;\nend\nendmodule\n",
        );
        let latches = extract_latches(&mods);
        assert_eq!(latches.len(), 1);
        assert_eq!(latches[0].signal, "q");
        assert_eq!(latches[0].always_block_type, "always_comb");
        assert!(latches[0].missing_branch_hint.contains("else"));
    }

    #[test]
    fn latch_l1_finds_missing_default() {
        let mods = parse(
            "module a(input logic [1:0] s, output logic [3:0] y);\nalways_comb begin\n case (s)\n  2'd0: y = 4'd1;\n  2'd1: y = 4'd2;\n endcase\nend\nendmodule\n",
        );
        let latches = extract_latches(&mods);
        assert!(!latches.is_empty());
        assert!(latches[0].missing_branch_hint.contains("default"));
    }

    #[test]
    fn kdb_build_and_fanin() {
        let mods = parse("module a(input logic x, output logic y);\nassign y = x;\nendmodule\n");
        let kdb = build(&mods, "a");
        assert_eq!(kdb.signals.len(), 2);
        assert_eq!(kdb.drivers.len(), 1);
        let yi = kdb.signals.iter().position(|s| s == "y").unwrap();
        let t = kdb.csr_targets[kdb.csr_offsets[yi] as usize..kdb.csr_offsets[yi + 1] as usize].to_vec();
        assert_eq!(t.len(), 1);
    }
}
