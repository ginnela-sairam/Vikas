//! VIKAS v1 — XEZIM execution VM: event-driven, IEEE-1364 delta-cycle and
//! NBA (nonblocking) regions, combinational iteration with glitch tracking,
//! edge-sensitive wake, taint dye, physics telemetry, latch-hold shadow signals.

use super::elab::{BoundExpr, Design, PortBinding};
use super::parser::{AlwaysBlock, Expr, ModuleDef, Stmt};
use vikas_wave::{VVal, WaveDump};

#[derive(Clone)]
pub struct SimResult {
    pub dump: WaveDump,
    pub top: String,
    pub cycles: u64,
    pub wall_ms: u128,
    pub glitches: u64,
    pub energy_fj_total: f64,
    pub latch_finds: Vec<super::latchsim::LatchFind>,
    /// every $display/$write line the run printed (mutation-test oracle)
    pub printed: Vec<String>,
}

struct Frame {
    /// owning module index — kept to document frame provenance (the prefix
    /// already disambiguates scopes today; index reserved for multi-scopes)
    #[allow(dead_code)]
    module: usize,
    prefix: String,
}

pub struct Vm {
    design: Design,
    mods: Vec<ModuleDef>,
    vals: std::collections::HashMap<String, VVal>,
    prev: std::collections::HashMap<String, VVal>,
    /// pending non-blocking-assign list — the NBA region is applied inline during
    /// the first delta iteration of each time step; the field is kept as the
    /// kernel's documented NBA hand-off point for upcoming delta-cycle split.
    #[allow(dead_code)]
    nba: Vec<(String, VVal)>,
    /// time wheel: (time, (module idx, stmt ptr-path)) via closure re-entrancy is
    /// not possible in Rust easily — we instead run per-module PC-less walkers.
    #[allow(dead_code)]
    wake_at: std::collections::BTreeMap<u64, Vec<usize>>, // time -> initial-block ids
    finished: bool,
    glitch_counts: std::collections::HashMap<String, u32>,
    latch_active: std::collections::HashMap<String, Vec<String>>, // "module:line" -> latched signals
    latch_hold_now: std::collections::HashMap<String, bool>,
    latch_finds: Vec<super::latchsim::LatchFind>,
    pub dump: WaveDump,
    /// per-signal "already dumped at t=0" set — t=0 state is emitted wholesale
    /// by dump_state(); the set is retained for planned incremental re-dumps.
    #[allow(dead_code)]
    dumped: std::collections::HashSet<String>,
    taint: std::collections::HashMap<String, u64>,
    phys_energy: f64,
    cycles: u64,
    ceff: std::collections::HashMap<String, f64>,
    clock_names: Vec<String>,
    glitch_total: u64,
    cur_t: u64,
    /// every continuous assign flattened with its instance prefix: (prefix, lhs, rhs)
    all_assigns: Vec<(String, Expr, Expr)>,
    /// quiet mode: suppress $display/$write to stdout (mutation testing runs
    /// hundreds of sims — output is captured in SimResult.printed instead)
    pub quiet: bool,
    /// runaway-loop guard shared by For exec (see KERNEL FIX #23)
    loop_guard: u64,
    /// every $display/$write line the design printed (mutation-test oracle)
    pub all_printed: Vec<String>,
}

const DELTA_CAP: u32 = 64;

impl Vm {
    pub fn new(mods: Vec<ModuleDef>, top: &str, global_consts: &std::collections::HashMap<String, u64>) -> Vm {
        let design = super::elab::elaborate(&mods, top, global_consts).expect("elaboration");
        let mut vals = std::collections::HashMap::new();
        let mut prev = std::collections::HashMap::new();
        for (n, s) in &design.sigs {
            let v = VVal::x(s.width);
            vals.insert(n.clone(), v.clone());
            prev.insert(n.clone(), v);
        }
        let mut dump = WaveDump::new();
        for n in &design.decl_order {
            let w = design.sigs[n].width;
            dump.add_signal(n, w);
        }
        Vm {
            design,
            mods,
            vals,
            prev,
            nba: Vec::new(),
            wake_at: std::collections::BTreeMap::new(),
            finished: false,
            glitch_counts: std::collections::HashMap::new(),
            latch_active: std::collections::HashMap::new(),
            latch_hold_now: std::collections::HashMap::new(),
            latch_finds: Vec::new(),
            dump,
            dumped: std::collections::HashSet::new(),
            taint: std::collections::HashMap::new(),
            phys_energy: 0.0,
            cycles: 0,
            ceff: std::collections::HashMap::new(),
            clock_names: Vec::new(),
            glitch_total: 0,
            cur_t: 0,
            all_assigns: Vec::new(),
            quiet: false,
            loop_guard: 0,
            all_printed: Vec::new(),
        }
    }

    pub fn design(&self) -> &Design {
        &self.design
    }

    pub fn set_ceff(&mut self, ceff: std::collections::HashMap<String, f64>) {
        self.ceff = ceff;
    }

    pub fn set_clock_names(&mut self, c: Vec<String>) {
        self.clock_names = c;
    }

    pub fn latch_finds_mut(&mut self) -> &mut Vec<super::latchsim::LatchFind> {
        &mut self.latch_finds
    }

    fn val(&self, name: &str) -> VVal {
        self.vals.get(name).cloned().unwrap_or_else(|| VVal::x(1))
    }

    fn qname(frame_prefix: &str, n: &str) -> String {
        if n.contains('.') {
            n.to_string()
        } else {
            format!("{}.{}", frame_prefix, n)
        }
    }

    // ---------------- expression evaluation ----------------

    fn eval(&self, e: &Expr, f: &Frame) -> VVal {
        match e {
            Expr::Num(v, w) => VVal::from_u64(*v, *w),
            Expr::Str(s) => VVal::from_bits_str(&s.bytes().map(|b| if b == b'1' { '1' } else { '0' }).collect::<String>()),
            Expr::Ident(n) => {
                let q = Self::qname(&f.prefix, n);
                if self.vals.contains_key(&q) {
                    self.val(&q)
                } else if let Some(b) = n.strip_prefix("1'b") {
                    VVal::from_bits_str(b)
                } else if let Some(&v) = self.design.param_values.get(&q) {
                    VVal::from_u64(v, 32)
                } else {
                    self.val(&q)
                }
            }
            Expr::Bin(op, a, b) => {
                let x = self.eval(a, f);
                let y = self.eval(b, f);
                self.binop(op, &x, &y)
            }
            Expr::Un(op, a) => {
                let x = self.eval(a, f);
                match op.as_str() {
                    "!" => {
                        let t = x.to_u64() == 0;
                        VVal::from_u64(t as u64, 1)
                    }
                    "~" => {
                        let mut v = x.clone();
                        for b in v.bits.iter_mut() {
                            *b = !*b;
                        }
                        v
                    }
                    "-" => VVal::from_u64(x.to_u64().wrapping_neg(), x.width().max(1)),
                    "&" => VVal::from_u64(x.to_u64(), 1),
                    "|" => VVal::from_u64(x.to_u64(), 1),
                    "^" => VVal::from_u64(x.to_u64() & 1, 1),
                    _ => x,
                }
            }
            Expr::Index(base, idx) => {
                // array cell load first: mem[i] where mem is an unpacked array
                if let Expr::Ident(n) = &**base {
                    let arr_q = Self::qname(&f.prefix, n);
                    if self.design.arrays.contains_key(&arr_q) {
                        let i = self.eval(idx, f).to_i64();
                        let cell = format!("{}__{}", arr_q, i);
                        if self.vals.contains_key(&cell) {
                            return self.val(&cell);
                        }
                        let (_, lo, hi) = self.design.arrays[&arr_q];
                        return VVal::x((hi - lo + 1).max(1) as usize);
                    }
                }
                // otherwise: bit-select
                let b = self.eval(base, f);
                let i = self.eval(idx, f).to_u64() as usize;
                let bit = b.bit(i).unwrap_or(vikas_wave::Bit::X);
                VVal { bits: vec![bit] }
            }
            Expr::Slice(base, msb, lsb) => {
                let b = self.eval(base, f);
                let hi = self.eval(msb, f).to_u64() as usize;
                let lo = self.eval(lsb, f).to_u64() as usize;
                let w = b.width();
                if hi >= w || lo > hi {
                    return VVal::x(hi.saturating_sub(lo) + 1);
                }
                // bits[] is MSB-first: Verilog a[hi:lo] maps to array [w-1-hi ..= w-1-lo]
                VVal { bits: b.bits[w - 1 - hi..=w - 1 - lo].to_vec() }
            }
            Expr::Call(fname, args) => {
                self.syscall(fname, args, f).unwrap_or_else(|| VVal::x(32))
            }
            Expr::Concat(items) => {
                // KERNEL FIX (#18, found via NOVA-SoC shift-register probe):
                // Verilog {A, B} means A's bits (MSB-first) followed by B's bits.
                // The old rev-accumulate-then-reverse assembled items in REVERSE
                // order ({A,B} evaluated as {B,A}) — silently corrupting every
                // heterogeneous concat (UART/SPI shift regs, bus packing, etc).
                let mut bits = Vec::new();
                for it in items {
                    let v = self.eval(it, f);
                    bits.extend(v.bits.iter().cloned());
                }
                VVal { bits }
            }
            Expr::Replicate(cnt, v) => {
                let n = self.eval(cnt, f).to_u64() as usize;
                let v = self.eval(v, f);
                // {n{v}} = n copies of v, MSB-first, in order
                let mut bits = Vec::new();
                for _ in 0..n {
                    bits.extend(v.bits.iter().cloned());
                }
                VVal { bits }
            }
            Expr::Ternary(c, t, e) => {
                if self.eval(c, f).to_u64() != 0 {
                    self.eval(t, f)
                } else {
                    self.eval(e, f)
                }
            }
        }
    }

    fn binop(&self, op: &str, x: &VVal, y: &VVal) -> VVal {
        use vikas_wave::Bit;
        let w = x.width().max(y.width()).max(1);
        match op {
            "&&" => VVal::from_u64(((x.to_u64() != 0) && (y.to_u64() != 0)) as u64, 1),
            "||" => VVal::from_u64(((x.to_u64() != 0) || (y.to_u64() != 0)) as u64, 1),
            // value-extended equality (4'b0 == 32'd0 is TRUE in Verilog); X-poisonous
            "==" => {
                if x.is_x() || y.is_x() {
                    VVal::x(1)
                } else {
                    VVal::from_u64((x.to_u64() == y.to_u64()) as u64, 1)
                }
            }
            "!=" => {
                if x.is_x() || y.is_x() {
                    VVal::x(1)
                } else {
                    VVal::from_u64((x.to_u64() != y.to_u64()) as u64, 1)
                }
            }
            // unsigned comparisons (Verilog `logic` vector semantics)
            "<" => VVal::from_u64((x.to_u64() < y.to_u64()) as u64, 1),
            "<=" => VVal::from_u64((x.to_u64() <= y.to_u64()) as u64, 1),
            ">" => VVal::from_u64((x.to_u64() > y.to_u64()) as u64, 1),
            ">=" => VVal::from_u64((x.to_u64() >= y.to_u64()) as u64, 1),
            "+" => VVal::from_u64(x.to_u64().wrapping_add(y.to_u64()), w),
            "-" => VVal::from_u64(x.to_u64().wrapping_sub(y.to_u64()), w),
            "*" => VVal::from_u64(x.to_u64().wrapping_mul(y.to_u64()), w),
            "/" => VVal::from_u64(if y.to_u64() != 0 { x.to_u64() / y.to_u64() } else { 0 }, w),
            "%" => VVal::from_u64(if y.to_u64() != 0 { x.to_u64() % y.to_u64() } else { 0 }, w),
            "**" => {
                let (b, e) = (x.to_u64(), y.to_u64());
                VVal::from_u64(if e > 63 { 0 } else { b.pow(e as u32) }, w)
            }
            "&" => {
                let mut bits = Vec::with_capacity(w);
                for i in 0..w {
                    let a = x.bit(i).unwrap_or(Bit::X);
                    let b = y.bit(i).unwrap_or(Bit::X);
                    bits.push(match (a, b) {
                        (Bit::One, Bit::One) => Bit::One,
                        (Bit::Zero, _) | (_, Bit::Zero) => Bit::Zero,
                        _ => Bit::X,
                    });
                }
                bits.reverse();
                VVal { bits }
            }
            "|" => {
                let mut bits = Vec::with_capacity(w);
                for i in 0..w {
                    let a = x.bit(i).unwrap_or(Bit::X);
                    let b = y.bit(i).unwrap_or(Bit::X);
                    bits.push(match (a, b) {
                        (Bit::One, _) | (_, Bit::One) => Bit::One,
                        (Bit::Zero, Bit::Zero) => Bit::Zero,
                        _ => Bit::X,
                    });
                }
                bits.reverse();
                VVal { bits }
            }
            "^" => {
                let mut bits = Vec::with_capacity(w);
                for i in 0..w {
                    let a = x.bit(i).unwrap_or(Bit::X);
                    let b = y.bit(i).unwrap_or(Bit::X);
                    bits.push(match (a, b) {
                        (Bit::One, Bit::Zero) | (Bit::Zero, Bit::One) => Bit::One,
                        (Bit::One, Bit::One) | (Bit::Zero, Bit::Zero) => Bit::Zero,
                        _ => Bit::X,
                    });
                }
                bits.reverse();
                VVal { bits }
            }
            "<<" => VVal::from_u64(x.to_u64().wrapping_shl(y.to_u64() as u32), w),
            ">>" => VVal::from_u64(x.to_u64().wrapping_shr(y.to_u64() as u32), w),
            _ => VVal::x(w),
        }
    }

    fn syscall(&self, name: &str, args: &[Expr], f: &Frame) -> Option<VVal> {
        let _ = f;
        match name {
            "$time" => Some(VVal::from_u64(self.cur_t, 64)),
            "$random" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
                let x = s.subsec_nanos() as u64 ^ (self.cur_t.wrapping_mul(2654435761));
                Some(VVal::from_u64(x, 32))
            }
            "$clog2" => {
                let v = args.first().map(|a| self.eval(a, f).to_u64()).unwrap_or(1);
                Some(VVal::from_u64((v as f64).log2().ceil().max(0.0) as u64, 32))
            }
            _ => None,
        }
    }

    // ---------------- statement execution ----------------

    fn exec_stmt(&mut self, s: &Stmt, f: &Frame, out: &mut ExecSide) -> bool {
        // returns false => hit a delay that parked this block
        match s {
            Stmt::Assign { lhs, rhs, blocking } => {
                let v = self.eval(rhs, f);
                let (q, msel, lsel) = self.lhs_target(lhs, f);
                if *blocking {
                    self.store(&q, msel, lsel, &v, out);
                } else {
                    out.nba.push((q.clone(), msel, lsel, v.clone()));
                }
                out.assigned.push(q);
                true
            }
            Stmt::If { cond, then, els } => {
                let c = self.eval(cond, f).to_u64();
                let body = if c != 0 { then } else { els };
                self.exec_seq(body, f, out)
            }
            Stmt::Case { subject, arms, default } => {
                let sv = self.eval(subject, f);
                let svu = sv.to_u64();
                let mut hit = false;
                for (labels, body) in arms {
                    for l in labels {
                        let lv = self.eval(l, f);
                        let lu = lv.to_u64();
                        let mask = (1u64 << sv.width().min(63)) - 1;
                        if lu & mask == svu & mask {
                            hit = true;
                            if !self.exec_seq(body, f, out) {
                                return false;
                            }
                            break;
                        }
                    }
                    if hit {
                        break;
                    }
                }
                if !hit {
                    if let Some(body) = default {
                        for st in body {
                            if !self.exec_stmt(st, f, out) {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            Stmt::For { init, cond, step, body } => {
                if let Expr::Bin(op, l, r) = init {
                    if op == "=" {
                        let v = self.eval(r, f);
                        let (q, m, ls) = self.lhs_target(l, f);
                        self.store(&q, m, ls, &v, out);
                    }
                }
                loop {
                    let c = self.eval(cond, f).to_u64();
                    if c == 0 {
                        break;
                    }
                    // KERNEL FIX #23 (found via mutation testing): a mutated or
                    // broken loop condition (e.g. step mutated to `i = i + 0`)
                    // used to spin forever, growing the NBA list unboundedly and
                    // OOM-killing the process. Same 1M-iteration guard as While.
                    if self.loop_guard > 1_000_000 {
                        self.loop_guard = 0;
                        break;
                    }
                    self.loop_guard += 1;
                    if !self.exec_seq(body, f, out) {
                        // KERNEL FIX (found via NOVA-SoC uart sampler probe): a park
                        // inside a for-body used to silently DROP all remaining
                        // iterations. Resume = rest of this iteration + the step
                        // assignment + a synthetic loop head with a no-op init
                        // (re-running the real init would reset the loop var).
                        let mut cont = std::mem::take(&mut out.parked_tail);
                        if let Expr::Bin(op, l, r) = step {
                            let rhs = match op.as_str() {
                                "+=" => Expr::Bin("+".to_string(), l.clone(), r.clone()),
                                "-=" => Expr::Bin("-".to_string(), l.clone(), r.clone()),
                                _ => (**r).clone(),
                            };
                            cont.push(Stmt::Assign { lhs: (**l).clone(), rhs, blocking: true });
                        }
                        cont.push(Stmt::For { init: Expr::Num(0, 1), cond: cond.clone(), step: step.clone(), body: body.clone() });
                        out.parked_tail = cont;
                        return false;
                    }
                    if let Expr::Bin(op, l, r) = step {
                        let v = self.eval(r, f);
                        let (q, m, ls) = self.lhs_target(l, f);
                        if op == "+=" {
                            let old = self.load_slice(&q, m, ls);
                            self.store(&q, m, ls, &VVal::from_u64(old.to_u64().wrapping_add(v.to_u64()), old.width().max(v.width())), out);
                        } else if op == "-=" {
                            let old = self.load_slice(&q, m, ls);
                            self.store(&q, m, ls, &VVal::from_u64(old.to_u64().wrapping_sub(v.to_u64()), old.width().max(v.width())), out);
                        } else {
                            self.store(&q, m, ls, &v, out);
                        }
                    }
                }
                true
            }
            Stmt::While { cond, body } => {
                let mut guard = 0u64;
                loop {
                    if self.eval(cond, f).to_u64() == 0 {
                        break;
                    }
                    if !self.exec_seq(body, f, out) {
                        // resume this iteration, then re-evaluate the loop head
                        let mut cont = std::mem::take(&mut out.parked_tail);
                        cont.push(Stmt::While { cond: cond.clone(), body: body.clone() });
                        out.parked_tail = cont;
                        return false;
                    }
                    guard += 1;
                    if guard > 1_000_000 {
                        break;
                    }
                }
                true
            }
            Stmt::Repeat { count, body } => {
                let n = self.eval(count, f).to_u64().min(1_000_000);
                if n == 0 {
                    return true;
                }
                if !self.exec_seq(body, f, out) {
                    // resume this iteration, then reschedule the remaining n-1
                    let mut cont = std::mem::take(&mut out.parked_tail);
                    if n > 1 {
                        cont.push(Stmt::Repeat { count: Expr::Num(n - 1, 64), body: body.clone() });
                    }
                    out.parked_tail = cont;
                    return false;
                }
                // remaining iterations ran without parking only if delay-free
                for _ in 1..n {
                    if !self.exec_seq(body, f, out) {
                        let cont = std::mem::take(&mut out.parked_tail);
                        out.parked_tail = cont;
                        return false;
                    }
                }
                true
            }
            Stmt::Delay { ns, body } => {
                out.park_at = Some(self.cur_t + ns);
                out.parked_tail = body.clone();
                false
            }
            Stmt::Event { edges, body } => {
                // inline event inside initial: run body each time reached (edge semantics
                // handled by always-block scheduler; here treat as immediate once)
                let _ = edges;
                self.exec_seq(body, f, out)
            }
            Stmt::Wait { cond, body } => {
                if self.eval(cond, f).to_u64() != 0 {
                    self.exec_seq(body, f, out)
                } else {
                    // retry next timestep
                    out.park_at = Some(self.cur_t + 1);
                    out.parked_wait = Some((*cond).clone());
                    out.parked_tail = body.clone();
                    false
                }
            }
            Stmt::Decl { name, width, init, dims, kind } => {
                let _ = dims;
                let _ = kind;
                let q = Self::qname(&f.prefix, name);
                if !self.vals.contains_key(&q) {
                    let v = init.as_ref().map(|i| self.eval(i, f)).unwrap_or_else(|| VVal::x(*width));
                    let w = self.design.sigs.get(&q).map(|s| s.width).unwrap_or(*width);
                    let w2 = w.max(v.width()).max(1);
                    self.vals.insert(q.clone(), v.clone());
                    self.dump.add_signal(&q, w2);
                    self.prev.insert(q, v);
                } else if let Some(i) = init {
                    let v = self.eval(i, f);
                    let (q2, m, l) = (q.clone(), -1, -1);
                    self.store(&q2, m, l, &v, out);
                }
                true
            }
            Stmt::FCall(name, args) => {
                self.syscall(name, args, f);
                true
            }
            Stmt::SysTask(name, args) => {
                self.systask(name, args, f, out);
                true
            }
            Stmt::Block(body) => self.exec_seq(body, f, out),
        }
    }

    /// Execute a statement list with proper continuation tracking: when a nested
    /// statement parks (delay/wait), the continuation = parked inner body + the
    /// remaining statements of THIS list. This is what makes `#10 x=1; y=2;`
    /// resume at y=2 instead of restarting at x=1.
    fn exec_seq(&mut self, body: &[Stmt], f: &Frame, out: &mut ExecSide) -> bool {
        for (k, st) in body.iter().enumerate() {
            if !self.exec_stmt(st, f, out) {
                let mut cont: Vec<Stmt> = std::mem::take(&mut out.parked_tail);
                cont.extend(body[k + 1..].iter().cloned());
                out.parked_tail = cont;
                return false;
            }
        }
        true
    }

    fn lhs_target(&self, lhs: &Expr, f: &Frame) -> (String, i64, i64) {
        match lhs {
            Expr::Ident(n) => (Self::qname(&f.prefix, n), -1, -1),
            Expr::Index(b, i) => {
                if let Expr::Ident(n) = &**b {
                    let arr_q = Self::qname(&f.prefix, n);
                    if self.design.arrays.contains_key(&arr_q) {
                        // array element store: route to the per-cell signal
                        let idx = self.eval(i, f).to_i64();
                        let cell = format!("{}__{}", arr_q, idx);
                        return (cell, -1, -1);
                    }
                    let idx = self.eval(i, f).to_u64();
                    return (Self::qname(&f.prefix, n), idx as i64, idx as i64);
                }
                ("?".into(), -1, -1)
            }
            Expr::Slice(b, msb, lsb) => {
                if let Expr::Ident(n) = &**b {
                    let m = self.eval(msb, f).to_u64() as i64;
                    let l = self.eval(lsb, f).to_u64() as i64;
                    (Self::qname(&f.prefix, n), m, l)
                } else {
                    ("?".into(), -1, -1)
                }
            }
            _ => ("?".into(), -1, -1),
        }
    }

    fn load_slice(&self, q: &str, msb: i64, lsb: i64) -> VVal {
        let v = self.val(q);
        if msb < 0 {
            return v;
        }
        let (m, l) = (msb as usize, lsb as usize);
        let w = v.width();
        if m >= w || l > m {
            return VVal::x((m - l + 1).max(1));
        }
        VVal { bits: v.bits[w - 1 - m..=w - 1 - l].to_vec() }
    }

    fn store(&mut self, q: &str, msb: i64, lsb: i64, v: &VVal, out: &mut ExecSide) {
        if !self.vals.contains_key(q) {
            let w = v.width().max(1);
            self.vals.insert(q.to_string(), VVal::x(w));
            self.prev.insert(q.to_string(), VVal::x(w));
            self.dump.add_signal(q, w);
            self.design_decl_add(q, w);
        }
        if msb < 0 {
            let w = self.design.sigs.get(q).map(|s| s.width).unwrap_or(v.width()).max(1);
            let nv = if v.width() == w { v.clone() } else if v.width() > w { VVal { bits: v.bits[v.width() - w..].to_vec() } } else { let mut b = v.bits.clone(); let n = b.len(); b.resize(w, vikas_wave::Bit::X); b.rotate_right(w.saturating_sub(n)); VVal { bits: b } };
            if self.vals.get(q) != Some(&nv) {
                self.vals.insert(q.to_string(), nv);
                out.touched.push(q.to_string());
            }
        } else {
            let (m, l) = (msb as usize, lsb as usize);
            let mut cur = self.val(q);
            let mut nb = v.bits.clone();
            let nbits = (m - l + 1).max(1);
            while nb.len() < nbits {
                nb.insert(0, vikas_wave::Bit::X);
            }
            // cur.bits is MSB-first; index i = bit position from LSB
            for (k, bit) in nb.iter().rev().enumerate() {
                let pos = l + k;
                let n = cur.bits.len();
                if pos < n {
                    cur.bits[n - 1 - pos] = *bit;
                }
            }
            if self.vals.get(q) != Some(&cur) {
                self.vals.insert(q.to_string(), cur);
                out.touched.push(q.to_string());
            }
        }
    }

    fn design_decl_add(&mut self, _q: &str, _w: usize) {}

    fn systask(&mut self, name: &str, args: &[Expr], f: &Frame, out: &mut ExecSide) {
        match name {
            "$display" | "$write" => {
                let mut line = String::new();
                let mut ai = 0;
                while ai < args.len() {
                    if let Expr::Str(fmt) = &args[ai] {
                        let mut rest = fmt.clone();
                        while let Some(pp) = rest.find('%') {
                            line.push_str(&rest[..pp]);
                            // skip spec: % [flags/digits] <spec-char>   (handles %0d, %02h, %d)
                            let after = &rest[pp + 1..];
                            let skip = after.chars().take_while(|c| c.is_ascii_digit() || *c == '0').count() + 1;
                            let spec = after.chars().nth(skip - 1).unwrap_or('d');
                            rest = rest[pp + 1 + skip..].to_string();
                            if ai + 1 < args.len() {
                                ai += 1;
                                let v = self.eval(&args[ai], f);
                                match spec {
                                    'h' | 'H' | 'x' | 'X' => line.push_str(&v.to_hex_str()),
                                    'b' | 'B' => line.push_str(&v.to_bin_str()),
                                    'd' | 'D' => line.push_str(&v.to_u64().to_string()),
                                    'c' | 'C' => line.push(v.to_u64() as u8 as char),
                                    _ => {}
                                }
                            }
                        }
                        line.push_str(&rest);
                    } else {
                        let v = self.eval(&args[ai], f);
                        line.push_str(&format!("{}", v.to_u64()));
                    }
                    ai += 1;
                }
                out.printed.push(if name == "$display" { format!("{}\n", line) } else { line });
            }
            "$finish" | "$stop" => {
                self.finished = true;
            }
            "$fatal" | "$error" => {
                out.printed.push(format!("[VIKAS] {} at t={}ns\n", name.to_uppercase(), self.cur_t));
                self.finished = true;
            }
            "$dumpvars" | "$dumpon" => {}
            "$dumpfile" | "$dumpoff" => {}
            "$displayh" | "$displayb" => {}
            "$assert" => {
                let v = args.first().map(|a| self.eval(a, f).to_u64()).unwrap_or(0);
                if v == 0 {
                    out.printed.push(format!("[VIKAS] assertion FAILED at t={}ns\n", self.cur_t));
                }
            }
            _ => {}
        }
    }

    // ---------------- port sync ----------------

    /// Verilog width adaptation (KERNEL FIX #19): truncation drops MSBs
    /// (keeps the low bits — `1` bound to a 1-bit port must stay 1, previously
    /// became 0 by keeping the MSB of the 32-bit constant); widening
    /// zero-extends on the MSB side (was: X-padding the LSB side).
    fn fit_width(v: &VVal, w: usize) -> VVal {
        if v.width() == w {
            v.clone()
        } else if v.width() > w {
            VVal { bits: v.bits[v.bits.len() - w..].to_vec() }
        } else {
            let mut bits = vec![vikas_wave::Bit::Zero; w - v.bits.len()];
            bits.extend(v.bits.iter().cloned());
            VVal { bits }
        }
    }

    fn sync_ports_in(&mut self, out: &mut ExecSide) {
        // deterministic order: inputs parent->child, then outputs child->parent
        let binds: Vec<PortBinding> = self.design.port_bindings.clone();
        for b in &binds {
            if b.dir & 1 != 0 {
                let v = self.bound_value(&b.parent_expr);
                let old = self.vals.get(&b.child_sig).cloned();
                if old.as_ref() != Some(&v) {
                    let w = self.design.sigs.get(&b.child_sig).map(|s| s.width).unwrap_or(v.width());
                    let nv = Self::fit_width(&v, w);
                    self.vals.insert(b.child_sig.clone(), nv);
                    out.touched.push(b.child_sig.clone());
                }
            }
        }
    }

    fn sync_ports_out(&mut self, out: &mut ExecSide) {
        let binds: Vec<PortBinding> = self.design.port_bindings.clone();
        for b in &binds {
            if b.dir & 2 != 0 {
                let cv = self.val(&b.child_sig);
                match &b.parent_expr {
                    BoundExpr::Sig(p) => {
                        if self.vals.contains_key(p) {
                            let w = self.design.sigs.get(p).map(|s| s.width).unwrap_or(cv.width());
                            let nv = if cv.width() == w { cv.clone() } else { Self::fit_width(&cv, w) };
                            if self.vals.get(p) != Some(&nv) {
                                self.vals.insert(p.clone(), nv);
                                out.touched.push(p.clone());
                            }
                        }
                    }
                    BoundExpr::Slice(p, msb, lsb) => {
                        if self.vals.contains_key(p) {
                            let mut cur = self.val(p);
                            let (m, l) = (*msb as usize, *lsb as usize);
                            let wcur = cur.width();
                            let wcv = cv.width();
                            for k in 0..cv.width() {
                                let pos = l + k;
                                if pos < wcur {
                                    cur.bits[wcur - 1 - pos] = cv.bits[wcv - 1 - k];
                                }
                            }
                            let _ = m;
                            if self.vals.get(p) != Some(&cur) {
                                self.vals.insert(p.clone(), cur);
                                out.touched.push(p.clone());
                            }
                        }
                    }
                    BoundExpr::Concat(items) => {
                        let mut offset = 0usize; // from LSB
                        for it in items {
                            let seg = match it {
                                BoundExpr::Num(v, w) => VVal::from_u64(*v, *w),
                                _ => VVal::x(1),
                            };
                            let seg = match it {
                                BoundExpr::Sig(p) => {
                                    if self.vals.contains_key(p) {
                                        let w = self.design.sigs.get(p.as_str()).map(|s| s.width).unwrap_or(1);
                                        let cv2 = self.val(p);
                                        let bits = Self::fit_width(&cv2, w).bits;
                                        VVal { bits }
                                    } else {
                                        VVal::x(1)
                                    }
                                }
                                _ => seg,
                            };
                            offset += seg.width();
                        }
                        let total = offset;
                        // write child value into parent concat segments (LSB-first order reversed)
                        let mut cvl = cv.bits.clone();
                        cvl.reverse();
                        let mut pos = 0usize;
                        for it in items.iter().rev() {
                            let seg_w: usize = match it {
                                BoundExpr::Num(_, w) => *w,
                                BoundExpr::Sig(p) => self.design.sigs.get(p.as_str()).map(|s| s.width).unwrap_or(1),
                                _ => 1,
                            };
                            if let BoundExpr::Sig(p) = it {
                                if self.vals.contains_key(p) {
                                    let mut cur = self.val(p);
                                    let wcur = cur.width();
                                    for k in 0..seg_w {
                                        if pos + k < cvl.len() && k < wcur {
                                            cur.bits[wcur - 1 - k] = cvl[pos + k];
                                        }
                                    }
                                    if self.vals.get(p) != Some(&cur) {
                                        self.vals.insert(p.clone(), cur);
                                        out.touched.push(p.clone());
                                    }
                                }
                            }
                            pos += seg_w;
                        }
                        let _ = total;
                    }
                    _ => {}
                }
            }
        }
    }

    fn bound_value(&self, b: &BoundExpr) -> VVal {
        match b {
            BoundExpr::Num(v, w) => VVal::from_u64(*v, *w),
            BoundExpr::Sig(p) => self.val(p),
            BoundExpr::Expr(e, pfx) => {
                // kernel #21: expression connection — evaluate in the PARENT scope
                let f = Frame { module: 0, prefix: pfx.clone() };
                self.eval(e, &f)
            }
            BoundExpr::Slice(p, msb, lsb) => {
                let v = self.val(p);
                let (m, l) = (*msb as usize, *lsb as usize);
                let w = v.width();
                if m >= w || l > m {
                    VVal::x((m - l + 1).max(1))
                } else {
                    VVal { bits: v.bits[w - 1 - m..=w - 1 - l].to_vec() }
                }
            }
            BoundExpr::Concat(items) => {
                // KERNEL FIX (#18b): items in Verilog order, MSB-first, first item
                // leftmost (previously assembled reversed: {A,B} == {B,A}).
                let mut bits = Vec::new();
                for it in items {
                    let v = match it {
                        BoundExpr::Num(v, w) => VVal::from_u64(*v, *w),
                        BoundExpr::Sig(p) => {
                            let w = self.design.sigs.get(p.as_str()).map(|s| s.width).unwrap_or(1);
                            let raw = self.val(p);
                            let mut b = raw.bits.clone();
                            b.resize(w, vikas_wave::Bit::X);
                            VVal { bits: b }
                        }
                        BoundExpr::Slice(p, m, l) => {
                            // KERNEL FIX (#18c): a[msb:lsb] on an MSB-first bits
                            // array maps to [w-1-msb ..= w-1-lsb], NOT [lsb..=msb].
                            let raw = self.val(p);
                            let (mm, ll) = (*m as usize, *l as usize);
                            let w = raw.width();
                            if mm < w && ll <= mm {
                                VVal { bits: raw.bits[w - 1 - mm..=w - 1 - ll].to_vec() }
                            } else {
                                VVal::x(1)
                            }
                        }
                        _ => VVal::x(1),
                    };
                    bits.extend(v.bits.iter().cloned());
                }
                VVal { bits }
            }
            BoundExpr::Open => VVal::x(1),
        }
    }

    // ---------------- main loop ----------------

    pub fn run(&mut self, max_ns: u64) -> SimResult {
        let start = std::time::Instant::now();
        self.cur_t = 0;
        // find module index for top
        let top_idx = self.mods.iter().position(|m| m.name == self.design.top).map(|i| i).unwrap_or(0);
        let top_prefix = self.design.top.clone();
        // --- schedule initial blocks ---
        let mut initials: Vec<(usize, String, Vec<Stmt>, u32)> = Vec::new();
        self.collect_initials(top_idx, &top_prefix, &mut initials);
        // also always blocks flattened view: (module, prefix, AlwaysBlock clone)
        let mut always_blocks: Vec<(usize, String, AlwaysBlock)> = Vec::new();
        self.collect_always(top_idx, &top_prefix, &mut always_blocks);

        // latch analysis: mark always_comb blocks whose LHS set doesn't cover all paths
        self.analyze_latches(&always_blocks);
        // collect every module's continuous assigns (any depth) for the delta loop
        self.collect_all_assigns(top_idx, &top_prefix.clone());

        // parked walkers: (wake_time, module, prefix, body)
        let mut parked: Vec<(u64, usize, String, Vec<Stmt>)> = Vec::new();
        for (i, init) in initials.iter().enumerate() {
            let _ = i;
            parked.push((0, init.0, init.1.clone(), init.2.clone()));
        }
        // auto-detect the main clock for cycle counting (feature 45) if unset.
        // KERNEL FIX #24 (run-to-run determinism): `vals` is a HashMap whose
        // iteration order is randomized per process — picking the first
        // clock-named key in iteration order made the SAME design report
        // DIFFERENT cycle counts on different invocations (e.g. cdc_tb showed
        // "6 cycles" in one process, "2 cycles" in the next). The main clock
        // is now chosen deterministically: the alphabetically-first clock-
        // named signal, which also naturally prefers top-level clocks
        // ("t.clk" sorts before "t.dut.clk").
        if self.clock_names.is_empty() {
            let mut cands: Vec<String> = self.vals
                .keys()
                .filter(|k| k.rsplit('.').next().unwrap_or("").contains("clk"))
                .cloned()
                .collect();
            cands.sort();
            if let Some(clkq) = cands.first() {
                self.clock_names.push(clkq.clone());
            }
        }

        // dump t=0 state
        self.dump_state();

        let mut out;
        // main time loop
        let mut guard = 0u64;
        while !self.finished && self.cur_t <= max_ns && guard < 5_000_000 {
            guard += 1;
            out = ExecSide::default();
            // 1. run parked bodies due now
            let due: Vec<usize> = (0..parked.len()).filter(|i| parked[*i].0 <= self.cur_t).collect();
            for di in due.iter().rev() {
                let (t, mi, prefix, body) = parked[*di].clone();
                let _ = t;
                let f = Frame { module: mi, prefix: prefix.clone() };
                let mut sub = ExecSide::default();
                let completed = self.exec_seq(&body, &f, &mut sub);
                out.printed.extend(sub.printed.clone());
                out.touched.extend(sub.touched.clone());
                out.nba.extend(sub.nba.clone());
                if !completed {
                    // park continuation (inner delayed body + remaining stmts) at new time
                    parked.push((sub.park_at.unwrap_or(self.cur_t + 1), mi, prefix, sub.parked_tail.clone()));
                }
                parked.remove(*di);
            }
            // 1.5 sync input ports BEFORE edge detection so child clocks see
            // the parent's new value in the SAME timestep (critical for hierarchy)
            {
                let mut presync = ExecSide::default();
                self.sync_ports_in(&mut presync);
                for t in presync.touched {
                    out.touched.push(t);
                }
                // 1.6 sync output ports too: parent-scope edge tests
                // (`always @(dut.done)`, `@(irq)`) must see the child's new
                // output in the SAME timestep — previously these edges never
                // fired because out-port sync only happened later in the
                // delta loop, after edge detection had already run with the
                // stale value (found via NOVA-SoC uart sampler probe).
                let mut presync2 = ExecSide::default();
                self.sync_ports_out(&mut presync2);
                for t in presync2.touched {
                    out.touched.push(t);
                }
            }
            // 2. edge-sensitive always blocks: fire on edges
            let mut fired_once: std::collections::HashSet<String> = std::collections::HashSet::new();
            for (mi, prefix, ab) in always_blocks.iter() {
                if ab.kind == "initial" {
                    continue;
                }
                let f = Frame { module: *mi, prefix: prefix.clone() };
                if let Some(Stmt::Event { edges, body }) = ab.body.first() {
                    let mut fired = false;
                    for (edge, n) in edges {
                        let q = if n.contains('.') { n.clone() } else { format!("{}.{}", prefix, n) };
                        let cur = self.val(&q).to_u64();
                        let old = self.prev.get(&q).map(|v| v.to_u64()).unwrap_or(cur);
                        let hit = match edge {
                            super::parser::Edge::Pos => old == 0 && cur == 1,
                            super::parser::Edge::Neg => old == 1 && cur == 0,
                            super::parser::Edge::Both => old != cur,
                        };
                        if hit {
                            fired = true;
                            break;
                        }
                    }
                    if fired {
                        fired_once.insert(format!("{}:{}:{}", mi, prefix, ab.line));
                        let mut sub = ExecSide::default();
                        let completed = self.exec_seq(body, &f, &mut sub);
                        out.printed.extend(sub.printed.clone());
                        out.touched.extend(sub.touched.clone());
                        out.nba.extend(sub.nba.clone());
                        if !completed {
                            parked.push((sub.park_at.unwrap_or(self.cur_t + 1), *mi, prefix.clone(), sub.parked_tail.clone()));
                        }
                    }
                } else {
                    // no event wrapper: combinational-style, handled in delta below
                }
            }
            // 3. combinational delta loop: run comb blocks + assigns until stable.
            // IEEE order: edge-block NBAs (from step 2) apply in the FIRST delta
            // iteration so downstream combinational logic sees new register values
            // in the SAME timestep (no one-cycle lag through reg->comb paths).
            let pending_nba: Vec<(String, i64, i64, VVal)> = std::mem::take(&mut out.nba);
            let mut iter = 0u32;
            loop {
                iter += 1;
                let mut delta = ExecSide::default();
                self.sync_ports_in(&mut delta);
                if iter == 1 && !pending_nba.is_empty() {
                    for (q, msb, lsb, v) in pending_nba.clone() {
                        let msb2 = if msb < 0 { -1 } else { msb };
                        self.store(&q, msb2, lsb, &v, &mut delta);
                        delta.assigned.push(q);
                    }
                }
                // assigns (pre-collected with per-module prefixes for correct scope resolution)
                let assigns_now: Vec<(String, Expr, Expr)> = self.all_assigns.clone();
                for (pfx, lhs, rhs) in &assigns_now {
                    let f = Frame { module: 0, prefix: pfx.clone() };
                    let (q, msb, lsb) = self.lhs_target(lhs, &f);
                    let v = self.eval(rhs, &f);
                    if msb >= 0 {
                        self.store(&q, msb, lsb, &v, &mut delta);
                    } else {
                        self.store(&q, -1, -1, &v, &mut delta);
                    }
                    delta.assigned.push(q);
                }
                // combinational always blocks (always_comb, always @(*))
                for (mi, prefix, ab) in always_blocks.iter() {
                    let is_comb = ab.kind == "always_comb" || ab.kind == "always_latch"
                        || (ab.kind == "always" && matches!(ab.body.first(), Some(Stmt::Event { edges, .. }) if edges.is_empty()))
                        || (ab.kind == "always" && !matches!(ab.body.first(), Some(Stmt::Event { .. })));
                    if !is_comb {
                        continue;
                    }
                    let f = Frame { module: *mi, prefix: prefix.clone() };
                    let mut sub = ExecSide::default();
                    let body: Vec<Stmt> = match ab.body.first() {
                        Some(Stmt::Event { body, .. }) => body.clone(),
                        _ => ab.body.clone(),
                    };
                    for st in &body {
                        self.exec_stmt(st, &f, &mut sub);
                    }
                    delta.printed.extend(sub.printed);
                    delta.touched.extend(sub.touched);
                    delta.nba.extend(sub.nba);
                    delta.assigned.extend(sub.assigned.clone());
                    // latch-hold detection: which latched signals were NOT assigned
                    if let Some(latched) = self.latch_active_by_block(*mi, ab) {
                        for lname in latched {
                            let assigned = sub.assigned.iter().any(|a| a == &lname);
                            let hold = !assigned;
                            self.latch_hold_now.insert(lname.clone(), hold);
                        }
                    }
                }
                self.sync_ports_out(&mut delta);
                // apply NBA from this delta iteration
                if !delta.nba.is_empty() {
                    for (q, msb, lsb, v) in delta.nba.clone() {
                        self.store(&q, if msb < 0 { -1 } else { msb }, lsb, &v, &mut delta);
                        delta.assigned.push(q);
                    }
                    delta.nba.clear();
                }
                if delta.touched.is_empty() || iter >= DELTA_CAP {
                    if iter >= DELTA_CAP {
                        self.glitch_total += 1;
                    }
                    break;
                }
                // record glitch: >2 delta iterations with touches = potential hazard
                if iter > 2 {
                    for t in &delta.touched {
                        *self.glitch_counts.entry(t.clone()).or_insert(0) += 1;
                    }
                }
            }
            // 3.5 LATE EDGE PASS (kernel fix found via NOVA-SoC uart probe):
            // edges that only became visible AFTER the delta settle — typically
            // `always @(dut.done)` / `@(irq)` in a PARENT watching a CHILD output
            // port. Step 2 ran before the child's NBA had been applied, so the
            // parent saw the stale value; by the next timestep prev had caught
            // up and the edge was lost forever. Here we re-scan with settled
            // values. fired_once guarantees no block executes twice per timestep.
            let mut late_rounds = 0u32;
            loop {
                late_rounds += 1;
                if late_rounds > 40 {
                    break;
                }
                let mut late_nba: Vec<(String, i64, i64, VVal)> = Vec::new();
                let mut any_fired = false;
                for (mi, prefix, ab) in always_blocks.iter() {
                    if ab.kind == "initial" {
                        continue;
                    }
                    let (edges, body) = match ab.body.first() {
                        Some(Stmt::Event { edges, body }) => (edges, body),
                        _ => continue,
                    };
                    let key = format!("{}:{}:{}", mi, prefix, ab.line);
                    if fired_once.contains(&key) {
                        continue;
                    }
                    let f = Frame { module: *mi, prefix: prefix.clone() };
                    let mut hit = false;
                    for (edge, n) in edges {
                        let q = if n.contains('.') { n.clone() } else { format!("{}.{}", prefix, n) };
                        let cur = self.val(&q).to_u64();
                        let old = self.prev.get(&q).map(|v| v.to_u64()).unwrap_or(cur);
                        let fired_now = match edge {
                            super::parser::Edge::Pos => old == 0 && cur == 1,
                            super::parser::Edge::Neg => old == 1 && cur == 0,
                            super::parser::Edge::Both => old != cur,
                        };
                        if fired_now {
                            hit = true;
                            break;
                        }
                    }
                    if hit {
                        fired_once.insert(key);
                        any_fired = true;
                        let mut sub = ExecSide::default();
                        let completed = self.exec_seq(body, &f, &mut sub);
                        out.printed.extend(sub.printed.clone());
                        out.touched.extend(sub.touched.clone());
                        late_nba.extend(sub.nba.clone());
                        if !completed {
                            parked.push((sub.park_at.unwrap_or(self.cur_t + 1), *mi, prefix.clone(), sub.parked_tail.clone()));
                        }
                    }
                }
                if !any_fired {
                    break;
                }
                // settle the late NBAs with a bounded comb delta (mirrors step 3)
                let mut iter2 = 0u32;
                loop {
                    iter2 += 1;
                    let mut d2 = ExecSide::default();
                    self.sync_ports_in(&mut d2);
                    if iter2 == 1 && !late_nba.is_empty() {
                        for (q, msb, lsb, v) in late_nba.clone() {
                            self.store(&q, if msb < 0 { -1 } else { msb }, lsb, &v, &mut d2);
                            d2.assigned.push(q);
                        }
                        late_nba.clear();
                    }
                    let assigns_now: Vec<(String, Expr, Expr)> = self.all_assigns.clone();
                    for (pfx, lhs, rhs) in &assigns_now {
                        let f = Frame { module: 0, prefix: pfx.clone() };
                        let (q, msb, lsb) = self.lhs_target(lhs, &f);
                        let v = self.eval(rhs, &f);
                        if msb >= 0 {
                            self.store(&q, msb, lsb, &v, &mut d2);
                        } else {
                            self.store(&q, -1, -1, &v, &mut d2);
                        }
                        d2.assigned.push(q);
                    }
                    for (mi, prefix, ab) in always_blocks.iter() {
                        let is_comb = ab.kind == "always_comb" || ab.kind == "always_latch"
                            || (ab.kind == "always" && matches!(ab.body.first(), Some(Stmt::Event { edges, .. }) if edges.is_empty()))
                            || (ab.kind == "always" && !matches!(ab.body.first(), Some(Stmt::Event { .. })));
                        if !is_comb {
                            continue;
                        }
                        let f = Frame { module: *mi, prefix: prefix.clone() };
                        let mut sub = ExecSide::default();
                        let body: Vec<Stmt> = match ab.body.first() {
                            Some(Stmt::Event { body, .. }) => body.clone(),
                            _ => ab.body.clone(),
                        };
                        for st in &body {
                            self.exec_stmt(st, &f, &mut sub);
                        }
                        d2.printed.extend(sub.printed);
                        d2.touched.extend(sub.touched);
                        d2.nba.extend(sub.nba);
                        d2.assigned.extend(sub.assigned.clone());
                        if let Some(latched) = self.latch_active_by_block(*mi, ab) {
                            for lname in latched {
                                let assigned = sub.assigned.iter().any(|a| a == &lname);
                                let hold = !assigned;
                                self.latch_hold_now.insert(lname.clone(), hold);
                            }
                        }
                    }
                    self.sync_ports_out(&mut d2);
                    if !d2.nba.is_empty() {
                        for (q, msb, lsb, v) in d2.nba.clone() {
                            self.store(&q, if msb < 0 { -1 } else { msb }, lsb, &v, &mut d2);
                            d2.assigned.push(q);
                        }
                        d2.nba.clear();
                    }
                    if d2.touched.is_empty() || iter2 >= DELTA_CAP {
                        break;
                    }
                }
            }
            // 4. latch-hold shadow signals update
            let hold_names: Vec<(String, bool)> = self.latch_hold_now.iter().map(|(k, v)| (k.clone(), *v)).collect();
            for (sig, hold) in hold_names {
                let shadow = format!("{}.VIKAS_CAUSALITY_latch_hold_{}", self.design.top, sig.rsplit('.').next().unwrap_or(&sig));
                let v = VVal::from_u64(hold as u64, 1);
                if self.vals.get(&shadow) != Some(&v) {
                    if !self.vals.contains_key(&shadow) {
                        self.dump.add_signal(&shadow, 1);
                    }
                    self.vals.insert(shadow.clone(), v.clone());
                    out.touched.push(shadow);
                }
            }
            // 5. physics telemetry
            self.physics_tick(&mut out);
            // 6. taint propagation
            self.taint_tick(&mut out);
            // 7. clock cycle counting on main clock
            if let Some(clkq) = self.clock_names.first() {
                let q = if clkq.contains('.') { clkq.clone() } else { format!("{}.{}", self.design.top, clkq) };
                let cur = self.val(&q).to_u64();
                let old = self.prev.get(&q).map(|v| v.to_u64()).unwrap_or(cur);
                if old == 0 && cur == 1 {
                    self.cycles += 1;
                }
            }
            // 8. write outputs + dump
            for l in &out.printed {
                self.all_printed.push(l.clone());
                if !self.quiet {
                    print!("{}", l);
                }
            }
            self.dump_state();
            // 9. update prev for edge detection
            for (k, v) in self.vals.clone() {
                self.prev.insert(k, v);
            }
            // 10. advance time
            let nexts: Vec<u64> = parked.iter().map(|p| p.0).filter(|t| *t > self.cur_t).collect();
            let next = nexts.iter().min().copied();
            match next {
                Some(nt) if nt > self.cur_t => {
                    // before jumping, re-run delta at same time already done; jump
                    if nt > self.cur_t + 1 && self.finished {
                        break;
                    }
                    self.cur_t = nt;
                }
                Some(_) => {
                    self.cur_t += 1;
                }
                None => {
                    if self.finished {
                        break;
                    }
                    self.finished = true; // nothing left to do
                    break;
                }
            }
        }
        // final dump
        self.dump_state();
        SimResult {
            dump: std::mem::take(&mut self.dump),
            top: self.design.top.clone(),
            cycles: self.cycles,
            wall_ms: start.elapsed().as_millis(),
            glitches: self.glitch_total + self.glitch_counts.values().map(|v| *v as u64).sum::<u64>() / 4,
            energy_fj_total: self.phys_energy,
            latch_finds: std::mem::take(&mut self.latch_finds),
            printed: std::mem::take(&mut self.all_printed),
        }
    }

    fn latch_active_by_block(&self, mi: usize, ab: &AlwaysBlock) -> Option<Vec<String>> {
        let key = format!("{}:{}", mi, ab.line);
        self.latch_active.get(&key).cloned()
    }

    fn analyze_latches(&mut self, blocks: &[(usize, String, AlwaysBlock)]) {
        // L1 static: always_comb where a signal assigned in then-branch but missing in else
        for (mi, prefix, ab) in blocks {
            if ab.kind != "always_comb" && ab.kind != "always_latch" {
                continue;
            }
            let body: Vec<Stmt> = match ab.body.first() {
                Some(Stmt::Event { body, .. }) => body.clone(),
                Some(Stmt::Block(inner)) => inner.clone(),
                _ => ab.body.clone(),
            };
            let assigned_all: std::collections::HashSet<String> = collect_assigned(&body);
            let mut incomplete: std::collections::HashSet<String> = std::collections::HashSet::new();
            for st in &body {
                if let Stmt::If { cond, then, els } = st {
                    let ta: std::collections::HashSet<String> = collect_assigned(then);
                    let ea: std::collections::HashSet<String> = collect_assigned(els);
                    for s in &ta {
                        if !ea.contains(s) && els.is_empty() {
                            incomplete.insert(s.clone());
                        }
                    }
                    let _ = cond;
                }
                if let Stmt::Case { arms, default, .. } = st {
                    if default.is_none() {
                        let allc: std::collections::HashSet<String> =
                            arms.iter().flat_map(|(_, b)| collect_assigned(b).into_iter()).collect();
                        for s in &assigned_all {
                            if allc.contains(s) && !arms.iter().all(|(_, b)| collect_assigned(b).contains(s)) {
                                incomplete.insert(s.clone());
                            }
                        }
                    }
                }
            }
            for sig in incomplete {
                let fq = if sig.contains('.') { sig.clone() } else { format!("{}.{}", prefix, sig) };
                if !self.latch_finds.iter().any(|l| l.signal == fq) {
                    self.latch_finds.push(super::latchsim::LatchFind {
                        signal: fq.clone(),
                        block_kind: ab.kind.clone(),
                        line: ab.line,
                        hint: "incomplete if/case — missing else/default branch holds previous value".into(),
                        hold_events: 0,
                        hold_time_units: 0,
                        first_hold_t: None,
                    });
                }
                let key = format!("{}:{}", mi, ab.line);
                self.latch_active.entry(key).or_default().push(fq);
            }
        }
    }

    fn physics_tick(&mut self, out: &mut ExecSide) {
        // energy = 1/2 C V^2 per toggle (fF-scale Ceff heuristics), Vdd=0.8V
        let mut e = 0.0f64;
        let mut toggled = 0usize;
        for (k, v) in self.vals.iter() {
            if k.starts_with("VIKAS_") {
                continue;
            }
            let old = self.prev.get(k);
            if let Some(old) = old {
                if old.bits != v.bits && !v.is_x() && !old.is_x() {
                    let c = self.ceff.get(k).copied().unwrap_or(4.0); // fF
                    let bits_toggled = old.bits.iter().zip(&v.bits).filter(|(a, b)| a != b).count();
                    e += 0.5 * c * 0.64 * bits_toggled as f64; // fJ (V^2=0.64)
                    toggled += bits_toggled;
                }
            }
        }
        self.phys_energy += e;
        let top = &self.design.top;
        // instant energy + glitch hazard + transitions telemetry tracks
        let pairs = [
            ("VIKAS_PHYSICS_instant_energy_fJ", e),
            ("VIKAS_PHYSICS_cumulative_energy_fJ", self.phys_energy),
            ("VIKAS_CAUSALITY_transitions_this_cycle", toggled as f64),
        ];
        for (suffix, val) in pairs {
            let name = format!("{}.{}", top, suffix);
            let bits = VVal::from_u64(val as u64, 32);
            if self.vals.get(&name) != Some(&bits) {
                if !self.vals.contains_key(&name) {
                    self.dump.add_signal(&name, 32);
                }
                self.vals.insert(name.clone(), bits.clone());
                out.touched.push(name);
            }
        }
        // glitch hazard track: any net with >0 delta activity in the settled window
        let gh = self.glitch_counts.values().map(|v| *v as u64).sum::<u64>().min(1);
        let gname = format!("{}.VIKAS_CAUSALITY_subcycle_glitch_hazards", top);
        let gv = VVal::from_u64(gh, 1);
        if self.vals.get(&gname) != Some(&gv) {
            if !self.vals.contains_key(&gname) {
                self.dump.add_signal(&gname, 1);
            }
            self.vals.insert(gname.clone(), gv);
            out.touched.push(gname);
        }
    }

    fn taint_tick(&mut self, out: &mut ExecSide) {
        // simple forward dye: any input port taints what it drives (stored at $taint)
        let top = &self.design.top;
        let name = format!("{}.VIKAS_TAINT_active_dye", top);
        let tv = self.taint.values().fold(0u64, |a, b| a | b);
        let v = VVal::from_u64(tv, 64);
        if self.vals.get(&name) != Some(&v) {
            if !self.vals.contains_key(&name) {
                self.dump.add_signal(&name, 64);
            }
            self.vals.insert(name.clone(), v);
            out.touched.push(name);
        }
    }

    fn dump_state(&mut self) {
        // dump every value that changed since last snapshot
        let snapshot: Vec<(String, VVal)> = self.vals.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (k, v) in snapshot {
            if let Some(si) = self.dump.signals.iter().position(|s| s.name == k) {
                let last: Option<&VVal> = self.dump.changes.iter().rev().find(|c| c.sig == si).map(|c| &c.value);
                if last != Some(&v) {
                    self.dump.push(self.cur_t, si, v);
                }
            }
        }
    }

    fn collect_all_assigns(&mut self, mi: usize, prefix: &str) {
        let pairs: Vec<(Expr, Expr)> = self.mods[mi].assigns.iter().map(|(l, r, _)| (l.clone(), r.clone())).collect();
        for (l, r) in pairs {
            self.all_assigns.push((prefix.to_string(), l, r));
        }
        let kids: Vec<(String, usize)> = self.mods[mi]
            .instances
            .iter()
            .filter_map(|inst| {
                self.mods.iter().position(|x| x.name == inst.module).map(|ci| (inst.inst_name.clone(), ci))
            })
            .collect();
        for (name, ci) in kids {
            let iprefix = format!("{}.{}", prefix, name);
            self.collect_all_assigns(ci, &iprefix);
        }
    }

    fn collect_initials(&self, mi: usize, prefix: &str, out: &mut Vec<(usize, String, Vec<Stmt>, u32)>) {
        let m = &self.mods[mi];
        for ab in &m.always {
            if ab.kind == "initial" {
                out.push((mi, prefix.to_string(), ab.body.clone(), ab.line));
            }
        }
        for inst in &m.instances {
            if let Some(ci) = self.mods.iter().position(|x| x.name == inst.module) {
                let iprefix = format!("{}.{}", prefix, inst.inst_name);
                self.collect_initials(ci, &iprefix, out);
            }
        }
    }

    fn collect_always(&self, mi: usize, prefix: &str, out: &mut Vec<(usize, String, AlwaysBlock)>) {
        let m = &self.mods[mi];
        for ab in &m.always {
            if ab.kind != "initial" {
                out.push((mi, prefix.to_string(), ab.clone()));
            }
        }
        for inst in &m.instances {
            if let Some(ci) = self.mods.iter().position(|x| x.name == inst.module) {
                let iprefix = format!("{}.{}", prefix, inst.inst_name);
                self.collect_always(ci, &iprefix, out);
            }
        }
    }

    pub fn add_taint(&mut self, sig: &str, dye: u64) {
        self.taint.insert(sig.to_string(), dye);
    }
}

fn collect_assigned(body: &[Stmt]) -> std::collections::HashSet<String> {
    let mut s = std::collections::HashSet::new();
    fn walk(st: &Stmt, s: &mut std::collections::HashSet<String>) {
        match st {
            Stmt::Assign { lhs, .. } => {
                if let Expr::Ident(n) = lhs {
                    s.insert(n.clone());
                }
            }
            Stmt::If { then, els, .. } => {
                for x in then {
                    walk(x, s);
                }
                for x in els {
                    walk(x, s);
                }
            }
            Stmt::Case { arms, default, .. } => {
                for (_, b) in arms {
                    for x in b {
                        walk(x, s);
                    }
                }
                if let Some(d) = default {
                    for x in d {
                        walk(x, s);
                    }
                }
            }
            Stmt::Block(b) => {
                for x in b {
                    walk(x, s);
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::Repeat { body, .. } => {
                for x in body {
                    walk(x, s);
                }
            }
            _ => {}
        }
    }
    for st in body {
        walk(st, &mut s);
    }
    s
}

#[derive(Default)]
pub struct ExecSide {
    printed: Vec<String>,
    touched: Vec<String>,
    nba: Vec<(String, i64, i64, VVal)>,
    assigned: Vec<String>,
    park_at: Option<u64>,
    parked_tail: Vec<Stmt>,
    parked_wait: Option<Expr>,
}
