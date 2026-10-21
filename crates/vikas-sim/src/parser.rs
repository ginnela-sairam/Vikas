//! VIKAS v1 — XEZIM kernel parser (AST).
//! Parses modules, ports, params, always blocks, assign, initial, decls.

use super::lexer::Tok;

#[derive(Clone, Debug)]
pub enum Expr {
    Num(u64, usize),
    Str(String),
    Ident(String),
    Bin(String, Box<Expr>, Box<Expr>),
    Un(String, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),       // a[i]
    Slice(Box<Expr>, Box<Expr>, Box<Expr>),   // a[msb:lsb]
    Call(String, Vec<Expr>),           // function/system call
    Concat(Vec<Expr>),                 // {a,b,c}
    Replicate(Box<Expr>, Box<Expr>),   // {n{a}}
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign { lhs: Expr, rhs: Expr, blocking: bool },
    If { cond: Expr, then: Vec<Stmt>, els: Vec<Stmt> },
    Case { subject: Expr, arms: Vec<(Vec<Expr>, Vec<Stmt>)>, default: Option<Vec<Stmt>> },
    For { init: Expr, cond: Expr, step: Expr, body: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Repeat { count: Expr, body: Vec<Stmt> },
    Delay { ns: u64, body: Vec<Stmt> },
    Event { edges: Vec<(Edge, String)>, body: Vec<Stmt> }, // @(...)
    Wait { cond: Expr, body: Vec<Stmt> },
    Decl { name: String, width: usize, init: Option<Expr>, dims: Vec<(i32, i32)>, kind: VarKind },
    FCall(String, Vec<Expr>),
    Block(Vec<Stmt>),
    SysTask(String, Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    Pos,
    Neg,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VarKind {
    Wire,
    Reg,
    Logic,
    Int,
    Real,
    Genvar,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub name: String,
    pub width: usize,
    pub dir: u8, // 1=in 2=out 3=inout
}

#[derive(Clone, Debug)]
pub struct AlwaysBlock {
    pub body: Vec<Stmt>,
    pub kind: String, // always / always_comb / always_ff / always_latch / initial
    pub line: u32,
    pub file: String,
}

#[derive(Clone, Debug, Default)]
pub struct ModuleDef {
    pub name: String,
    pub params: Vec<(String, Option<Expr>)>,
    pub ports: Vec<Port>,
    pub signals: Vec<Stmt>, // Decl statements in order
    pub assigns: Vec<(Expr, Expr, u32)>, // lhs, rhs, line
    pub always: Vec<AlwaysBlock>,
    pub file: String,
    pub instances: Vec<InstanceRef>,
    pub functions: Vec<(String, Vec<(String, usize)>, Vec<Stmt>)>,
    pub enums: Vec<(String, Vec<(String, u64)>)>,
    pub typedefs: Vec<(String, usize)>,
    pub children: Vec<(String, Vec<(String, Expr)>)>, // module instantiation helpers
}

#[derive(Clone, Debug)]
pub struct InstanceRef {
    pub module: String,
    pub inst_name: String,
    pub conns: Vec<(String, ConnExpr)>,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub enum ConnExpr {
    Sig(String, i32, i32), // name, msb, lsb (bit/slice)
    Num(u64, usize),
    Concat(Vec<ConnExpr>),
    /// arbitrary connection expression: `.we(state == IDLE && en)` (KERNEL
    /// enhancement #21 — expression port connections, industry-common RTL)
    Expr(Box<Expr>),
    Wildcard,
}

pub struct Parser<'a> {
    t: &'a [Tok],
    ln: &'a [u32],
    pub p: usize,
    /// compile-time constant table: file-scope AND module-scope parameters.
    /// Range expressions are const-folded at parse time so widths are correct.
    pub consts: std::collections::HashMap<String, u64>,
}

impl<'a> Parser<'a> {
    pub fn new(t: &'a [Tok], ln: &'a [u32]) -> Parser<'a> {
        Parser { t, ln, p: 0, consts: std::collections::HashMap::new() }
    }

    /// fold an expression against the current constant table
    fn fold(&self, e: &Expr) -> Option<u64> {
        fold_const(e, &self.consts)
    }

    /// record `parameter NAME = expr;` into the constant table
    fn record_const(&mut self, name: &str, default: &Option<Expr>) {
        if let Some(e) = default {
            if let Some(v) = self.fold(e) {
                self.consts.insert(name.to_string(), v);
            }
        }
    }

    pub fn line(&self) -> u32 {
        self.ln.get(self.p).copied().unwrap_or(0)
    }

    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.p)
    }
    fn peek_op(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if o == s)
    }
    fn peek_ident(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(i)) if i == s)
    }
    fn eat(&mut self, s: &str) -> bool {
        if self.peek_op(s) || self.peek_ident(s) {
            self.p += 1;
            true
        } else {
            false
        }
    }
    fn expect_op(&mut self, s: &str) {
        if !self.eat(s) {
            self.p += 1; // error-tolerant: skip and continue
        }
    }
    fn ident(&mut self) -> String {
        match self.peek() {
            Some(Tok::Ident(s)) => {
                let v = s.clone();
                self.p += 1;
                v
            }
            _ => "?".into(),
        }
    }
    fn number(&mut self) -> Expr {
        if let Some(Tok::Num { val, width, .. }) = self.peek() {
            let v = *val;
            let w = *width;
            self.p += 1;
            Expr::Num(v, w.unwrap_or(32))
        } else if self.peek_ident("'") {
            self.p += 1;
            self.number()
        } else {
            Expr::Num(0, 32)
        }
    }

    fn skip_balanced(&mut self, open: &str, close: &str) {
        // assumes current token is `open`
        let mut depth = 0;
        loop {
            match self.peek() {
                None => return,
                Some(Tok::Op(o)) => {
                    if o == open {
                        depth += 1;
                    } else if o == close {
                        depth -= 1;
                        self.p += 1;
                        if depth == 0 {
                            return;
                        }
                        continue;
                    }
                    self.p += 1;
                }
                Some(_) => self.p += 1,
            }
        }
    }

    pub fn parse_program(&mut self) -> Vec<ModuleDef> {
        let mut mods = Vec::new();
        while self.p < self.t.len() {
            // file-scope parameter/localparam: enters the constant table so
            // `include`d headers (defs.vh) define widths for later modules
            if self.peek_ident("parameter") || self.peek_ident("localparam") {
                self.p += 1;
                let pn = self.ident();
                let mut default = None;
                if self.eat("=") {
                    default = Some(self.parse_expr());
                }
                self.eat(";");
                self.record_const(&pn, &default);
                continue;
            }
            if self.peek_ident("module") {
                if let Some(m) = self.parse_module() {
                    mods.push(m);
                }
            } else {
                self.p += 1;
            }
        }
        mods
    }

    fn parse_module(&mut self) -> Option<ModuleDef> {
        self.p += 1; // 'module'
        let name = self.ident();
        let file = String::new();
        let mut m = ModuleDef { name: name.clone(), file: file.clone(), ..Default::default() };
        // parameter list
        if self.peek_op("#") {
            self.p += 1;
            self.expect_op("(");
            while !self.peek_op(")") && self.p < self.t.len() {
                if self.eat("parameter") || self.eat("localparam") {
                    let pn = self.ident();
                    let mut default = None;
                    if self.eat("=") {
                        let e = self.parse_expr();
                        default = Some(e);
                    }
                    self.record_const(&pn, &default);
                    m.params.push((pn, default));
                } else {
                    self.p += 1;
                }
            }
            self.expect_op(")");
        }
        // port list (ANSI style: input [net_type] [packed_range] name) — OPTIONAL: portless modules
        if self.peek_op("(") {
            self.p += 1;
            let mut cur_dir = 1u8;
            let mut cur_width = 1usize;
            while !self.peek_op(")") && self.p < self.t.len() {
                if self.eat("input") {
                    cur_dir = 1;
                    cur_width = 1;
                } else if self.eat("output") {
                    cur_dir = 2;
                    cur_width = 1;
                } else if self.eat("inout") {
                    cur_dir = 3;
                    cur_width = 1;
                } else if self.eat("logic") || self.eat("wire") || self.eat("reg") || self.eat("bit") || self.eat("integer") || self.eat("signed") || self.eat("unsigned") || self.eat("var") {
                    // net type keyword between direction and range/name
                } else if self.peek_op("[") {
                    // packed range BEFORE the port name — const-folded like parse_range
                    self.p += 1;
                    let mut msb = self.parse_expr();
                    let mut lsb = if self.eat(":") { self.parse_expr() } else { msb.clone() };
                    self.expect_op("]");
                    for e in [&mut msb, &mut lsb] {
                        if let Expr::Bin(op, x, y) = e {
                            if op == "-" {
                                if let (Some(xv), Some(yv)) = (self.fold(x), self.fold(y)) {
                                    *e = Expr::Num(xv.wrapping_sub(yv), 32);
                                }
                            }
                        }
                        if let Expr::Ident(n) = e {
                            if let Some(v) = self.consts.get(n.as_str()).copied() {
                                *e = Expr::Num(v, 32);
                            }
                        }
                    }
                    cur_width = match (&msb, &lsb) {
                        (Expr::Num(a, _), Expr::Num(b, _)) => (*a as i64 - *b as i64 + 1).max(1) as usize,
                        _ => 1,
                    };
                } else if let Some(Tok::Ident(pn)) = self.peek().cloned() {
                    self.p += 1;
                    // optional unpacked dims after name
                    if self.peek_op("[") {
                        self.p += 1;
                        self.skip_balanced("[", "]");
                    }
                    m.ports.push(Port { name: pn, width: cur_width, dir: cur_dir });
                    if !self.eat(",") {
                        // continue
                    }
                } else {
                    self.p += 1;
                }
            }
            self.expect_op(")");
        }
        self.expect_op(";");
        // body
        while self.p < self.t.len() {
            if self.peek_ident("endmodule") {
                self.p += 1;
                break;
            }
            self.parse_module_item(&mut m);
        }
        Some(m)
    }

    fn parse_range(&mut self) -> (usize, Vec<(i32, i32)>) {
        // returns (width, dims) — consumes [a:b] [c:d] ...
        // Ranges const-fold through the parameter table so `[DATA_W-1:0]`
        // resolves to the true width (parser-level parameter substitution).
        let mut width = 1usize;
        let mut dims = Vec::new();
        while self.peek_op("[") {
            self.p += 1;
            let mut a = self.parse_expr();
            let mut b = a.clone();
            if self.eat(":") {
                b = self.parse_expr();
            }
            self.expect_op("]");
            if let Expr::Bin(op, x, y) = &a {
                if op == "-" {
                    if let (Some(xv), Some(yv)) = (self.fold(x), self.fold(y)) {
                        a = Expr::Num(xv.wrapping_sub(yv), 32);
                    }
                }
            }
            if let Expr::Bin(op, x, y) = &b {
                if op == "-" {
                    if let (Some(xv), Some(yv)) = (self.fold(x), self.fold(y)) {
                        b = Expr::Num(xv.wrapping_sub(yv), 32);
                    }
                }
            }
            if let Expr::Ident(n) = &a {
                if let Some(v) = self.consts.get(n.as_str()).copied() {
                    a = Expr::Num(v, 32);
                }
            }
            if let Expr::Ident(n) = &b {
                if let Some(v) = self.consts.get(n.as_str()).copied() {
                    b = Expr::Num(v, 32);
                }
            }
            let (av, bv) = match (&a, &b) {
                (Expr::Num(x, _), Expr::Num(y, _)) => (*x, *y),
                _ => (0, 0),
            };
            dims.push((av as i32, bv as i32));
            width = (av as i64 - bv as i64 + 1).max(1) as usize;
        }
        (width, dims)
    }

    fn parse_module_item(&mut self, m: &mut ModuleDef) {
        let line = self.line();
        if self.eat("parameter") || self.eat("localparam") {
            let pn = self.ident();
            let mut default = None;
            if self.eat("=") {
                default = Some(self.parse_expr());
            }
            self.record_const(&pn, &default);
            m.params.push((pn, default));
            self.eat(";");
        } else if self.eat("typedef") {
            // typedef enum / typedef logic [..] name;
            if self.eat("enum") {
                self.expect_op("{");
                let mut vals = Vec::new();
                let mut cnt = 0u64;
                while !self.peek_op("}") && self.p < self.t.len() {
                    let vn = self.ident();
                    let v = if self.eat("=") {
                        match self.parse_expr() {
                            Expr::Num(x, _) => x,
                            _ => cnt,
                        }
                    } else {
                        cnt
                    };
                    cnt = v + 1;
                    vals.push((vn, v));
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_op("}");
                let tn = self.ident();
                m.enums.push((tn, vals));
            } else {
                self.parse_range();
                let tn = self.ident();
                let _ = tn;
            }
            self.eat(";");
        } else if self.eat("enum") {
            self.expect_op("{");
            let mut vals = Vec::new();
            let mut cnt = 0u64;
            while !self.peek_op("}") && self.p < self.t.len() {
                let vn = self.ident();
                let v = if self.eat("=") {
                    match self.parse_expr() {
                        Expr::Num(x, _) => x,
                        _ => cnt,
                    }
                } else {
                    cnt
                };
                cnt = v + 1;
                vals.push((vn, v));
                if !self.eat(",") {
                    break;
                }
            }
            self.expect_op("}");
            let tn = self.ident();
            m.enums.push((tn, vals));
            self.eat(";");
        } else if self.eat("function") {
            // automatic/type [range] name(args); decls; body; endfunction
            self.parse_range();
            let fname = self.ident();
            let mut args = Vec::new();
            self.expect_op("(");
            while !self.peek_op(")") && self.p < self.t.len() {
                if self.eat("input") || self.eat("integer") || self.eat("logic") || self.eat("reg") || self.eat("wire") {
                    // kind
                }
                let (w, _) = self.parse_range();
                let an = self.ident();
                args.push((an, w));
                if !self.eat(",") {
                    // continue
                }
            }
            self.expect_op(")");
            self.eat(";");
            let mut body = Vec::new();
            let mut fname_ret = fname.clone();
            while !self.peek_ident("endfunction") && self.p < self.t.len() {
                if self.peek_ident("function") {
                    break;
                }
                if let Some(s) = self.parse_stmt() {
                    body.push(s);
                }
            }
            self.eat("endfunction");
            if body.is_empty() {
                // common style: name = expr;
                let _ = &mut fname_ret;
            }
            m.functions.push((fname, args, body));
        } else if self.eat("assign") {
            let lhs = self.parse_lhs();
            self.expect_op("=");
            let rhs = self.parse_expr();
            m.assigns.push((lhs, rhs, line));
            self.eat(";");
        } else if self.eat("initial") {
            let body = self.parse_stmt_block_or_single();
            m.always.push(AlwaysBlock { body, kind: "initial".into(), line, file: m.file.clone() });
        } else if self.eat("always") {
            let kind = if self.peek_op("@") {
                "always".to_string()
            } else {
                "always".to_string()
            };
            let body = self.parse_always_tail();
            m.always.push(AlwaysBlock { body, kind, line, file: m.file.clone() });
        } else if self.eat("always_comb") {
            let body = self.parse_stmt_block_or_single();
            m.always.push(AlwaysBlock { body, kind: "always_comb".into(), line, file: m.file.clone() });
        } else if self.eat("always_ff") {
            let body = self.parse_always_tail();
            m.always.push(AlwaysBlock { body, kind: "always_ff".into(), line, file: m.file.clone() });
        } else if self.eat("always_latch") {
            let body = self.parse_stmt_block_or_single();
            m.always.push(AlwaysBlock { body, kind: "always_latch".into(), line, file: m.file.clone() });
        } else if self.eat("genvar") || self.eat("gen") {
            self.p += 0;
            let _ = self.parse_range();
            let n = self.ident();
            m.signals.push(Stmt::Decl { name: n, width: 32, init: None, dims: vec![], kind: VarKind::Genvar });
            self.eat(",");
            self.eat(";");
        } else if self.eat("generate") || self.eat("endgenerate") {
            self.p += 0; // skip
        } else if self.eat("wire") || self.eat("reg") || self.eat("logic") || self.eat("integer") || self.eat("real") || self.eat("bit") || self.eat("byte") || self.eat("int") {
            let kind = match self.t[self.p - 1] {
                Tok::Ident(ref s) if s == "wire" => VarKind::Wire,
                Tok::Ident(ref s) if s == "reg" => VarKind::Reg,
                Tok::Ident(ref s) if s == "integer" || s == "int" => VarKind::Int,
                Tok::Ident(ref s) if s == "real" => VarKind::Real,
                Tok::Ident(ref s) if s == "bit" => VarKind::Logic,
                Tok::Ident(ref s) if s == "byte" => VarKind::Logic,
                _ => VarKind::Logic,
            };
            // multiple declarators: `logic [15:0] a, b, c;` — the packed range
            // applies to ALL names (Verilog semantics). Optional per-name ranges
            // after the name are treated as unpacked array dims.
            {
                let (base_w, base_dims) = self.parse_range();
                // KERNEL FIX #22: `integer`/`int` with no packed range is 32-bit
                // (was width-1: every `f = f + 1` truncated and counters wrapped
                // at 2 — found via the NOVA-SoC TB fails-counter probe)
                let decl_w = if matches!(kind, VarKind::Int | VarKind::Real) && base_w == 1 {
                    32
                } else {
                    base_w.max(1)
                };
                loop {
                    let n = self.ident();
                    let (_w2, extra_dims) = self.parse_range();
                    let mut init = None;
                    if self.eat("=") {
                        init = Some(self.parse_expr());
                    }
                    let mut dims = base_dims.clone();
                    dims.extend(extra_dims);
                    m.signals.push(Stmt::Decl { name: n, width: decl_w, init, dims, kind });
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            self.eat(";");
        } else if let Some(Tok::Ident(inst_module)) = self.peek().cloned() {
            // module instantiation: module inst (...) or module #(...) inst (...);
            self.p += 1;
            if self.peek_op("#") {
                self.p += 1;
                self.expect_op("(");
                let mut depth = 1;
                while depth > 0 && self.p < self.t.len() {
                    match self.peek() {
                        Some(Tok::Op(o)) if o == "(" => {
                            depth += 1;
                            self.p += 1;
                        }
                        Some(Tok::Op(o)) if o == ")" => {
                            depth -= 1;
                            self.p += 1;
                        }
                        Some(_) => self.p += 1,
                        None => break,
                    }
                }
            }
            let inst_name = self.ident();
            self.expect_op("(");
            let mut conns = Vec::new();
            if self.eat("*") {
                conns.push(("*".to_string(), ConnExpr::Wildcard));
                self.expect_op(")");
            } else {
                while !self.peek_op(")") && self.p < self.t.len() {
                    if self.eat(".") {
                        let port = self.ident();
                        self.expect_op("(");
                        let ce = self.parse_conn();
                        self.expect_op(")");
                        conns.push((port, ce));
                    } else {
                        // positional connection
                        let ce = self.parse_conn();
                        conns.push((format!("#{}", conns.len()), ce));
                    }
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_op(")");
            }
            self.eat(";");
            m.instances.push(InstanceRef { module: inst_module, inst_name, conns, line });
        } else {
            self.p += 1;
        }
    }

    fn parse_conn(&mut self) -> ConnExpr {
        // KERNEL enhancement #21: try the simple lvalue-ish shapes first; if the
        // connection continues with operators (e.g. `.we(a && b)`, `.req(x | y)`,
        // `.data(sel ? a : b)`), rewind and parse a full expression instead —
        // previously such bindings were silently DROPPED (port stayed X).
        let save = self.p;
        let simple = match self.peek().cloned() {
            Some(Tok::Num { val, width, .. }) => {
                self.p += 1;
                Some(ConnExpr::Num(val, width.unwrap_or(32)))
            }
            Some(Tok::Ident(n)) => {
                self.p += 1;
                let mut msb = -1i32;
                let mut lsb = -1i32;
                if self.peek_op("[") {
                    self.p += 1;
                    let a = self.parse_expr();
                    let mut b = a.clone();
                    if self.eat(":") {
                        b = self.parse_expr();
                    }
                    self.expect_op("]");
                    if let (Expr::Num(x, _), Expr::Num(y, _)) = (&a, &b) {
                        msb = *x as i32;
                        lsb = *y as i32;
                    }
                }
                Some(ConnExpr::Sig(n, msb, lsb))
            }
            Some(Tok::Op(o)) if o == "{" => {
                self.p += 1;
                let mut items = Vec::new();
                while !self.peek_op("}") && self.p < self.t.len() {
                    items.push(self.parse_conn());
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_op("}");
                Some(ConnExpr::Concat(items))
            }
            _ => None,
        };
        // continue tokens mean the simple shape didn't cover the whole
        // connection — rewind and take the general-expression path
        let continues = match self.peek() {
            Some(Tok::Op(o)) => !matches!(o.as_str(), ")" | "," | "}"),
            _ => false,
        };
        if simple.is_some() && !continues {
            return simple.unwrap();
        }
        self.p = save;
        match self.peek() {
            Some(Tok::Op(o)) if o == "{" => {
                // concat of general expressions
                self.p += 1;
                let mut items = Vec::new();
                while !self.peek_op("}") && self.p < self.t.len() {
                    items.push(ConnExpr::Expr(Box::new(self.parse_expr())));
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_op("}");
                ConnExpr::Concat(items)
            }
            _ => {
                let e = self.parse_expr();
                if matches!(e, Expr::Num(0, 1)) && self.p == save + 1 {
                    // nothing meaningful parsed — skip token (previous tolerance)
                    ConnExpr::Wildcard
                } else {
                    ConnExpr::Expr(Box::new(e))
                }
            }
        }
    }

    fn parse_always_tail(&mut self) -> Vec<Stmt> {
        // @(...) block
        if self.peek_op("@") {
            self.p += 1;
            if self.peek_op("(") || self.peek_op("*") {
                if self.eat("*") {
                    return self.parse_stmt_block_or_single();
                }
                self.p += 1; // (
                let mut edges = Vec::new();
                while !self.peek_op(")") && self.p < self.t.len() {
                    let e = if self.eat("posedge") {
                        Edge::Pos
                    } else if self.eat("negedge") {
                        Edge::Neg
                    } else {
                        Edge::Both
                    };
                    let n = self.ident();
                    edges.push((e, n));
                    self.eat("or");
                }
                self.expect_op(")");
                let body = self.parse_stmt_block_or_single();
                return vec![Stmt::Event { edges, body }];
            }
        }
        self.parse_stmt_block_or_single()
    }

    fn parse_stmt_block_or_single(&mut self) -> Vec<Stmt> {
        if self.peek_op("{") || (self.peek_op("begin") && { self.p += 1; true }) {
            let mut body = Vec::new();
            while !self.peek_ident("end") && self.p < self.t.len() {
                if let Some(s) = self.parse_stmt() {
                    body.push(s);
                }
            }
            self.eat("end");
            body
        } else if let Some(s) = self.parse_stmt() {
            vec![s]
        } else {
            Vec::new()
        }
    }

    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        // boundary keywords: parse_stmt never STARTS at these (case-arm terminators)
        if self.peek_ident("end") || self.peek_ident("endmodule") || self.peek_ident("endfunction")
            || self.peek_ident("endcase") || self.peek_ident("default") || self.peek_ident("else") {
            return None;
        }
        let line = self.line();
        if self.eat(";") {
            return Some(Stmt::Block(vec![])); // null statement
        }
        if self.eat("begin") {
            let mut body = Vec::new();
            while !self.peek_ident("end") && self.p < self.t.len() {
                match self.parse_stmt() {
                    Some(s) => body.push(s),
                    None => break,
                }
            }
            self.eat("end");
            return Some(Stmt::Block(body));
        }
        if self.eat("begin") {
            // unreachable
        }
        if self.peek_op("@") {
            self.p += 1;
            let mut edges = Vec::new();
            if self.peek_op("*") {
                self.p += 1;
                let body = self.parse_stmt_block_or_single();
                return Some(Stmt::Event { edges, body });
            }
            if self.peek_op("(") {
                self.p += 1;
                while !self.peek_op(")") && self.p < self.t.len() {
                    let e = if self.eat("posedge") {
                        Edge::Pos
                    } else if self.eat("negedge") {
                        Edge::Neg
                    } else {
                        Edge::Both
                    };
                    let n = self.ident();
                    edges.push((e, n));
                    self.eat("or");
                }
                self.expect_op(")");
            } else {
                let e = if self.eat("posedge") {
                    Edge::Pos
                } else if self.eat("negedge") {
                    Edge::Neg
                } else {
                    Edge::Both
                };
                let n = self.ident();
                edges.push((e, n));
            }
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::Event { edges, body });
        }
        if self.eat("if") {
            self.expect_op("(");
            let cond = self.parse_expr();
            self.expect_op(")");
            let then = self.parse_stmt_block_or_single();
            let els = if self.eat("else") {
                if self.peek_ident("if") {
                    match self.parse_stmt() {
                        Some(s) => vec![s],
                        None => vec![],
                    }
                } else {
                    self.parse_stmt_block_or_single()
                }
            } else {
                vec![]
            };
            return Some(Stmt::If { cond, then, els });
        }
        if self.eat("case") || self.eat("casez") || self.eat("casex") {
            self.expect_op("(");
            let subject = self.parse_expr();
            self.expect_op(")");
            let mut arms = Vec::new();
            let mut default = None;
            while !self.peek_ident("endcase") && self.p < self.t.len() {
                // default arm: first-class branch (never falls into the empty-label skip)
                if self.eat("default") {
                    self.expect_op(":");
                    let body = self.parse_stmt_block_or_single();
                    default = Some(body);
                    continue;
                }
                // gather labels
                let mut labels = Vec::new();
                loop {
                    labels.push(self.parse_expr());
                    if !self.eat(",") {
                        break;
                    }
                }
                if labels.is_empty() {
                    self.p += 1; // malformed junk token — skip safely
                    continue;
                }
                self.eat(":");
                let body = self.parse_stmt_block_or_single();
                arms.push((labels, body));
            }
            self.eat("endcase");
            return Some(Stmt::Case { subject, arms, default });
        }
        if self.eat("for") {
            self.expect_op("(");
            // optional loop-var type keyword: for (int i = 0; ...) — skipped
            let _ = self.eat("int") || self.eat("integer") || self.eat("genvar") || self.eat("logic");
            let init = self.parse_lhs();
            self.expect_op("=");
            let iv = self.parse_expr();
            self.expect_op(";");
            let cond = self.parse_expr();
            self.expect_op(";");
            let step = self.parse_lhs();
            let sop = if self.eat("+=") {
                "+="
            } else if self.eat("-=") {
                "-="
            } else {
                self.eat("=");
                "="
            };
            let sv = self.parse_expr();
            self.expect_op(")");
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::For {
                init: Expr::Bin("=".into(), Box::new(init), Box::new(iv)),
                cond,
                step: Expr::Bin(sop.into(), Box::new(step), Box::new(sv)),
                body,
            });
        }
        if self.eat("while") {
            self.expect_op("(");
            let cond = self.parse_expr();
            self.expect_op(")");
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::While { cond, body });
        }
        if self.eat("repeat") {
            self.expect_op("(");
            let count = self.parse_expr();
            self.expect_op(")");
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::Repeat { count, body });
        }
        if self.eat("forever") {
            // forever <stmt>  ==  while(1) <stmt>  (clock generators)
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::While { cond: Expr::Num(1, 1), body });
        }
        if self.eat("wait") {
            self.expect_op("(");
            let cond = self.parse_expr();
            self.expect_op(")");
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::Wait { cond, body });
        }
        // delay #10 or #10ns
        if self.peek_op("#") {
            self.p += 1;
            let (ns, is_expr) = match self.peek().cloned() {
                Some(Tok::Num { val, .. }) => {
                    self.p += 1;
                    (val, false)
                }
                Some(Tok::Ident(_)) => {
                    // named delay like #TICK — evaluate as expr
                    let e = self.parse_expr();
                    (0, true)
                }
                _ => (1, false),
            };
            let _ = is_expr;
            let body = self.parse_stmt_block_or_single();
            return Some(Stmt::Delay { ns, body });
        }
        if self.peek_op("$") || matches!(self.peek(), Some(Tok::Ident(s)) if s.starts_with('$')) {
            let name = self.ident();
            let mut args = Vec::new();
            if self.peek_op("(") {
                self.p += 1;
                while !self.peek_op(")") && self.p < self.t.len() {
                    args.push(self.parse_expr());
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_op(")");
            }
            self.eat(";");
            return Some(Stmt::SysTask(name, args));
        }
        // declaration inside block (wire/reg/logic/integer)
        if self.eat("reg") || self.eat("wire") || self.eat("logic") || self.eat("integer") {
            let kind = match self.t[self.p - 1] {
                Tok::Ident(ref s) if s == "reg" => VarKind::Reg,
                Tok::Ident(ref s) if s == "wire" => VarKind::Wire,
                Tok::Ident(ref s) if s == "integer" => VarKind::Int,
                _ => VarKind::Logic,
            };
            let (w, dims) = self.parse_range();
            let n = self.ident();
            let init = if self.eat("=") {
                Some(self.parse_expr())
            } else {
                None
            };
            self.eat(";");
            return Some(Stmt::Decl { name: n, width: w.max(1), init, dims, kind });
        }
        // assignment: LHS = / <= RHS  — parse LHS specially so <= stays NBA
        let lhs = self.parse_lhs();
        if self.eat("<=") {
            let rhs = self.parse_expr();
            self.eat(";");
            return Some(Stmt::Assign { lhs, rhs, blocking: false });
        }
        if self.peek_op("=") || self.peek_op("+=") || self.peek_op("-=") {
            let op = match self.peek() {
                Some(Tok::Op(o)) => o.clone(),
                _ => "=".into(),
            };
            self.p += 1;
            let rhs = self.parse_expr();
            let rhs2 = if op == "+=" || op == "-=" {
                Expr::Bin(op[..1].to_string(), Box::new(lhs.clone()), Box::new(rhs))
            } else {
                rhs
            };
            self.eat(";");
            return Some(Stmt::Assign { lhs, rhs: rhs2, blocking: true });
        }
        if self.peek_op(":") {
            // label — skip
            self.p += 1;
            return self.parse_stmt();
        }
        // function/task call statement
        if let Some(Tok::Ident(_)) = self.peek() {
            let save = self.p;
            let name = self.ident();
            if self.peek_op("(") {
                self.p += 1;
                let mut args = Vec::new();
                while !self.peek_op(")") && self.p < self.t.len() {
                    args.push(self.parse_expr());
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect_op(")");
                self.eat(";");
                return Some(Stmt::FCall(name, args));
            }
            self.p = save;
        }
        let _ = line;
        self.p += 1; // skip unknown token
        Some(Stmt::Block(vec![]))
    }

    /// LHS parse: ident, ident[expr], ident[a:b] — never consumes <= as comparison.
    fn parse_lhs(&mut self) -> Expr {
        let base = match self.peek().cloned() {
            Some(Tok::Ident(n)) => {
                self.p += 1;
                Expr::Ident(n)
            }
            Some(Tok::Num { val, width, .. }) => {
                self.p += 1;
                Expr::Num(val, width.unwrap_or(32))
            }
            _ => return Expr::Ident("?".into()),
        };
        if self.peek_op("[") {
            self.p += 1;
            let a = self.parse_expr();
            if self.eat(":") {
                let b = self.parse_expr();
                self.expect_op("]");
                return Expr::Slice(Box::new(base), Box::new(a), Box::new(b));
            }
            self.expect_op("]");
            return Expr::Index(Box::new(base), Box::new(a));
        }
        base
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Expr {
        let c = self.parse_bin(0);
        if self.eat("?") {
            let t = self.parse_ternary();
            self.expect_op(":");
            let e = self.parse_ternary();
            return Expr::Ternary(Box::new(c), Box::new(t), Box::new(e));
        }
        c
    }

    fn parse_bin(&mut self, lvl: u8) -> Expr {
        const LEVELS: [&[&str]; 9] = [
            &["||"],
            &["&&"],
            &["|"],
            &["^", "~^", "^~"],
            &["&"],
            &["==", "!=", "<", "<=", ">", ">="],
            &["<<", ">>"],
            &["+", "-"],
            &["*", "/", "%", "**"],
        ];
        if lvl as usize >= LEVELS.len() {
            return self.parse_unary();
        }
        let mut lhs = self.parse_bin(lvl + 1);
        loop {
            let matched = LEVELS[lvl as usize].iter().any(|op| self.peek_op(op));
            if !matched {
                return lhs;
            }
            let op = match self.peek() {
                Some(Tok::Op(o)) => o.clone(),
                _ => unreachable!(),
            };
            self.p += 1;
            let rhs = self.parse_bin(lvl + 1);
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
    }

    fn parse_unary(&mut self) -> Expr {
        if self.eat("!") || self.eat("~") || self.eat("-") || self.eat("+") || self.eat("&") || self.eat("|") || self.eat("^") || self.eat("~&") || self.eat("~|") {
            let op = match self.t[self.p - 1] {
                Tok::Op(ref o) => o.clone(),
                _ => "?".into(),
            };
            let e = self.parse_unary();
            return Expr::Un(op, Box::new(e));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek().cloned() {
            Some(Tok::Num { val, width, .. }) => {
                self.p += 1;
                Expr::Num(val, width.unwrap_or(32))
            }
            Some(Tok::Str(s)) => {
                self.p += 1;
                Expr::Str(s)
            }
            Some(Tok::Ident(n)) => {
                self.p += 1;
                if n.starts_with('$') {
                    let mut args = Vec::new();
                    if self.peek_op("(") {
                        self.p += 1;
                        while !self.peek_op(")") && self.p < self.t.len() {
                            args.push(self.parse_expr());
                            if !self.eat(",") {
                                break;
                            }
                        }
                        self.expect_op(")");
                    }
                    return Expr::Call(n, args);
                }
                if self.peek_op("(") {
                    // function call (user functions)
                    self.p += 1;
                    let mut args = Vec::new();
                    while !self.peek_op(")") && self.p < self.t.len() {
                        args.push(self.parse_expr());
                        if !self.eat(",") {
                            break;
                        }
                    }
                    self.expect_op(")");
                    return Expr::Call(n, args);
                }
                let mut e = Expr::Ident(n);
                while self.peek_op("[") {
                    self.p += 1;
                    let a = self.parse_expr();
                    if self.eat(":") {
                        let b = self.parse_expr();
                        self.expect_op("]");
                        e = Expr::Slice(Box::new(e), Box::new(a), Box::new(b));
                    } else {
                        self.expect_op("]");
                        e = Expr::Index(Box::new(e), Box::new(a));
                    }
                }
                e
            }
            Some(Tok::Op(o)) if o == "(" => {
                self.p += 1;
                let e = self.parse_expr();
                self.expect_op(")");
                e
            }
            Some(Tok::Op(o)) if o == "{" => {
                self.p += 1;
                // replication {n{v}} or concat {a,b}
                let mut items = Vec::new();
                let mut count: Option<Expr> = None;
                // try replication: expr { expr , ... }
                let save = self.p;
                let first = self.parse_expr();
                if self.peek_op("{") {
                    self.p += 1;
                    let mut inner = Vec::new();
                    while !self.peek_op("}") && self.p < self.t.len() {
                        inner.push(self.parse_expr());
                        if !self.eat(",") {
                            break;
                        }
                    }
                    self.expect_op("}");
                    self.expect_op("}");
                    return Expr::Replicate(Box::new(first), Box::new(Expr::Concat(inner)));
                }
                items.push(first);
                while self.eat(",") {
                    if self.peek_op("}") {
                        break;
                    }
                    items.push(self.parse_expr());
                }
                self.expect_op("}");
                let _ = save;
                let _ = &mut count;
                Expr::Concat(items)
            }
            Some(Tok::Op(o)) if o == "$" => {
                self.p += 1;
                Expr::Ident("$".into())
            }
            _ => {
                self.p += 1;
                Expr::Num(0, 1)
            }
        }
    }
}

/// Fold an expression to a constant using a name->value table.
pub fn fold_const(e: &Expr, consts: &std::collections::HashMap<String, u64>) -> Option<u64> {
    match e {
        Expr::Num(v, _) => Some(*v),
        Expr::Ident(n) => consts.get(n.as_str()).copied(),
        Expr::Bin(op, a, b) => {
            let (x, y) = (fold_const(a, consts)?, fold_const(b, consts)?);
            Some(match op.as_str() {
                "+" => x.wrapping_add(y),
                "-" => x.wrapping_sub(y),
                "*" => x.wrapping_mul(y),
                "/" => if y != 0 { x / y } else { 0 },
                "%" => if y != 0 { x % y } else { 0 },
                "&" => x & y,
                "|" => x | y,
                "^" => x ^ y,
                "<<" => x.wrapping_shl(y as u32),
                ">>" => x.wrapping_shr(y as u32),
                "==" => (x == y) as u64,
                "!=" => (x != y) as u64,
                "<" => (x < y) as u64,
                "<=" => (x <= y) as u64,
                ">" => (x > y) as u64,
                ">=" => (x >= y) as u64,
                _ => return None,
            })
        }
        Expr::Un(op, a) => {
            let x = fold_const(a, consts)?;
            Some(match op.as_str() {
                "!" => (x == 0) as u64,
                "~" => !x,
                "-" => x.wrapping_neg(),
                _ => x,
            })
        }
        _ => None,
    }
}
