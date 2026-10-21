//! # vikas-wave — waveform engine: 4-state values, VCD, 6 native formats,
//! parallel diff, SIMD temporal search, FFT ENOB, bus decode, ISA disasm.

pub mod natformats;
pub mod search;
pub mod vval;

pub use vval::{Bit, Change, Signal, SigKind, ToggleReport, VVal, WaveDump, diff_dumps, toggle_coverage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vcd_roundtrip_simple() {
        let mut d = WaveDump::new();
        let clk = d.add_signal("tb.clk", 1);
        let bus = d.add_signal("tb.bus", 8);
        d.push(0, clk, VVal::from_u64(0, 1));
        d.push(0, bus, VVal::from_u64(0xA5, 8));
        d.push(5, clk, VVal::from_u64(1, 1));
        d.push(10, clk, VVal::from_u64(0, 1));
        d.push(10, bus, VVal::from_u64(0x3C, 8));
        let p = std::path::Path::new("/dev/shm/test_rt.vcd");
        d.write_vcd(p).unwrap();
        let back = WaveDump::load_vcd(p).unwrap();
        assert_eq!(back.signals.len(), 2);
        assert_eq!(back.value_at(0, 5).unwrap().to_u64(), 1);
        assert_eq!(back.value_at(1, 10).unwrap().to_u64(), 0x3C);
    }

    #[test]
    fn diff_detects_divergence() {
        let mut a = WaveDump::new();
        let s = a.add_signal("x.y", 1);
        a.push(0, s, VVal::from_u64(0, 1));
        a.push(10, s, VVal::from_u64(1, 1));
        let mut b = a.clone();
        b.changes[1].value = VVal::from_u64(0, 1);
        b.push(12, s, VVal::from_u64(1, 1));
        let first = diff_dumps(&a, &b).unwrap();
        assert_eq!(first.map(|(t, _)| t), Some(10));
    }
}
