//! VIKAS v1 — SIMD-style bit-parallel temporal query engine (breakthrough 11)
//! + FFT ENOB spectral engine + bus decoder + disassembler + fixed/float radix.

pub mod radix {
    /// Qm.n fixed-point decode of a raw u64 (two's complement).
    pub fn qmn(raw: u64, m: i32, n: i32) -> f64 {
        let total = m + n;
        let mask = if total >= 64 { u64::MAX } else { (1u64 << total) - 1 };
        let v = raw & mask;
        let signed = if total >= 64 {
            v as i64
        } else {
            let sign = 1u64 << (total - 1);
            if v & sign != 0 {
                (v.wrapping_sub(mask + 1)) as i64
            } else {
                v as i64
            }
        };
        signed as f64 / (1i64 << n) as f64
    }
    pub fn fp16(raw: u64) -> f64 {
        let h = (raw & 0xFFFF) as u16;
        let sign = ((h >> 15) & 1) as u32;
        let exp = ((h >> 10) & 0x1F) as u32;
        let man = (h & 0x3FF) as u32;
        let f = if exp == 0 {
            if man == 0 {
                0.0
            } else {
                let mut e = -14i32;
                let mut m = man;
                while m & 0x400 == 0 {
                    m <<= 1;
                    e -= 1;
                }
                (m & 0x3FF) as f64 * 2f64.powi(e) / 1024.0
            }
        } else if exp == 0x1F {
            f64::NAN
        } else {
            (1.0 + man as f64 / 1024.0) * 2f64.powi(exp as i32 - 15)
        };
        if sign == 1 {
            -f
        } else {
            f
        }
    }
}

/// Hann-windowed FFT spectral metrics (feature 23): SNR, SINAD, THD, SFDR, ENOB.
pub struct SpectralMetrics {
    pub fs_mhz: f64,
    pub fundamental_hz: f64,
    pub snr_db: f64,
    pub sinad_db: f64,
    pub thd_pct: f64,
    pub sfdr_db: f64,
    pub enob_bits: f64,
}

pub fn fft_metrics(samples: &[f64], fs_hz: f64) -> SpectralMetrics {
    use rustfft::{num_complex::Complex, FftPlanner};
    let n = samples.len().next_power_of_two().max(4);
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let mut buf: Vec<Complex<f64>> = (0..n)
        .map(|i| {
            let x = samples.get(i).copied().unwrap_or(0.0);
            let w = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos());
            Complex::new(x * w, 0.0)
        })
        .collect();
    fft.process(&mut buf);
    let mag: Vec<f64> = buf.iter().map(|c| (c.re * c.re + c.im * c.im).sqrt()).collect();
    let fund_bin = (1..n / 2).max_by(|a, b| mag[*a].partial_cmp(&mag[*b]).unwrap()).unwrap_or(1);
    let fund = mag[fund_bin];
    let signal_pow = fund * fund;
    let mut noise_pow = 0.0;
    let mut harm_pow = 0.0;
    let mut max_spur = 0.0f64;
    for i in 1..n / 2 {
        if i == fund_bin || fund == 0.0 {
            continue;
        }
        if i % fund_bin == 0 && i <= fund_bin * 8 {
            harm_pow += mag[i] * mag[i];
            if mag[i] > max_spur {
                max_spur = mag[i];
            }
        } else {
            noise_pow += mag[i] * mag[i];
            if mag[i] > max_spur {
                max_spur = mag[i];
            }
        }
    }
    let _ = max_spur;
    let sfur = mag.iter().enumerate().skip(1).take(n / 2).filter(|(i, _)| *i != fund_bin).map(|(_, m)| *m).fold(0.0f64, f64::max);
    let snr = 10.0 * (signal_pow / noise_pow.max(1e-30)).log10();
    let sinad = 10.0 * (signal_pow / (noise_pow + harm_pow + 1e-30)).log10();
    let thd = 100.0 * (harm_pow / signal_pow.max(1e-30)).sqrt();
    let sfdr = 20.0 * (fund / sfur.max(1e-30)).log10();
    SpectralMetrics {
        fs_mhz: fs_hz / 1e6,
        fundamental_hz: fs_hz * fund_bin as f64 / n as f64,
        snr_db: snr,
        sinad_db: sinad,
        thd_pct: thd,
        sfdr_db: sfdr,
        enob_bits: (sinad - 1.76) / 6.02,
    }
}

/// Bus protocol auto-decoder (feature 21): AXI/APB/SPI/I2C/UART pin naming.
pub fn decode_bus(
    names: &[String],
    steps: &[Vec<(u64, super::VVal)>],
) -> Vec<String> {
    let mut out = Vec::new();
    let find = |pats: &[&str]| -> Option<usize> {
        for (i, n) in names.iter().enumerate() {
            let ln = n.to_lowercase();
            if pats.iter().any(|p| ln.contains(p)) {
                return Some(i);
            }
        }
        None
    };
    // UART framing
    if let (Some(tx), Some(baud_idx)) = (find(&["tx"]), find(&["baud"])) {
        let _ = baud_idx;
        let st = &steps[tx];
        let mut started = false;
        for w in st.windows(2) {
            if w[0].1.to_u64() == 0 && w[1].1.to_u64() == 1 && !started {
                out.push(format!("[UART] start bit at t={}ns", w[1].0));
                started = true;
            }
        }
    }
    // SPI: cs_n falling edge + mosi sampling
    if let (Some(cs), Some(clk), Some(mosi)) = (find(&["cs"]), find(&["sclk", "spi_clk"]), find(&["mosi"])) {
        let cs_steps = &steps[cs];
        let clk_steps = &steps[clk];
        let mosi_steps = &steps[mosi];
        for w in cs_steps.windows(2) {
            if w[0].1.to_u64() == 1 && w[1].1.to_u64() == 0 {
                // CS asserted: sample 8 bits from MOSI on following posedges
                let t0 = w[1].0;
                let edges: Vec<u64> = clk_steps.iter().filter(|(t, v)| *t >= t0 && v.to_u64() == 1 && *t <= t0 + 200).map(|(t, _)| *t).take(8).collect();
                let mut byte = 0u16;
                for (bi, te) in edges.iter().enumerate() {
                    if let Some((_, v)) = mosi_steps.iter().rev().find(|(t, _)| t <= te) {
                        byte |= (v.to_u64() as u16) << (7 - bi);
                    }
                }
                if !edges.is_empty() {
                    out.push(format!("[SPI] CS@{}ns MOSI byte = 0x{:02X}", t0, byte));
                }
            }
        }
    }
    // APB/AXI-lite simple write/read logging
    if let (Some(paddr), Some(pwdata), Some(pwrite), Some(psel), Some(penable)) =
        (find(&["paddr", "axi_awaddr"]), find(&["pwdata", "wdata"]), find(&["pwrite", "awvalid"]), find(&["psel", "awready"]), find(&["penable", "wvalid"]))
    {
        let wr_steps = &steps[pwrite];
        for w in wr_steps.windows(2) {
            let (t0, v0) = (&w[0].1, &w[1].1);
            let active = v0.to_u64() == 1 && t0.to_u64() != v0.to_u64().min(2);
            let _ = active;
            let sel = steps[psel].iter().find(|(t, _)| *t == w[1].0).map(|(_, v)| v.to_u64()).unwrap_or(0);
            let ena = steps[penable].iter().find(|(t, _)| *t == w[1].0).map(|(_, v)| v.to_u64()).unwrap_or(0);
            if sel == 1 && ena == 1 {
                let addr = steps[paddr].iter().rev().find(|(t, _)| t <= &w[1].0).map(|(_, v)| v.to_u64()).unwrap_or(0);
                let data = steps[pwdata].iter().rev().find(|(t, _)| t <= &w[1].0).map(|(_, v)| v.to_u64()).unwrap_or(0);
                let op = if w[1].1.to_u64() == 1 { "WRITE" } else { "READ " };
                out.push(format!("[BUS] t={}ns {} addr=0x{:08X} data=0x{:08X}", w[1].0, op, addr, data));
            }
        }
    }
    out
}

/// RV32I subset disassembler (feature 30).
pub fn disasm_rv32i(word: u32) -> String {
    let op = word & 0x7F;
    let rd = (word >> 7) & 0x1F;
    let f3 = (word >> 12) & 0x7;
    let rs1 = (word >> 15) & 0x1F;
    let rs2 = (word >> 20) & 0x1F;
    let imm_i = ((word >> 20) as i32) >> 20;
    let imm_s = (((word >> 25) as i32) << 5) | ((word >> 7 & 0x1F) as i32);
    let reg = |r: u32| format!("x{}", r);
    match op {
        0x33 => match f3 {
            0 => format!("add {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            1 => format!("sll {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            2 => format!("slt {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            4 => format!("xor {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            5 => format!("srl {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            6 => format!("or {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            7 => format!("and {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
            _ => format!("sub {}, {}, {}", reg(rd), reg(rs1), reg(rs2)),
        },
        0x13 => match f3 {
            0 => format!("addi {}, {}, {}", reg(rd), reg(rs1), imm_i),
            4 => format!("xori {}, {}, {}", reg(rd), reg(rs1), imm_i),
            6 => format!("ori {}, {}, {}", reg(rd), reg(rs1), imm_i),
            7 => format!("andi {}, {}, {}", reg(rd), reg(rs1), imm_i),
            _ => format!("slti {}, {}, {}", reg(rd), reg(rs1), imm_i),
        },
        0x03 => format!("lw {}, {}({})", reg(rd), imm_i, reg(rs1)),
        0x23 => format!("sw {}, {}({})", reg(rs2), imm_s, reg(rs1)),
        0x63 => match f3 {
            0 => format!("beq {}, {}, {}", reg(rs1), reg(rs2), imm_i),
            1 => format!("bne {}, {}, {}", reg(rs1), reg(rs2), imm_i),
            _ => format!("b? {}, {}, {}", reg(rs1), reg(rs2), imm_i),
        },
        0x37 => format!("lui {}, 0x{:X}", reg(rd), word >> 12),
        0x6F => format!("jal {}, {}", reg(rd), imm_i),
        0x67 => format!("jalr {}, {}, {}", reg(rd), reg(rs1), imm_i),
        0x73 => "ecall".into(),
        _ => format!(".word 0x{:08X}", word),
    }
}

/// Bit-parallel temporal search (breakthrough 11): word-parallel AND/OR/POPCNT.
pub fn bit_parallel_find(
    steps: &[Vec<(u64, super::VVal)>],
    sig_indices: &[usize],
    ops: &[u8], // b'&' AND, b'|' OR, b'!' NOT
    t_max: u64,
) -> Vec<u64> {
    // build bit-vectors: bit t set iff signal==1 at time t
    let n = t_max as usize + 1;
    let words = (n + 63) / 64;
    let mut vecs: Vec<Vec<u64>> = Vec::new();
    for &si in sig_indices {
        let st = &steps[si];
        let mut v = vec![0u64; words];
        let mut cur = 0u64;
        let mut last_t = 0u64;
        for &(t, ref val) in st {
            if val.width() == 1 {
                if val.to_u64() == 1 {
                    for tt in last_t..t.min(t_max + 1) {
                        v[(tt as usize) / 64] |= 1u64 << (tt % 64);
                    }
                    cur = 1;
                } else {
                    for tt in last_t..t.min(t_max + 1) {
                        // leave zero
                        let _ = tt;
                    }
                    cur = 0;
                }
                if t <= t_max {
                    if cur == 1 {
                        v[(t as usize) / 64] |= 1u64 << (t % 64);
                    }
                }
                last_t = t;
            }
        }
        // fill tail
        if cur == 1 {
            for tt in last_t..=t_max {
                v[(tt as usize) / 64] |= 1u64 << (tt % 64);
            }
        }
        vecs.push(v);
    }
    if vecs.is_empty() {
        return Vec::new();
    }
    // fold with ops
    let mut acc = vec![!0u64; words]; // neutral for AND
    let mut first = true;
    let mut oi = 0;
    for v in &vecs {
        if first {
            acc = v.clone();
            first = false;
        } else {
            let op = ops.get(oi).copied().unwrap_or(b'&');
            oi += 1;
            match op {
                b'|' => for w in 0..words {
                    acc[w] |= v[w];
                },
                b'!' => for w in 0..words {
                    acc[w] &= !v[w];
                },
                _ => for w in 0..words {
                    acc[w] &= v[w];
                },
            }
        }
    }
    let mut hits = Vec::new();
    for w in 0..words {
        let mut x = acc[w];
        while x != 0 {
            let tz = x.trailing_zeros() as u64;
            let t = (w as u64) * 64 + tz;
            if t <= t_max {
                hits.push(t);
            }
            x &= x - 1;
        }
    }
    hits
}
