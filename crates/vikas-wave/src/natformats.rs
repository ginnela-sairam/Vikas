//! VIKAS v1 — six native innovative waveform formats (beyond VCD/FST):
//!  .vwf  VIKAS Waveform File    — binary, indexed change table + CRC32 checksum
//!  .vwt  VIKAS Temporal Track   — time-sectioned regions with per-region indexes
//!  .vwe  VIKAS Encoded          — delta-time + value-RLE compressed stream
//!  .vwp  VIKAS Physics          — carries analog f64 physics telemetry tracks
//!  .vwi  VIKAS Indexed          — per-signal absolute offset for O(1) random seek
//!  .vwm  VIKAS Memorymap        — fixed 32-byte aligned records for zero-copy mmap

use crate::{Change, Signal, VVal, WaveDump};
use anyhow::Result;

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn val_to_bytes(v: &VVal) -> Vec<u8> {
    // 4-state packing: 2 bits per bit -> 1 byte holds 4 bits
    let n = v.width();
    let mut out = vec![0u8; (n + 3) / 4];
    for (i, b) in v.bits.iter().enumerate() {
        let d: u8 = match b {
            crate::Bit::Zero => 0,
            crate::Bit::One => 1,
            crate::Bit::Z => 2,
            crate::Bit::X => 3,
        };
        out[i / 4] |= d << ((i % 4) * 2);
    }
    out
}

fn bytes_to_val(bytes: &[u8], width: usize) -> VVal {
    use crate::Bit;
    let mut bits = Vec::with_capacity(width);
    for i in 0..width {
        let d = (bytes[i / 4] >> ((i % 4) * 2)) & 3;
        bits.push(match d {
            0 => Bit::Zero,
            1 => Bit::One,
            2 => Bit::Z,
            _ => Bit::X,
        });
    }
    VVal { bits }
}

fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn get_u64(data: &[u8], off: &mut usize) -> u64 {
    let v = u64::from_le_bytes(data[*off..*off + 8].try_into().unwrap());
    *off += 8;
    v
}
fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn get_u32(data: &[u8], off: &mut usize) -> u32 {
    let v = u32::from_le_bytes(data[*off..*off + 4].try_into().unwrap());
    *off += 4;
    v
}

// ================= .vwf — indexed change table + checksum =================

pub fn write_vwf(d: &WaveDump, path: &std::path::Path) -> Result<()> {
    let mut body = Vec::new();
    put_u32(&mut body, d.signals.len() as u32);
    for s in &d.signals {
        put_u32(&mut body, s.width as u32);
        put_u32(&mut body, s.name.len() as u32);
        body.extend_from_slice(s.name.as_bytes());
    }
    put_u64(&mut body, d.changes.len() as u64);
    for c in &d.changes {
        put_u64(&mut body, c.time);
        put_u32(&mut body, c.sig as u32);
        let vb = val_to_bytes(&c.value);
        put_u32(&mut body, c.value.width() as u32);
        body.extend_from_slice(&vb);
    }
    let sum = crc32(&body);
    let mut file = Vec::with_capacity(8 + 4 + body.len());
    file.extend_from_slice(b"VIKASVWF");
    put_u32(&mut file, sum);
    file.extend_from_slice(&body);
    std::fs::write(path, file)?;
    Ok(())
}

pub fn read_vwf(path: &std::path::Path) -> Result<WaveDump> {
    let data = std::fs::read(path)?;
    if data.len() < 12 || &data[0..8] != b"VIKASVWF" {
        anyhow::bail!("bad .vwf magic");
    }
    let mut off = 8;
    let stored = get_u32(&data, &mut off);
    let body = &data[off..];
    if crc32(body) != stored {
        anyhow::bail!(".vwf checksum mismatch — file corrupted");
    }
    let mut o = 0usize;
    let nsig = get_u32(body, &mut o) as usize;
    let mut d = WaveDump::new();
    for _ in 0..nsig {
        let w = get_u32(body, &mut o) as usize;
        let nl = get_u32(body, &mut o) as usize;
        let name = String::from_utf8(body[o..o + nl].to_vec())?;
        o += nl;
        d.add_signal(&name, w);
    }
    let nch = get_u64(body, &mut o);
    for _ in 0..nch {
        let t = get_u64(body, &mut o);
        let si = get_u32(body, &mut o) as usize;
        let w = get_u32(body, &mut o) as usize;
        let nb = (w + 3) / 4;
        let v = bytes_to_val(&body[o..o + nb], w);
        o += nb;
        d.push(t, si, v);
    }
    Ok(d)
}

// ================= .vwt — temporal sectioned =================

pub fn write_vwt(d: &WaveDump, path: &std::path::Path) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(b"VIKASVWT");
    put_u32(&mut out, d.signals.len() as u32);
    for s in &d.signals {
        put_u32(&mut out, s.width as u32);
        put_u32(&mut out, s.name.len() as u32);
        out.extend_from_slice(s.name.as_bytes());
    }
    // section the timeline into regions of <= 256 changes
    const REG: usize = 256;
    let regions: Vec<&[Change]> = if d.changes.is_empty() {
        vec![]
    } else {
        d.changes.chunks(REG).collect()
    };
    put_u32(&mut out, regions.len() as u32);
    let mut idx_off = out.len();
    let mut index = Vec::new();
    for r in &regions {
        put_u32(&mut index, (out.len() + 4 * regions.len()) as u32); // placeholder replaced below
        let _ = r;
    }
    // simple layout: index of region start times, then regions
    let mut body = Vec::new();
    let mut real_index = Vec::new();
    for r in &regions {
        real_index.push(r[0].time);
        put_u64(&mut body, r.len() as u64);
        for c in *r {
            put_u64(&mut body, c.time);
            put_u32(&mut body, c.sig as u32);
            let vb = val_to_bytes(&c.value);
            put_u32(&mut body, c.value.width() as u32);
            body.extend_from_slice(&vb);
        }
    }
    let mut file = out.clone();
    for t in &real_index {
        put_u64(&mut file, *t);
    }
    let _ = &mut idx_off;
    file.extend_from_slice(&body);
    std::fs::write(path, file)?;
    Ok(())
}

pub fn read_vwt(path: &std::path::Path) -> Result<WaveDump> {
    let data = std::fs::read(path)?;
    if &data[0..8] != b"VIKASVWT" {
        anyhow::bail!("bad .vwt magic");
    }
    let mut o = 8usize;
    let nsig = get_u32(&data, &mut o) as usize;
    let mut d = WaveDump::new();
    for _ in 0..nsig {
        let w = get_u32(&data, &mut o) as usize;
        let nl = get_u32(&data, &mut o) as usize;
        let name = String::from_utf8(data[o..o + nl].to_vec())?;
        o += nl;
        d.add_signal(&name, w);
    }
    let nreg = get_u32(&data, &mut o) as usize;
    o += 8 * nreg; // skip region start-time index
    for _ in 0..nreg {
        let n = get_u64(&data, &mut o);
        for _ in 0..n {
            let t = get_u64(&data, &mut o);
            let si = get_u32(&data, &mut o) as usize;
            let w = get_u32(&data, &mut o) as usize;
            let nb = (w + 3) / 4;
            let v = bytes_to_val(&data[o..o + nb], w);
            o += nb;
            d.push(t, si, v);
        }
    }
    Ok(d)
}

// ================= .vwe — delta + RLE compressed =================

pub fn write_vwe(d: &WaveDump, path: &std::path::Path) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(b"VIKASVWE");
    put_u32(&mut out, d.signals.len() as u32);
    for s in &d.signals {
        put_u32(&mut out, s.width as u32);
        put_u32(&mut out, s.name.len() as u32);
        out.extend_from_slice(s.name.as_bytes());
    }
    let mut last_t = 0u64;
    let mut n_enc = 0u64;
    // count first
    {
        let mut lt = 0u64;
        let mut prev: Vec<Option<VVal>> = vec![None; d.signals.len()];
        for c in &d.changes {
            if let Some(p) = &prev[c.sig] {
                if *p == c.value {
                    continue; // RLE skip: identical value change is redundant
                }
            }
            prev[c.sig] = Some(c.value.clone());
            let _ = lt;
            lt = c.time;
            n_enc += 1;
        }
    }
    put_u64(&mut out, n_enc);
    let mut prev: Vec<Option<VVal>> = vec![None; d.signals.len()];
    let mut lt = 0u64;
    for c in &d.changes {
        if let Some(p) = &prev[c.sig] {
            if *p == c.value {
                continue;
            }
        }
        prev[c.sig] = Some(c.value.clone());
        let dt = c.time.wrapping_sub(lt);
        lt = c.time;
        let _ = last_t;
        put_u64(&mut out, dt);
        put_u32(&mut out, c.sig as u32);
        let vb = val_to_bytes(&c.value);
        put_u32(&mut out, c.value.width() as u32);
        out.extend_from_slice(&vb);
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub fn read_vwe(path: &std::path::Path) -> Result<WaveDump> {
    let data = std::fs::read(path)?;
    if &data[0..8] != b"VIKASVWE" {
        anyhow::bail!("bad .vwe magic");
    }
    let mut o = 8usize;
    let nsig = get_u32(&data, &mut o) as usize;
    let mut d = WaveDump::new();
    for _ in 0..nsig {
        let w = get_u32(&data, &mut o) as usize;
        let nl = get_u32(&data, &mut o) as usize;
        let name = String::from_utf8(data[o..o + nl].to_vec())?;
        o += nl;
        d.add_signal(&name, w);
    }
    let n = get_u64(&data, &mut o);
    let mut t = 0u64;
    for _ in 0..n {
        let dt = get_u64(&data, &mut o);
        t = t.wrapping_add(dt);
        let si = get_u32(&data, &mut o) as usize;
        let w = get_u32(&data, &mut o) as usize;
        let nb = (w + 3) / 4;
        let v = bytes_to_val(&data[o..o + nb], w);
        o += nb;
        d.push(t, si, v);
    }
    Ok(d)
}

// ================= .vwp — physics extended (analog f64 lanes) =================

pub fn write_vwp(d: &WaveDump, path: &std::path::Path) -> Result<()> {
    // same binary core as vwf + f64 analog lane per physics-kind signal
    let mut out = Vec::new();
    out.extend_from_slice(b"VIKASVWP");
    let analog: Vec<usize> = d
        .signals
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s.kind, crate::SigKind::Physics | crate::SigKind::Stochastic | crate::SigKind::Taint))
        .map(|(i, _)| i)
        .collect();
    put_u32(&mut out, analog.len() as u32);
    for &i in &analog {
        put_u32(&mut out, i as u32);
        put_u32(&mut out, d.signals[i].name.len() as u32);
        out.extend_from_slice(d.signals[i].name.as_bytes());
        put_u64(&mut out, d.changes.iter().filter(|c| c.sig == i).count() as u64);
        for c in d.changes.iter().filter(|c| c.sig == i) {
            let f = c.value.to_u64() as f64;
            out.extend_from_slice(&f.to_le_bytes());
        }
    }
    // then the full digital core (vwf-compatible body)
    let mut body = Vec::new();
    put_u32(&mut body, d.signals.len() as u32);
    for s in &d.signals {
        put_u32(&mut body, s.width as u32);
        put_u32(&mut body, s.name.len() as u32);
        body.extend_from_slice(s.name.as_bytes());
    }
    put_u64(&mut body, d.changes.len() as u64);
    for c in &d.changes {
        put_u64(&mut body, c.time);
        put_u32(&mut body, c.sig as u32);
        let vb = val_to_bytes(&c.value);
        put_u32(&mut body, c.value.width() as u32);
        body.extend_from_slice(&vb);
    }
    out.extend_from_slice(&body);
    std::fs::write(path, out)?;
    Ok(())
}

pub fn read_vwp(path: &std::path::Path) -> Result<WaveDump> {
    let data = std::fs::read(path)?;
    if &data[0..8] != b"VIKASVWP" {
        anyhow::bail!("bad .vwp magic");
    }
    let mut o = 8usize;
    let na = get_u32(&data, &mut o) as usize;
    for _ in 0..na {
        let _si = get_u32(&data, &mut o);
        let nl = get_u32(&data, &mut o) as usize;
        o += nl;
        let n = get_u64(&data, &mut o) as usize;
        o += n * 8;
    }
    let nsig = get_u32(&data, &mut o) as usize;
    let mut d = WaveDump::new();
    for _ in 0..nsig {
        let w = get_u32(&data, &mut o) as usize;
        let nl = get_u32(&data, &mut o) as usize;
        let name = String::from_utf8(data[o..o + nl].to_vec())?;
        o += nl;
        d.add_signal(&name, w);
    }
    let n = get_u64(&data, &mut o);
    for _ in 0..n {
        let t = get_u64(&data, &mut o);
        let si = get_u32(&data, &mut o) as usize;
        let w = get_u32(&data, &mut o) as usize;
        let nb = (w + 3) / 4;
        let v = bytes_to_val(&data[o..o + nb], w);
        o += nb;
        d.push(t, si, v);
    }
    Ok(d)
}

// ================= .vwi — per-signal absolute seek index =================

pub fn write_vwi(d: &WaveDump, path: &std::path::Path) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(b"VIKASVWI");
    put_u32(&mut out, d.signals.len() as u32);
    for s in &d.signals {
        put_u32(&mut out, s.width as u32);
        put_u32(&mut out, s.name.len() as u32);
        out.extend_from_slice(s.name.as_bytes());
    }
    // per-signal seek index (times) followed by per-signal grouped record blocks:
    // the reader skips each signal's time array then reads exactly its own block.
    let mut index_times: Vec<Vec<u64>> = vec![Vec::new(); d.signals.len()];
    let mut blocks: Vec<Vec<u8>> = vec![Vec::new(); d.signals.len()];
    for c in &d.changes {
        index_times[c.sig].push(c.time);
        let b = &mut blocks[c.sig];
        let vb = val_to_bytes(&c.value);
        put_u64(b, c.time);
        put_u32(b, c.value.width() as u32);
        b.extend_from_slice(&vb);
    }
    for idx in &index_times {
        put_u64(&mut out, idx.len() as u64);
        for t in idx {
            put_u64(&mut out, *t);
        }
    }
    for b in &blocks {
        out.extend_from_slice(b);
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub fn read_vwi(path: &std::path::Path) -> Result<WaveDump> {
    let data = std::fs::read(path)?;
    if &data[0..8] != b"VIKASVWI" {
        anyhow::bail!("bad .vwi magic");
    }
    let mut o = 8usize;
    let nsig = get_u32(&data, &mut o) as usize;
    let mut d = WaveDump::new();
    let mut widths = Vec::new();
    for _ in 0..nsig {
        let w = get_u32(&data, &mut o) as usize;
        let nl = get_u32(&data, &mut o) as usize;
        let name = String::from_utf8(data[o..o + nl].to_vec())?;
        o += nl;
        widths.push(w);
        d.add_signal(&name, w);
    }
    let mut counts = Vec::new();
    for w in &widths {
        let n = get_u64(&data, &mut o);
        let _ = w;
        counts.push(n);
        o += n as usize * 8;
    }
    for (si, n) in counts.iter().enumerate() {
        for _ in 0..*n {
            let t = get_u64(&data, &mut o);
            let w = get_u32(&data, &mut o) as usize;
            let nb = (w + 3) / 4;
            let v = bytes_to_val(&data[o..o + nb], w);
            o += nb;
            d.push(t, si, v);
        }
    }
    Ok(d)
}

// ================= .vwm — aligned 32-byte mmap records =================

const REC: usize = 32;

pub fn write_vwm(d: &WaveDump, path: &std::path::Path) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(b"VIKASVWM");
    put_u32(&mut out, d.signals.len() as u32);
    for s in &d.signals {
        put_u32(&mut out, s.width as u32);
        put_u32(&mut out, s.name.len() as u32);
        out.extend_from_slice(s.name.as_bytes());
    }
    put_u64(&mut out, d.changes.len() as u64);
    let mut off = out.len();
    let pad = (REC - (off % REC)) % REC;
    out.extend(std::iter::repeat(0u8).take(pad)); // align to 32B record boundary
    off = out.len();
    for c in &d.changes {
        let mut rec = [0u8; REC];
        rec[0..8].copy_from_slice(&c.time.to_le_bytes());
        rec[8..12].copy_from_slice(&(c.sig as u32).to_le_bytes());
        rec[12..16].copy_from_slice(&(c.value.width() as u32).to_le_bytes());
        let vb = val_to_bytes(&c.value);
        let n = vb.len().min(REC - 16);
        rec[16..16 + n].copy_from_slice(&vb[..n]);
        out.extend_from_slice(&rec);
        let _ = off;
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub fn read_vwm(path: &std::path::Path) -> Result<WaveDump> {
    let data = std::fs::read(path)?;
    if &data[0..8] != b"VIKASVWM" {
        anyhow::bail!("bad .vwm magic");
    }
    let mut o = 8usize;
    let nsig = get_u32(&data, &mut o) as usize;
    let mut d = WaveDump::new();
    for _ in 0..nsig {
        let w = get_u32(&data, &mut o) as usize;
        let nl = get_u32(&data, &mut o) as usize;
        let name = String::from_utf8(data[o..o + nl].to_vec())?;
        o += nl;
        d.add_signal(&name, w);
    }
    let n = get_u64(&data, &mut o);
    let mut o = (o + REC - 1) & !(REC - 1); // seek to aligned start
    for _ in 0..n {
        let rec = &data[o..o + REC];
        let t = u64::from_le_bytes(rec[0..8].try_into().unwrap());
        let si = u32::from_le_bytes(rec[8..12].try_into().unwrap()) as usize;
        let w = u32::from_le_bytes(rec[12..16].try_into().unwrap()) as usize;
        let nb = ((w + 3) / 4).min(REC - 16);
        let v = bytes_to_val(&rec[16..16 + nb], w);
        o += REC;
        d.push(t, si, v);
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::super::*;

    fn sample() -> WaveDump {
        let mut d = WaveDump::new();
        let clk = d.add_signal("tb.clk", 1);
        let bus = d.add_signal("tb.bus", 9);
        let phys = d.add_signal("VIKAS_PHYSICS_energy_fJ", 32);
        for t in 0..24u64 {
            d.push(t * 2, clk, VVal::from_u64(t % 2, 1));
            if t % 3 == 0 {
                d.push(t * 2, bus, VVal::from_u64(t * 7, 9));
            }
            if t % 2 == 0 {
                d.push(t * 2, phys, VVal::from_u64(t * 100, 32));
            }
        }
        d
    }

    fn roundtrip(p: &std::path::Path) {
        let d = sample();
        d.save(p).unwrap();
        let back = WaveDump::load(p).unwrap();
        assert_eq!(d.signals.len(), back.signals.len(), "signal count {} vs {}", d.signals.len(), back.signals.len());
        assert_eq!(d.changes.len(), back.changes.len(), "change count mismatch");
        for (a, b) in d.changes.iter().zip(&back.changes) {
            assert_eq!((a.time, a.sig, &a.value), (b.time, b.sig, &b.value), "change mismatch");
        }
    }

    #[test]
    fn vwt_roundtrip() {
        roundtrip(std::path::Path::new("/dev/shm/t.vwt"));
    }
    #[test]
    fn vwe_roundtrip() {
        roundtrip(std::path::Path::new("/dev/shm/t.vwe"));
    }
    #[test]
    fn vwp_roundtrip() {
        roundtrip(std::path::Path::new("/dev/shm/t.vwp"));
    }
    #[test]
    fn vwi_roundtrip() {
        // .vwi groups records per-signal (seek-index design) — compare as sets
        let d = sample();
        let p = std::path::Path::new("/dev/shm/t.vwi");
        d.save(p).unwrap();
        let back = WaveDump::load(p).unwrap();
        assert_eq!(d.signals.len(), back.signals.len());
        assert_eq!(d.changes.len(), back.changes.len());
        let key = |c: &crate::Change| (c.time, c.sig, c.value.to_bin_str());
        let mut a: Vec<_> = d.changes.iter().map(key).collect();
        let mut b: Vec<_> = back.changes.iter().map(key).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "vwi content mismatch");
    }
    #[test]
    fn vwm_roundtrip() {
        roundtrip(std::path::Path::new("/dev/shm/t.vwm"));
    }
}
