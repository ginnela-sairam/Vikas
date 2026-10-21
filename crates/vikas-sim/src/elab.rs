//! VIKAS v1 — XEZIM elaborator: flattens module hierarchy into a flat
//! signal namespace with port bindings. Supports multi-file projects.

use super::parser::{ConnExpr, Expr, ModuleDef, Stmt, VarKind};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SigInfo {
    pub name: String,   // fully qualified: top.a.b.c
    pub width: usize,
    pub dims: Vec<(i32, i32)>,
    pub is_array: bool,
}

#[derive(Clone, Debug)]
pub struct PortBinding {
    pub child_sig: String,      // child port signal name (qualified)
    pub parent_expr: BoundExpr, // what it's connected to in parent scope
    pub dir: u8,                // 1 in, 2 out, 3 inout
}

#[derive(Clone, Debug)]
pub enum BoundExpr {
    Sig(String),
    Slice(String, i32, i32),
    Num(u64, usize),
    Concat(Vec<BoundExpr>),
    /// arbitrary expression connection (input ports only) — evaluated in the
    /// PARENT scope each delta iteration (kernel enhancement #21)
    Expr(Box<Expr>, String),
    Open,
}

pub struct Design {
    pub top: String,
    pub sigs: HashMap<String, SigInfo>,
    pub port_bindings: Vec<PortBinding>, // ordered for deterministic sync
    pub decl_order: Vec<String>,
    /// unpacked arrays: "dut.mem" -> (cell_width, lo, hi) — cells stored as "dut.mem__<i>"
    pub arrays: HashMap<String, (usize, i32, i32)>,
    /// resolved parameter values: "dut.W" -> numeric value
    pub param_values: HashMap<String, u64>,
}

pub fn elaborate(mods: &[ModuleDef], top: &str, global_consts: &std::collections::HashMap<String, u64>) -> Result<Design> {
    let by_name: HashMap<&str, &ModuleDef> = mods.iter().map(|m| (m.name.as_str(), m)).collect();
    let topm = *by_name
        .get(top)
        .ok_or_else(|| anyhow::anyhow!("top module '{}' not found (modules: {})", top, mods.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")))?;
    let mut d = Design {
        top: top.to_string(),
        sigs: HashMap::new(),
        port_bindings: Vec::new(),
        decl_order: Vec::new(),
        arrays: HashMap::new(),
        param_values: HashMap::new(),
    };
    elab_module(topm, top, &by_name, &mut d, &mut HashMap::new(), global_consts)?;
    Ok(d)
}

fn norm_dims(width: usize, dims: &[(i32, i32)]) -> (usize, Vec<(i32, i32)>, bool) {
    // first range is the vector range, subsequent are array dims
    if dims.is_empty() {
        (width, vec![], false)
    } else {
        let vec_w = ((dims[0].0 as i64 - dims[0].1 as i64 + 1).max(1)) as usize;
        let arr_dims: Vec<(i32, i32)> = dims[1..].to_vec();
        let is_arr = !arr_dims.is_empty();
        (vec_w.max(1), arr_dims, is_arr)
    }
}

#[allow(clippy::too_many_arguments)]
fn elab_module(
    m: &ModuleDef,
    prefix: &str,
    by_name: &HashMap<&str, &ModuleDef>,
    d: &mut Design,
    visited: &mut HashMap<String, String>,
    global_consts: &std::collections::HashMap<String, u64>,
) -> Result<()> {
    // ports
    let mut port_map: HashMap<String, (usize, u8)> = HashMap::new();
    for p in &m.ports {
        let fq = format!("{}.{}", prefix, p.name);
        if !d.sigs.contains_key(&fq) {
            d.sigs.insert(fq.clone(), SigInfo { name: fq.clone(), width: p.width, dims: vec![], is_array: false });
            d.decl_order.push(fq);
        }
        port_map.insert(p.name.clone(), (p.width, p.dir));
    }
    // resolve module parameters for this instance scope (numeric defaults).
    // File-scope constants (`include`d headers) are visible in every scope.
    {
        let mut pmap: HashMap<String, u64> = global_consts.clone();
        for (pn, pv) in &m.params {
            if let Some(e) = pv {
                if let Some(v) = const_fold(e, &pmap) {
                    pmap.insert(pn.clone(), v);
                }
            }
        }
        for (k, v) in &pmap {
            let fq = format!("{}.{}", prefix, k);
            d.param_values.insert(fq, *v);
        }
    }
    // internal signals
    for s in &m.signals {
        if let Stmt::Decl { name, width, dims, kind, init } = s {
            let _ = kind;
            let _ = init;
            let (w, arr_dims, is_arr) = norm_dims(*width, dims);
            let fq = format!("{}.{}", prefix, name);
            if is_arr {
                // unpacked array: register + expand cells as separate signals
                let (lo, hi) = arr_dims[0];
                let (lo, hi) = (lo.min(hi), lo.max(hi));
                d.arrays.insert(fq.clone(), (w.max(1), lo, hi));
                for ci in lo..=hi {
                    let cell = format!("{}__{}", fq, ci);
                    if !d.sigs.contains_key(&cell) {
                        d.sigs.insert(cell.clone(), SigInfo { name: cell.clone(), width: w.max(1), dims: vec![], is_array: false });
                        d.decl_order.push(cell);
                    }
                }
                continue;
            }
            if !d.sigs.contains_key(&fq) {
                d.sigs.insert(fq.clone(), SigInfo { name: fq.clone(), width: w, dims: arr_dims, is_array: is_arr });
                d.decl_order.push(fq);
            }
        }
    }
    // genvars/functions don't create hardware signals; functions execute inline
    // instances
    for inst in &m.instances {
        let child = match by_name.get(inst.module.as_str()) {
            Some(c) => *c,
            None => {
                // unresolved module — treat as blackbox, create stub ports
                continue;
            }
        };
        let iprefix = format!("{}.{}", prefix, inst.inst_name);
        visited.insert(inst.inst_name.clone(), iprefix.clone());
        // wildcard .* : auto-connect same-name ports
        let mut conns: Vec<(String, ConnExpr)> = Vec::new();
        let mut wildcard = false;
        for (pn, ce) in &inst.conns {
            if pn == "*" {
                wildcard = true;
                continue;
            }
            conns.push((pn.clone(), ce.clone()));
        }
        if wildcard {
            for p in &child.ports {
                if !conns.iter().any(|(n, _)| n == &p.name) {
                    // connect to same-name signal in parent if exists
                    let fq = format!("{}.{}", prefix, p.name);
                    if d.sigs.contains_key(&fq) {
                        conns.push((p.name.clone(), ConnExpr::Sig(p.name.clone(), -1, -1)));
                    }
                }
            }
        }
        // create child port signals + bindings
        for p in &child.ports {
            let fq = format!("{}.{}", iprefix, p.name);
            if !d.sigs.contains_key(&fq) {
                d.sigs.insert(fq.clone(), SigInfo { name: fq.clone(), width: p.width, dims: vec![], is_array: false });
                d.decl_order.push(fq.clone());
            }
            let conn = conns.iter().find(|(n, _)| n == &p.name).map(|(_, c)| c.clone());
            // kernel #21: expression connections are INPUT-only (an output
            // bound to a non-lvalue is illegal RTL — skip the binding, keep
            // the child signal floating X with a lint-style tolerance)
            if let Some(ConnExpr::Expr(_)) = &conn {
                if p.dir & 2 != 0 {
                    continue;
                }
            }
            let parent_expr = match conn {
                Some(ConnExpr::Sig(n, msb, lsb)) => {
                    let pfx = if n.contains('.') { n.to_string() } else { format!("{}.{}", prefix, n) };
                    if msb >= 0 {
                        BoundExpr::Slice(pfx, msb, lsb)
                    } else {
                        BoundExpr::Sig(pfx)
                    }
                }
                Some(ConnExpr::Num(v, w)) => BoundExpr::Num(v, w),
                Some(ConnExpr::Expr(e)) => BoundExpr::Expr(e, prefix.to_string()),
                Some(ConnExpr::Concat(items)) => BoundExpr::Concat(items
                    .iter()
                    .map(|ci| match ci {
                        ConnExpr::Sig(n, msb, lsb) => {
                            let pfx = if n.contains('.') { n.to_string() } else { format!("{}.{}", prefix, n) };
                            if *msb >= 0 { BoundExpr::Slice(pfx, *msb, *lsb) } else { BoundExpr::Sig(pfx) }
                        }
                        ConnExpr::Num(v, w) => BoundExpr::Num(*v, *w),
                        ConnExpr::Expr(e) => BoundExpr::Expr(e.clone(), prefix.to_string()),
                        _ => BoundExpr::Open,
                    })
                    .collect()),
                _ => BoundExpr::Open,
            };
            d.port_bindings.push(PortBinding {
                child_sig: fq.clone(),
                parent_expr,
                dir: p.dir,
            });
        }
        elab_module(child, &iprefix, by_name, d, visited, global_consts)?;
    }
    Ok(())
}

/// Pre-eval constant for parameter substitution (simple numeric fold).
pub fn const_fold(e: &Expr, params: &HashMap<String, u64>) -> Option<u64> {
    match e {
        Expr::Num(v, _) => Some(*v),
        Expr::Ident(n) => params.get(n).copied(),
        Expr::Bin(op, a, b) => {
            let (x, y) = (const_fold(a, params)?, const_fold(b, params)?);
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
            let x = const_fold(a, params)?;
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
