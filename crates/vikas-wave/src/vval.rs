//! VIKAS v1 — 4-state logic value model (0/1/X/Z), arbitrary width.

use serde::{Deserialize, Serialize};
use std::fmt;

/// One logic value: 0, 1, X (unknown), Z (high-impedance).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum Bit {
    Zero,
    One,
    X,
    Z,
}

impl Bit {
    pub fn from_char(c: char) -> Bit {
        match c {
            '0' => Bit::Zero,
            '1' => Bit::One,
            'z' | 'Z' => Bit::Z,
            _ => Bit::X,
        }
    }
    pub fn as_char(&self) -> char {
        match self {
            Bit::Zero => '0',
            Bit::One => '1',
            Bit::X => 'x',
            Bit::Z => 'z',
        }
    }
    pub fn to_u32(&self) -> u32 {
        match self {
            Bit::One => 1,
            Bit::Zero => 0,
            _ => 0, // X/Z read as 0, 4-state semantics recorded separately
        }
    }
    pub fn is_defined(&self) -> bool {
        matches!(self, Bit::Zero | Bit::One)
    }
}

impl std::ops::Not for Bit {
    type Output = Bit;
    fn not(self) -> Bit {
        match self {
            Bit::Zero => Bit::One,
            Bit::One => Bit::Zero,
            Bit::Z => Bit::X,
            Bit::X => Bit::X,
        }
    }
}

/// Multi-bit vector of 4-state logic.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct VVal {
    pub bits: Vec<Bit>, // MSB-first
}

impl VVal {
    pub fn zero(width: usize) -> VVal {
        VVal { bits: vec![Bit::Zero; width] }
    }
    pub fn one(width: usize) -> VVal {
        let mut b = vec![Bit::Zero; width];
        if width > 0 {
            b[width - 1] = Bit::One;
        }
        VVal { bits: b }
    }
    pub fn x(width: usize) -> VVal {
        VVal { bits: vec![Bit::X; width] }
    }
    pub fn from_u64(v: u64, width: usize) -> VVal {
        let mut bits = Vec::with_capacity(width);
        for i in (0..width).rev() {
            bits.push(if (v >> i) & 1 == 1 { Bit::One } else { Bit::Zero });
        }
        VVal { bits }
    }
    pub fn from_bits_str(s: &str) -> VVal {
        // accepts "0101x" (MSB first) or "b0101" VCD form
        let s = s.strip_prefix('b').unwrap_or(s);
        VVal { bits: s.chars().map(Bit::from_char).collect() }
    }
    pub fn width(&self) -> usize {
        self.bits.len()
    }
    pub fn to_u64(&self) -> u64 {
        let mut v = 0u64;
        for b in &self.bits {
            v = (v << 1) | b.to_u32() as u64;
        }
        v
    }
    pub fn is_x(&self) -> bool {
        self.bits.iter().any(|b| !b.is_defined())
    }
    pub fn to_bin_str(&self) -> String {
        self.bits.iter().map(|b| b.as_char()).collect()
    }
    pub fn to_hex_str(&self) -> String {
        if self.is_x() {
            return self.to_bin_str();
        }
        let mut out = String::new();
        let n = self.bits.len();
        let pad = (4 - (n % 4)) % 4;
        let mut padded = vec![Bit::Zero; pad];
        padded.extend_from_slice(&self.bits);
        for chunk in padded.chunks(4) {
            let mut d = 0u8;
            for b in chunk {
                d = (d << 1) | b.to_u32() as u8;
            }
            out.push(char::from_digit(d as u32, 16).unwrap());
        }
        if out.is_empty() {
            out.push('0');
        }
        out
    }
    /// VCD-style string: 1-bit => "0"/"1", n-bit => "b0101"
    pub fn vcd_str(&self) -> String {
        if self.bits.len() == 1 {
            self.bits[0].as_char().to_string()
        } else {
            format!("b{}", self.to_bin_str())
        }
    }
    pub fn bit(&self, i: usize) -> Option<Bit> {
        // i is counted FROM THE LSB (Verilog semantics); bits[] is MSB-first
        if i < self.bits.len() {
            Some(self.bits[self.bits.len() - 1 - i])
        } else {
            None
        }
    }
    /// two's complement signed value (defined bits only)
    pub fn to_i64(&self) -> i64 {
        let n = self.bits.len();
        if n == 0 {
            return 0;
        }
        let u = self.to_u64();
        if n >= 64 {
            return u as i64;
        }
        let sign = 1u64 << (n - 1);
        if u & sign != 0 {
            (u.wrapping_sub(1u64 << n)) as i64
        } else {
            u as i64
        }
    }
}

impl fmt::Display for VVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}h{}", self.width(), self.to_hex_str())
    }
}

/// Signal category for GUI colouring & analytics.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SigKind {
    Clock,
    Reset,
    Rtl,
    Physics,
    Causality,
    Stochastic,
    Taint,
    LatchHold,
}

impl SigKind {
    pub fn classify(name: &str) -> SigKind {
        let n = name.rsplit('.').next().unwrap_or(name).to_lowercase();
        if n.starts_with("vikas_physics") {
            SigKind::Physics
        } else if n.starts_with("vikas_causality") {
            SigKind::Causality
        } else if n.starts_with("vikas_stochastic") || n.starts_with("vikas_pbit") {
            SigKind::Stochastic
        } else if n.starts_with("vikas_taint") {
            SigKind::Taint
        } else if n.starts_with("vikas_latch") || n.contains("latch_hold") {
            SigKind::LatchHold
        } else if n == "clk" || n.contains("clock") || n.starts_with("clk_") {
            SigKind::Clock
        } else if n.starts_with("rst") || n.contains("reset") {
            SigKind::Reset
        } else {
            SigKind::Rtl
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            SigKind::Clock => "clock",
            SigKind::Reset => "reset",
            SigKind::Rtl => "rtl",
            SigKind::Physics => "physics",
            SigKind::Causality => "causality",
            SigKind::Stochastic => "stochastic",
            SigKind::Taint => "taint",
            SigKind::LatchHold => "latch",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Signal {
    pub name: String,
    pub width: usize,
    pub kind: SigKind,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Change {
    pub time: u64,
    pub sig: usize,
    pub value: VVal,
}

/// In-memory waveform database.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaveDump {
    pub signals: Vec<Signal>,
    pub changes: Vec<Change>,
}

impl WaveDump {
    pub fn new() -> WaveDump {
        WaveDump::default()
    }
    pub fn add_signal(&mut self, name: &str, width: usize) -> usize {
        if let Some(i) = self.signals.iter().position(|s| s.name == name && s.width == width) {
            return i;
        }
        self.signals.push(Signal { name: name.to_string(), width, kind: SigKind::classify(name) });
        self.signals.len() - 1
    }
    pub fn push(&mut self, time: u64, sig: usize, value: VVal) {
        self.changes.push(Change { time, sig, value });
    }
    pub fn end_time(&self) -> u64 {
        self.changes.last().map(|c| c.time).unwrap_or(0)
    }
    /// per-signal step lists for O(log n) point queries
    pub fn steps(&self) -> Vec<Vec<(u64, VVal)>> {
        let mut steps: Vec<Vec<(u64, VVal)>> = vec![Vec::new(); self.signals.len()];
        for c in &self.changes {
            if c.sig < steps.len() {
                steps[c.sig].push((c.time, c.value.clone()));
            }
        }
        steps
    }
    pub fn value_at(&self, sig: usize, t: u64) -> Option<VVal> {
        let mut r = None;
        for c in &self.changes {
            if c.sig == sig && c.time <= t {
                r = Some(c.value.clone());
            } else if c.sig == sig && c.time > t {
                break;
            }
        }
        r
    }

    // ---------- VCD ----------
    pub fn write_vcd(&self, path: &std::path::Path) -> anyhow::Result<()> {
        use std::io::Write;
        let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
        write!(w, "$date {} $end\n", ts)?;
        write!(w, "$version VIKAS v1 XEZIM kernel $end\n")?;
        write!(w, "$timescale 1ns $end\n")?;
        let mut ids: Vec<String> = Vec::new();
        for (i, s) in self.signals.iter().enumerate() {
            let mut id = String::new();
            let mut x = i;
            loop {
                id.push((b'!' + (x % 94) as u8) as char);
                x /= 94;
                if x == 0 {
                    break;
                }
            }
            ids.push(id.clone());
            write!(w, "$var wire {} {} {} $end\n", s.width, id, s.name)?;
        }
        write!(w, "$enddefinitions $end\n$dumpvars\n")?;
        // initial values then deltas; group by time
        let mut cur: Vec<Option<&VVal>> = vec![None; self.signals.len()];
        let mut last_t = 0u64;
        let mut first = true;
        for c in &self.changes {
            if c.time != last_t || first {
                if !first {
                    write!(w, "#{}\n", c.time)?;
                } else {
                    write!(w, "#{}\n", c.time)?;
                    first = false;
                }
                last_t = c.time;
            }
            cur[c.sig] = Some(&c.value);
            let v = c.value.vcd_str();
            if c.value.width() == 1 {
                writeln!(w, "{}{}", &v, ids[c.sig])?;
            } else {
                writeln!(w, "{} {}", v, ids[c.sig])?;
            }
        }
        write!(w, "#{}\n", self.end_time() + 1)?;
        // final full dump state
        for (i, s) in self.signals.iter().enumerate() {
            let v = cur[i].cloned().unwrap_or_else(|| VVal::x(s.width));
            let vs = v.vcd_str();
            if s.width == 1 {
                writeln!(w, "{}{}", vs, ids[i])?;
            } else {
                writeln!(w, "{} {}", vs, ids[i])?;
            }
        }
        w.flush()?;
        Ok(())
    }

    pub fn load_vcd(path: &std::path::Path) -> anyhow::Result<WaveDump> {
        let text = std::fs::read_to_string(path)?;
        let mut dump = WaveDump::new();
        let mut id2idx: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut t = 0u64;
        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            if let Some(rest) = l.strip_prefix("$var") {
                let toks: Vec<&str> = rest.split_whitespace().collect();
                if toks.len() >= 4 {
                    let width: usize = toks[0].parse().unwrap_or(1);
                    let idx = dump.add_signal(toks[3], width);
                    id2idx.insert(toks[2].to_string(), idx);
                }
            } else if let Some(rest) = l.strip_prefix('#') {
                t = rest.trim().parse().unwrap_or(0);
            } else if l.starts_with("b") || l.starts_with("B") {
                let toks: Vec<&str> = l.split_whitespace().collect();
                if toks.len() == 2 {
                    if let Some(&si) = id2idx.get(toks[1]) {
                        let w = dump.signals[si].width;
                        dump.push(t, si, VVal::from_bits_str(&toks[0][1..]));
                        let _ = w;
                    }
                }
            } else if l == "$end" || l.starts_with('$') || l.starts_with("$dumpvars") {
                continue;
            } else {
                // 1-bit short form: "<val><id>"
                let (val, _id) = l.split_at(l.len().saturating_sub(1).max(l.len() - 1));
                if l.len() >= 2 {
                    let (val, id) = (&l[..1], &l[1..]);
                    if "01xzXZ".contains(val) {
                        if let Some(&si) = id2idx.get(id) {
                            dump.push(t, si, VVal::from_bits_str(val));
                        }
                    }
                }
                let _ = val;
            }
        }
        Ok(dump)
    }

    // ---------- generic load by extension ----------
    pub fn load(path: &std::path::Path) -> anyhow::Result<WaveDump> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "vcd" => Self::load_vcd(path),
            "vwf" => crate::natformats::read_vwf(path),
            "vwt" => crate::natformats::read_vwt(path),
            "vwe" => crate::natformats::read_vwe(path),
            "vwp" => crate::natformats::read_vwp(path),
            "vwi" => crate::natformats::read_vwi(path),
            "vwm" => crate::natformats::read_vwm(path),
            other => anyhow::bail!("unsupported waveform format '.{}' (supported: .vcd .vwf .vwt .vwe .vwp .vwi .vwm)", other),
        }
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "vcd" => self.write_vcd(path),
            "vwf" => crate::natformats::write_vwf(self, path),
            "vwt" => crate::natformats::write_vwt(self, path),
            "vwe" => crate::natformats::write_vwe(self, path),
            "vwp" => crate::natformats::write_vwp(self, path),
            "vwi" => crate::natformats::write_vwi(self, path),
            "vwm" => crate::natformats::write_vwm(self, path),
            other => anyhow::bail!("unsupported write format '.{}'", other),
        }
    }
}

/// Toggle coverage (feature 19): per-signal 0->1 / 1->0 counts and dead nets.
pub struct ToggleReport {
    pub rows: Vec<(String, u64, u64, u64)>, // name, rising, falling, static%
}

pub fn toggle_coverage(d: &WaveDump) -> ToggleReport {
    let steps = d.steps();
    let mut rows = Vec::new();
    for (i, s) in d.signals.iter().enumerate() {
        let st = &steps[i];
        let (mut rise, mut fall) = (0u64, 0u64);
        if st.len() > 1 {
            for w in st.windows(2) {
                let (a, b) = (&w[0].1, &w[1].1);
                if a.width() == 1 && b.width() == 1 {
                    if a.to_u64() == 0 && b.to_u64() == 1 {
                        rise += 1;
                    } else if a.to_u64() == 1 && b.to_u64() == 0 {
                        fall += 1;
                    }
                }
            }
        }
        let toggles = rise + fall;
        let stat = if toggles == 0 { 100 } else { 0 };
        rows.push((s.name.clone(), rise, fall, stat));
    }
    ToggleReport { rows }
}

/// Parallel waveform differ (feature 25, rayon): earliest divergence.
pub fn diff_dumps(a: &WaveDump, b: &WaveDump) -> anyhow::Result<Option<(u64, String)>> {
    use rayon::prelude::*;
    let name2i_b: std::collections::HashMap<&str, usize> =
        b.signals.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();
    let results: Vec<(u64, String)> = a
        .signals
        .par_iter()
        .enumerate()
        .filter_map(|(ia, sa)| {
            let ib = *name2i_b.get(sa.name.as_str())?;
            let mut best: Option<(u64, String)> = None;
            let mut last_a = VVal::x(sa.width);
            let mut last_b = VVal::x(sa.width);
            let mut ti = 0u64;
            let mut ai = 0;
            let mut bi = 0;
            loop {
                let ta = a.changes.get(ai).filter(|c| c.sig == ia).map(|c| c.time);
                let tb = b.changes.get(bi).filter(|c| c.sig == ib).map(|c| c.time);
                let t_next = match (ta, tb) {
                    (Some(x), Some(y)) => x.min(y),
                    (Some(x), None) => x,
                    (None, Some(y)) => y,
                    (None, None) => break,
                };
                while ti <= t_next {
                    if ta == Some(ti) {
                        last_a = a.changes[ai].value.clone();
                        ai += 1;
                    }
                    if tb == Some(ti) {
                        last_b = b.changes[bi].value.clone();
                        bi += 1;
                    }
                    if last_a != last_b {
                        if best.is_none() || ti < best.as_ref().unwrap().0 {
                            best = Some((ti, sa.name.clone()));
                        }
                    }
                    if ti == u64::MAX {
                        break;
                    }
                    ti += 1;
                }
            }
            best
        })
        .collect();
    Ok(results.into_iter().min_by_key(|(t, _)| *t))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> WaveDump {
        let mut d = WaveDump::new();
        let clk = d.add_signal("tb.clk", 1);
        let bus = d.add_signal("tb.bus", 4);
        for t in (0..10).step_by(2) {
            d.push(t, clk, VVal::from_u64((t / 2) % 2, 1));
            d.push(t + 1, bus, VVal::from_u64(t, 4));
        }
        d
    }

    #[test]
    fn hex_str_is_msb_first() {
        assert_eq!(VVal::from_u64(0xF, 4).to_hex_str(), "f");
        assert_eq!(VVal::from_u64(0x4A, 8).to_hex_str(), "4a");
    }

    #[test]
    fn vwf_roundtrip() {
        let d = sample();
        let p = std::path::Path::new("/dev/shm/test_vwf.vwf");
        d.save(p).unwrap();
        let back = WaveDump::load(p).unwrap();
        assert_eq!(d.signals.len(), back.signals.len());
        assert_eq!(d.changes.len(), back.changes.len());
        for (a, b) in d.changes.iter().zip(&back.changes) {
            assert_eq!(a.time, b.time);
            assert_eq!(a.sig, b.sig);
            assert_eq!(a.value, b.value);
        }
    }
}
