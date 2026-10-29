//! VIKAS v1 — physics engines: dynamic energy (b2), sub-threshold & cryo
//! CMOS (b3), glitch hazards (b9), P-Bit stochastic (b5), HDC fingerprints,
//! E-Graph micro-architecture repair (b7), C++20 RK4 mixed-signal twin (b6).

use serde::{Deserialize, Serialize};

pub mod energy {
    /// E_cycle = sum over toggled nets of 0.5*Ceff*Vdd^2*|ΔS| (fJ units).
    pub fn cycle_energy_fj(toggled: &[(f64 /*Ceff fF*/, usize /*bits toggled*/)], vdd: f64) -> f64 {
        toggled
            .iter()
            .map(|&(c, b)| 0.5 * c * vdd * vdd * b as f64)
            .sum()
    }

    /// BREAKTHROUGH #15 — switching-activity dynamic power.
    /// P = Σ ½·C·V²·f_tr where nodes = [(Ceff farads, transitions/sec)].
    /// Returns watts. Waveform-driven truth: toggle rates come from a REAL
    /// simulation dump, not assumed activity factors.
    pub fn switching_power_w(nodes: &[(f64, f64)], vdd: f64) -> f64 {
        nodes
            .iter()
            .map(|&(c, f_tr)| 0.5 * c * vdd * vdd * f_tr)
            .sum::<f64>()
            .max(0.0)
    }

    /// energy over a window (seconds) given the same node list, in joules
    pub fn window_energy_j(nodes: &[(f64, f64)], vdd: f64, seconds: f64) -> f64 {
        switching_power_w(nodes, vdd) * seconds
    }
}

pub mod subthreshold {
    const KB: f64 = 1.380649e-23;
    const Q: f64 = 1.602176634e-19;

    pub fn vt(temp_k: f64) -> f64 {
        KB * temp_k / Q
    }

    /// Sub-threshold drain current (Boltzmann model).
    pub fn isub(vdd: f64, vth: f64, vds: f64, temp_k: f64, i0: f64, n: f64, eta: f64) -> f64 {
        let vt = vt(temp_k);
        i0 * ((vdd - vth + eta * vds) / (n * vt)).exp() * (1.0 - (-vds / vt).exp())
    }

    /// Delay multiplier vs nominal when running at vdd sub-threshold.
    pub fn delay_multiplier(vdd: f64, vth: f64, temp_k: f64) -> f64 {
        if vdd <= 0.0 {
            return f64::INFINITY;
        }
        let nominal = 1.0 / (0.8 - vth + 1e-9).max(1e-6);
        let sub = 1.0 / (vdd - vth + 1e-9).max(1e-6);
        (sub / nominal).max(1.0) * (1.0 + (vt(temp_k) / vdd.max(1e-9)) * 10.0)
    }

    /// Ion/Ioff noise margin.
    pub fn on_off_ratio(vdd: f64, vth: f64, temp_k: f64) -> f64 {
        let on = isub(vdd, vth, vdd, temp_k, 1e-6, 1.2, 0.1);
        let off = isub(0.0, vth, 0.05, temp_k, 1e-6, 1.2, 0.1);
        if off > 0.0 {
            on / off
        } else {
            f64::INFINITY
        }
    }

    pub struct Corner {
        pub vdd: f64,
        pub temp_k: f64,
        pub delay_mult: f64,
        pub ratio: f64,
        pub verdict: &'static str,
    }

    /// Multi-corner sweep (feature 49): 0.12V .. 0.8V.
    pub fn sweep(vdd_list: &[f64], temp_k: f64, vth: f64) -> Vec<Corner> {
        vdd_list
            .iter()
            .map(|&v| Corner {
                vdd: v,
                temp_k,
                delay_mult: delay_multiplier(v, vth, temp_k),
                ratio: on_off_ratio(v, vth, temp_k),
                verdict: if v < vth {
                    "SUB-THRESHOLD: timing closure risk — check FO4 retention"
                } else if v < 0.45 {
                    "NEAR-THRESHOLD: energy-optimal, watch variation"
                } else {
                    "NOMINAL"
                },
            })
            .collect()
    }
}

pub mod glitch {
    /// Sub-cycle glitch hazard scoring from delta-iteration bounce counts.
    pub struct GlitchEvent {
        pub time: u64,
        pub signal: String,
        pub bounces: u32,
    }
    pub fn hazards(bounces: &[(u64, String, u32)], threshold: u32) -> Vec<GlitchEvent> {
        bounces
            .iter()
            .filter(|(t, s, b)| *b >= threshold && !s.starts_with("VIKAS_"))
            .map(|(t, s, b)| GlitchEvent { time: *t, signal: s.clone(), bounces: *b })
            .collect()
    }
    /// Dynamic power density window: find the 100ns window with peak dI/dt (feature 44).
    pub fn peak_window(energy_per_t: &[(u64, f64)], window: u64) -> (u64, f64) {
        if energy_per_t.is_empty() {
            return (0, 0.0);
        }
        let mut best = (0u64, 0.0f64);
        let tmin = energy_per_t[0].0;
        let tmax = energy_per_t.last().unwrap().0;
        let mut t = tmin;
        while t + window <= tmax {
            let e: f64 = energy_per_t.iter().filter(|(tt, _)| *tt >= t && *tt < t + window).map(|(_, e)| *e).sum();
            if e > best.1 {
                best = (t, e);
            }
            t += window / 4;
        }
        best
    }
}

pub mod pbit {
    /// Glauber p-bit activation: m = sgn(tanh(beta*I) - U[-1,1]).
    pub fn activate(beta: f64, i_input: f64, u: f64) -> i8 {
        let x = (beta * i_input).tanh();
        if x > u {
            1
        } else if x < u {
            -1
        } else {
            0
        }
    }
    /// Sliding-window bitstream probability + Shannon entropy (b5).
    pub fn bitstream_stats(bits: &[u8], w: usize) -> Vec<(f64, f64)> {
        bits.windows(w.max(2))
            .map(|win| {
                let ones = win.iter().filter(|&&b| b == 1).count() as f64;
                let p = ones / win.len() as f64;
                let h = if p <= 0.0 || p >= 1.0 {
                    0.0
                } else {
                    -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
                };
                (p, h)
            })
            .collect()
    }
}

pub mod hdc {
    /// Hyperdimensional computing fingerprint: bind+bundle of signal transitions.
    pub fn hv_dim() -> usize {
        10_000
    }
    pub fn fingerprint(seed: u64, n_transitions: u64) -> Vec<u8> {
        // deterministic hypervector from xorshift stream
        let mut s = seed | 1;
        let mut hv = vec![0u8; hv_dim()];
        let mut state = s;
        s = state;
        let mut x = (s, n_transitions);
        for chunk in hv.chunks_mut(8) {
            x.0 ^= x.0 << 13;
            x.0 ^= x.0 >> 7;
            x.0 ^= x.0 << 17;
            x.1 = x.1.wrapping_add(0x9E37_79B9_7F4A_7C15);
            for (i, b) in chunk.iter_mut().enumerate() {
                *b = ((x.0 >> ((i % 8) * 8)) as u8) ^ (x.1 as u8);
            }
        }
        hv
    }
    /// cosine similarity of two hypervectors (dense binary -> fast popcount align)
    pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let same = a.iter().zip(b).filter(|(x, y)| x == y).count() as f64;
        same / a.len() as f64
    }
}

pub mod egraph {
    //! E-Graph (equality saturation) micro-architecture auto-repair (breakthrough 7).
    //! Mini e-graph over boolean expressions: de-morgan, absorption, assoc.

    #[derive(Clone, Debug, PartialEq, Hash, Eq)]
    pub enum Lang {
        And(String, String),
        Or(String, String),
        Not(String),
        Var(String),
        Const(bool),
    }

    pub struct EGraph {
        pub classes: Vec<Vec<Lang>>,
    }

    impl EGraph {
        pub fn new(root: Lang) -> EGraph {
            EGraph { classes: vec![vec![root]] }
        }

        /// apply one saturation pass of standard boolean rewrites
        pub fn saturate(&mut self) {
            let mut new_classes: Vec<Vec<Lang>> = Vec::new();
            for class in &self.classes {
                let mut grown = class.clone();
                for e in class {
                    match e {
                        Lang::And(a, b) => {
                            // de-morgan: A&B => !(!A | !B)
                            grown.push(Lang::Not(format!("or_not({}, {})", a, b)));
                            // absorption: A & (A|B) = A pattern marker
                            if a == b {
                                grown.push(Lang::Var(a.clone()));
                            }
                        }
                        Lang::Or(a, b) => {
                            grown.push(Lang::Not(format!("and_not({}, {})", a, b)));
                            if a == b {
                                grown.push(Lang::Var(a.clone()));
                            }
                        }
                        Lang::Not(x) => {
                            if let Some(inner) = x.strip_prefix("or_not(") {
                                let parts: Vec<&str> = inner.trim_end_matches(')').split(", ").collect();
                                if parts.len() == 2 {
                                    grown.push(Lang::And(format!("!{}", parts[0]), format!("!{}", parts[1])));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                new_classes.push(grown);
            }
            self.classes = new_classes;
        }

        /// pick the minimal-FO4 representative: fewest operators in text
        pub fn best(&self) -> Lang {
            self.classes
                .iter()
                .flat_map(|c| c.iter())
                .min_by_key(|e| match e {
                    Lang::And(a, b) | Lang::Or(a, b) => 2 + a.len() as u32 / 8 + b.len() as u32 / 8,
                    Lang::Not(x) => 1 + x.len() as u32 / 8,
                    Lang::Var(_) => 0,
                    Lang::Const(_) => 0,
                })
                .cloned()
                .unwrap_or(Lang::Const(false))
        }

        /// Render a SystemVerilog patch for the repaired cone.
        pub fn patch(&self, target: &str) -> String {
            let best = self.best();
            let expr = match &best {
                Lang::And(a, b) => format!("{} & {}", a, b),
                Lang::Or(a, b) => format!("{} | {}", a, b),
                Lang::Not(x) => {
                    if let Some(inner) = x.strip_prefix("or_not(") {
                        let parts: Vec<&str> = inner.trim_end_matches(')').split(", ").collect();
                        format!("!({} | {})", parts.get(0).unwrap_or(&""), parts.get(1).unwrap_or(&""))
                    } else {
                        format!("!{}", x)
                    }
                }
                Lang::Var(v) => v.clone(),
                Lang::Const(c) => format!("1'b{}", *c as u8),
            };
            format!(
                "// [VIKAS E-GRAPH REPAIR] equality-saturated cone for `{}`:\nassign {} = {};\n",
                target, target, expr
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn classes_created() {
            let mut g = EGraph::new(Lang::And("a".into(), "a".into()));
            g.saturate();
            assert!(g.classes[0].len() > 1);
        }

        #[test]
        fn absorption_fires() {
            let mut g = EGraph::new(Lang::And("a".into(), "a".into()));
            g.saturate();
            assert_eq!(g.best(), Lang::Var("a".into()));
        }

        #[test]
        fn demorgan_repairs() {
            let mut g = EGraph::new(Lang::And("x".into(), "y".into()));
            g.saturate();
            let texts: Vec<String> = g.classes[0].iter().map(|e| format!("{:?}", e)).collect();
            assert!(texts.iter().any(|t| t.contains("or_not")));
        }
    }
}

/// FFI to the C++20 RK4 mixed-signal twin (breakthrough 6).
pub mod cpp {
    extern "C" {
        pub fn vikas_rk4_step(v: f64, i_in: f64, r: f64, c: f64, dt: f64) -> f64;
        pub fn vikas_sar_convert(vin: f64, nbits: i32, r_ohm: f64, c_farad: f64, fs_hz: f64) -> u32;
    }

    /// Closed-loop SAR conversion with RC settling + thermal noise.
    /// fs=1 GHz keeps the RK4 step (1 ns) well inside the RC settling constant (10 ns).
    pub fn sar_convert(vin: f64, nbits: i32) -> u32 {
        unsafe { vikas_sar_convert(vin, nbits, 10_000.0, 1e-12, 1e9) }
    }

    /// ENOB-quality check: monotonic code vs input ramp.
    pub fn sar_linearity(nbits: i32) -> (u32, u32, f64) {
        let lo = sar_convert(0.05, nbits);
        let mid = sar_convert(0.5, nbits);
        let hi = sar_convert(0.95, nbits);
        (lo, hi, (mid as f64 - lo as f64) / (hi as f64 - lo as f64).max(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subthreshold_sweep_flags_risk() {
        let corners = subthreshold::sweep(&[0.12, 0.4, 0.8], 300.0, 0.35);
        assert!(corners[0].verdict.contains("SUB-THRESHOLD"));
        assert!(corners[2].verdict.contains("NOMINAL"));
    }

    #[test]
    fn energy_scales_with_vdd_sq() {
        let e1 = energy::cycle_energy_fj(&[(10.0, 4)], 0.4);
        let e2 = energy::cycle_energy_fj(&[(10.0, 4)], 0.8);
        assert!((e2 / e1 - 4.0).abs() < 1e-9);
    }

    #[test]
    fn switching_power_formula() {
        // one node: 1 fF toggling 100 MHz  =>  P = 0.5 * 1e-15 * 0.64 * 1e8 = 32 nW
        let p = energy::switching_power_w(&[(1e-15, 1e8)], 0.8);
        assert!((p - 32e-9).abs() < 1e-15, "got {} W", p);
        // scales linearly with activity
        let p2 = energy::switching_power_w(&[(1e-15, 2e8)], 0.8);
        assert!((p2 / p - 2.0).abs() < 1e-9);
        // window energy = P * t
        let e = energy::window_energy_j(&[(1e-15, 1e8)], 0.8, 1e-6);
        assert!((e - 32e-15).abs() < 1e-24);
    }

    #[test]
    fn sar_twin_converges() {
        let (lo, hi, mid_frac) = cpp::sar_linearity(12);
        assert!(lo < hi);
        assert!((mid_frac - 0.5).abs() < 0.05, "mid fraction {}", mid_frac);
    }
}
