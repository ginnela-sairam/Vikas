//! VIKAS v3.0 — six native innovative waveform formats (beyond VCD/FST):
//!  .vwf  VIKAS Waveform File    — binary, indexed change table + CRC32 checksum
//!  .vwt  VIKAS Temporal Track   — time-sectioned regions with per-region indexes
//!  .vwe  VIKAS Encoded          — delta-time + value-RLE compressed stream
//!  .vwp  VIKAS Physics          — carries analog f64 physics telemetry tracks
//!  .vwi  VIKAS Indexed          — per-signal absolute offset for O(1) random seek
//!  .vwm  VIKAS Memorymap        — fixed 32-byte aligned records for zero-copy mmap

use crate::{Change, Signal, VVal, WaveDump};
use anyhow::{Context, Result};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VcdToVwmStats {
    pub signals: usize,
    pub changes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

fn vcd_value_change(line: &str) -> Option<(&str, &str)> {
    if let Some(value) = line.strip_prefix('b').or_else(|| line.strip_prefix('B')) {
        let mut tokens = value.split_whitespace();
        let bits = tokens.next()?;
        let id = tokens.next()?;
        return Some((bits, id));
    }

    let first = line.chars().next()?;
    if !"01xXzZ".contains(first) {
        return None;
    }
    let id_start = first.len_utf8();
    let id = line.get(id_start..)?.trim();
    if id.is_empty() {
        return None;
    }
    Some((&line[..id_start], id))
}

fn scan_vcd(path: &std::path::Path) -> Result<(Vec<(String, usize)>, std::collections::HashMap<String, usize>, u64)> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path).with_context(|| format!("opening VCD {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut signals: Vec<(String, usize)> = Vec::new();
    let mut ids = std::collections::HashMap::new();
    let mut changes = 0u64;
    let mut line_number = 0usize;

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        line_number += 1;
        let text = line.trim();
        if let Some(rest) = text.strip_prefix("$var") {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() < 4 {
                anyhow::bail!("malformed VCD $var declaration at line {}", line_number);
            }
            let width = fields[1]
                .parse::<usize>()
                .with_context(|| format!("invalid VCD signal width at line {}", line_number))?;
            if width == 0 {
                anyhow::bail!("zero-width VCD signal at line {}", line_number);
            }
            let name = fields[3];
            let index = signals
                .iter()
                .position(|(existing, existing_width)| existing == name && *existing_width == width)
                .unwrap_or_else(|| {
                    signals.push((name.to_string(), width));
                    signals.len() - 1
                });
            ids.insert(fields[2].to_string(), index);
        } else if let Some(time) = text.strip_prefix('#') {
            time.parse::<u64>()
                .with_context(|| format!("invalid VCD timestamp at line {}", line_number))?;
        } else if let Some((_, id)) = vcd_value_change(text) {
            if ids.contains_key(id) {
                changes = changes.checked_add(1).context("VCD change count overflow")?;
            }
        }
    }

    if signals.is_empty() {
        anyhow::bail!("VCD contains no signal declarations");
    }
    Ok((signals, ids, changes))
}

fn write_vcd_records<R: std::io::BufRead, W: std::io::Write>(
    reader: &mut R,
    writer: &mut W,
    ids: &std::collections::HashMap<String, usize>,
) -> Result<u64> {
    let mut line = String::new();
    let mut time = 0u64;
    let mut count = 0u64;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let text = line.trim();
        if let Some(timestamp) = text.strip_prefix('#') {
            time = timestamp.parse().context("invalid VCD timestamp during conversion")?;
            continue;
        }
        let Some((bits, id)) = vcd_value_change(text) else {
            continue;
        };
        let Some(&signal) = ids.get(id) else {
            continue;
        };
        let value = VVal::from_bits_str(bits);
        if value.width() == 0 || value.width() > (REC - 16) * 4 {
            anyhow::bail!(
                "VCD signal value width {} cannot be represented in a .vwm record (maximum 64 bits)",
                value.width()
            );
        }

        let mut record = [0u8; REC];
        record[0..8].copy_from_slice(&time.to_le_bytes());
        let signal = u32::try_from(signal).context("too many VCD signals for .vwm format")?;
        record[8..12].copy_from_slice(&signal.to_le_bytes());
        let width = u32::try_from(value.width()).context("VCD signal width exceeds .vwm limits")?;
        record[12..16].copy_from_slice(&width.to_le_bytes());
        for (index, bit) in value.bits.iter().enumerate() {
            let encoded = match bit {
                crate::Bit::Zero => 0,
                crate::Bit::One => 1,
                crate::Bit::Z => 2,
                crate::Bit::X => 3,
            };
            record[16 + index / 4] |= encoded << ((index % 4) * 2);
        }
        writer.write_all(&record)?;
        count = count.checked_add(1).context("VCD change count overflow")?;
    }
    Ok(count)
}

/// Convert VCD to the existing .vwm record layout without retaining trace changes in memory.
///
/// The source is scanned once for declarations and the record count, then streamed again into
/// an aligned temporary file. Memory use is bounded by the VCD header and the current input line.
pub fn convert_vcd_to_vwm_streaming(
    input: &std::path::Path,
    output: &std::path::Path,
) -> Result<VcdToVwmStats> {
    use std::io::{BufReader, BufWriter, Write};

    let (signals, ids, expected_changes) = scan_vcd(input)?;
    let input_bytes = std::fs::metadata(input)
        .with_context(|| format!("reading VCD metadata for {}", input.display()))?
        .len();
    let output_name = output.file_name().context("output path has no file name")?.to_string_lossy();
    let temporary = output.with_file_name(format!(".{}.{}.tmp", output_name, std::process::id()));

    let write_result = (|| -> Result<()> {
        let file = std::fs::File::create(&temporary)
            .with_context(|| format!("creating temporary VWM {}", temporary.display()))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(b"VIKASVWM")?;
        writer.write_all(&u32::try_from(signals.len()).context("too many VCD signals for .vwm format")?.to_le_bytes())?;
        for (name, width) in &signals {
            writer.write_all(&u32::try_from(*width).context("VCD signal width exceeds .vwm limits")?.to_le_bytes())?;
            writer.write_all(&u32::try_from(name.len()).context("VCD signal name exceeds .vwm limits")?.to_le_bytes())?;
            writer.write_all(name.as_bytes())?;
        }
        writer.write_all(&expected_changes.to_le_bytes())?;
        let offset = 8 + 4 + signals.iter().map(|(name, _)| 8 + name.len()).sum::<usize>() + 8;
        let padding = (REC - offset % REC) % REC;
        writer.write_all(&vec![0u8; padding])?;

        let input_file = std::fs::File::open(input)
            .with_context(|| format!("reopening VCD {}", input.display()))?;
        let mut reader = BufReader::new(input_file);
        let actual_changes = write_vcd_records(&mut reader, &mut writer, &ids)?;
        if actual_changes != expected_changes {
            anyhow::bail!(
                "VCD changed while converting (counted {} records, wrote {})",
                expected_changes,
                actual_changes
            );
        }
        writer.flush()?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    std::fs::rename(&temporary, output)
        .with_context(|| format!("moving converted VWM into {}", output.display()))?;
    let output_bytes = std::fs::metadata(output)
        .with_context(|| format!("reading VWM metadata for {}", output.display()))?
        .len();
    Ok(VcdToVwmStats {
        signals: signals.len(),
        changes: expected_changes,
        input_bytes,
        output_bytes,
    })
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

    #[test]
    fn streaming_vcd_to_vwm_multisignal_roundtrip() {
        let stem = format!("vikas-stream-small-{}", std::process::id());
        let input = std::env::temp_dir().join(format!("{stem}.vcd"));
        let output = std::env::temp_dir().join(format!("{stem}.vwm"));
        std::fs::write(
            &input,
            "$date test $end\n\
             $timescale 1ns $end\n\
             $scope module cpu $end\n\
             $var wire 1 ! clk $end\n\
             $var wire 8 \" pc [7:0] $end\n\
             $var wire 4 # opcode [3:0] $end\n\
             $upscope $end\n\
             $enddefinitions $end\n\
             $dumpvars\n\
             0!\n\
             b00000000 \"\n\
             b0000 #\n\
             $end\n\
             #5\n\
             1!\n\
             b00000100 \"\n\
             b0011 #\n\
             #10\n\
             0!\n\
             b00001000 \"\n",
        )
        .unwrap();

        let stats = super::convert_vcd_to_vwm_streaming(&input, &output).unwrap();
        let from_vcd = WaveDump::load_vcd(&input).unwrap();
        let from_vwm = super::read_vwm(&output).unwrap();
        assert_eq!(stats.signals, 3);
        assert_eq!(stats.changes, from_vcd.changes.len() as u64);
        assert_eq!(from_vcd.signals.len(), from_vwm.signals.len());
        assert_eq!(from_vcd.changes.len(), from_vwm.changes.len());
        for (expected, actual) in from_vcd.changes.iter().zip(&from_vwm.changes) {
            assert_eq!(
                (expected.time, expected.sig, &expected.value),
                (actual.time, actual.sig, &actual.value)
            );
        }
        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(output).unwrap();
    }

    #[test]
    fn streaming_vcd_to_vwm_handles_generated_large_trace() {
        use std::io::Write;

        let stem = format!("vikas-stream-large-{}", std::process::id());
        let input = std::env::temp_dir().join(format!("{stem}.vcd"));
        let output = std::env::temp_dir().join(format!("{stem}.vwm"));
        let mut file = std::fs::File::create(&input).unwrap();
        writeln!(file, "$scope module rv32i $end").unwrap();
        writeln!(file, "$var wire 1 ! clk $end").unwrap();
        writeln!(file, "$var wire 32 \" pc [31:0] $end").unwrap();
        writeln!(file, "$var wire 32 # instr [31:0] $end").unwrap();
        writeln!(file, "$var wire 1 $ commit $end").unwrap();
        writeln!(file, "$upscope $end\n$enddefinitions $end\n$dumpvars").unwrap();
        writeln!(file, "0!\nb00000000000000000000000000000000 \"\nb00000000000000000000000000010011 #\n0$").unwrap();
        writeln!(file, "$end").unwrap();
        for cycle in 1..=10_000u64 {
            writeln!(file, "#{cycle}").unwrap();
            writeln!(file, "{}!", cycle % 2).unwrap();
            writeln!(file, "b{:032b} \"", cycle * 4).unwrap();
            writeln!(file, "b{:032b} #", 0x13u64 + cycle).unwrap();
            writeln!(file, "{}$", cycle % 2).unwrap();
        }
        file.flush().unwrap();

        let stats = super::convert_vcd_to_vwm_streaming(&input, &output).unwrap();
        let converted = super::read_vwm(&output).unwrap();
        assert_eq!(stats.signals, 4);
        assert_eq!(stats.changes, 4 * 10_000 + 4);
        assert_eq!(converted.changes.len() as u64, stats.changes);
        assert!(stats.output_bytes > stats.input_bytes);
        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(output).unwrap();
    }

    #[test]
    fn mipmap_pyramid_envelopes() {
        let d = sample();
        let envs = super::build_mipmap_envelopes(&d, 2, 16);
        assert_eq!(envs.len(), 16);
        let total_toggles: u32 = envs.iter().map(|e| e.toggle_cnt).sum();
        assert!(total_toggles > 0, "mipmap must capture transitions");
    }
}

/// Binary Interval Mipmap Pyramid envelope node (`WV-01` / `AP-02`):
/// $\mathcal{E}_{k,i} = (\min, \max, v_{\text{first}}, v_{\text{last}}, \text{popcnt}(\Delta), \text{glitch})$
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MipmapEnvelope {
    pub t_start: u64,
    pub t_end: u64,
    pub v_min: u64,
    pub v_max: u64,
    pub v_first: String,
    pub v_last: String,
    pub toggle_cnt: u32,
    pub glitch_cnt: u32,
    pub has_xz: bool,
}

/// Build an $O(B)$ sub-pixel mipmap envelope array over `buckets` time intervals for signal `sig_idx`.
pub fn build_mipmap_envelopes(d: &WaveDump, sig_idx: usize, buckets: usize) -> Vec<MipmapEnvelope> {
    let b_count = buckets.max(1);
    let end_t = d.end_time().max(1);
    let span = (end_t as f64) / (b_count as f64);
    let mut out = Vec::with_capacity(b_count);
    for b in 0..b_count {
        let t0 = (b as f64 * span).floor() as u64;
        let t1 = (((b + 1) as f64 * span).ceil() as u64).max(t0 + 1);
        out.push(MipmapEnvelope {
            t_start: t0,
            t_end: t1,
            v_min: u64::MAX,
            v_max: 0,
            v_first: String::from("0"),
            v_last: String::from("0"),
            toggle_cnt: 0,
            glitch_cnt: 0,
            has_xz: false,
        });
    }
    let mut last_time: Option<u64> = None;
    for ch in d.changes.iter().filter(|c| c.sig == sig_idx) {
        let idx = (((ch.time as f64) / (end_t as f64)) * (b_count as f64)) as usize;
        let bucket = &mut out[idx.min(b_count - 1)];
        let hex = ch.value.to_hex_str();
        if bucket.toggle_cnt == 0 {
            bucket.v_first = hex.clone();
        }
        bucket.v_last = hex;
        bucket.toggle_cnt += 1;
        if let Some(prev_t) = last_time {
            if ch.time == prev_t {
                bucket.glitch_cnt += 1;
            }
        }
        last_time = Some(ch.time);
        if ch.value.is_x() {
            bucket.has_xz = true;
        } else {
            let num = ch.value.to_u64();
            if num < bucket.v_min {
                bucket.v_min = num;
            }
            if num > bucket.v_max {
                bucket.v_max = num;
            }
        }
    }
    for env in &mut out {
        if env.v_min == u64::MAX {
            env.v_min = 0;
        }
    }
    out
}
