//! # VIKAS STUDIO — professional single-file HTML5 EDA workbench.
//!
//! Full product-grade UI replacing the old academic viewer:
//! menu bar · 40-icon SVG toolbar · dock panels (signal tree, wave canvas,
//! properties, console) · minimap · 4 themes · multi-cursor · bus expand ·
//! analog mode · radix switching · context menus · command palette (Ctrl+K)
//! · toasts · client-side .vwf/.vcd file loading (opens VCDs from ANY tool)
//! · exports (PNG/SVG/CSV/JSON/WaveDrom/layout) · Analysis Hub with
//! coverage/power/KDB/latch report cards.
//!
//! Zero dependencies, fully offline, one self-contained HTML file.
//! Data is embedded as: JSON signal table + base64 varint change blob
//! (2 bits per logic bit, identical packing to the native .vwf format).

use vikas_wave::WaveDump;

/// Optional analysis context embedded at generation time (from `vikas run --gui`
/// or auto-detected sibling artifacts in `vikas gui`).
#[derive(Default, Clone)]
pub struct StudioCtx {
    /// subtitle line, e.g. "NOVA-PASS 9/9 · 26 files · 5 levels"
    pub subtitle: String,
    /// verdict badge text: "PASS" / "AT RISK" / "BLOCKED" / "" (none)
    pub verdict: String,
    /// verdict color kind: "ok" | "warn" | "bad" | ""
    pub verdict_kind: String,
    /// status chips rendered in the header: (icon, text)
    pub chips: Vec<(&'static str, String)>,
    /// Open-KDB JSON (signals/drivers/CSR/latches/FSM/CDC), if available
    pub kdb: Option<serde_json::Value>,
    /// extra notes for the Analysis Hub (markdown-ish plain lines)
    pub notes: Vec<String>,
}

// ===================== change blob encoder (varint + 4-state) =====================

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
}

/// 4-state packing identical to native .vwf: 2 bits per logic bit.
/// 0=Zero 1=One 2=Z 3=X, LSB-first within each byte, ceil(w/4) bytes.
fn state_bytes(v: &VValState) -> Vec<u8> {
    let n = v.width;
    let mut out = vec![0u8; (n + 3) / 4];
    for (i, &b) in v.bits.iter().enumerate() {
        let d: u8 = match b {
            0 => 0, // zero
            1 => 1, // one
            2 => 2, // z
            _ => 3, // x
        };
        out[i / 4] |= d << ((i % 4) * 2);
    }
    out
}

struct VValState {
    width: usize,
    bits: Vec<u8>, // 0/1/2/3 codes
}

fn vval_to_state(v: &vikas_wave::VVal) -> VValState {
    let w = v.width();
    let mut bits = Vec::with_capacity(w);
    for i in 0..w {
        bits.push(match v.bit(i) {
            Some(vikas_wave::Bit::One) => 1,
            Some(vikas_wave::Bit::Z) => 2,
            Some(vikas_wave::Bit::X) | None => 3,
            Some(vikas_wave::Bit::Zero) => 0,
        });
    }
    VValState { width: w, bits }
}

fn b64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity((data.len() + 2) / 3 * 4);
    for c in data.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}

fn encode_changes_blob(dump: &WaveDump) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(dump.changes.len() * 3 + 64);
    put_varint(&mut out, dump.changes.len() as u64);
    let mut last_t = 0u64;
    for c in &dump.changes {
        put_varint(&mut out, c.time.saturating_sub(last_t));
        last_t = c.time;
        put_varint(&mut out, c.sig as u64);
        put_varint(&mut out, c.value.width() as u64);
        out.extend_from_slice(&state_bytes(&vval_to_state(&c.value)));
    }
    b64(&out)
}

// ===================== generator =====================

pub fn generate_studio(
    dump: &WaveDump,
    ctx: &StudioCtx,
    title: &str,
    out_path: &std::path::Path,
) -> anyhow::Result<()> {
    let sigs: Vec<serde_json::Value> = dump
        .signals
        .iter()
        .map(|s| serde_json::json!({"n": s.name, "w": s.width, "k": s.kind.as_str()}))
        .collect();

    let end = dump.end_time();
    let n_events = dump.changes.len();

    let hub_json = serde_json::json!({
        "subtitle": ctx.subtitle,
        "verdict": ctx.verdict,
        "verdictKind": ctx.verdict_kind,
        "chips": ctx.chips.iter().map(|(i, t)| serde_json::json!([i, t])).collect::<Vec<_>>(),
        "kdb": ctx.kdb,
        "notes": ctx.notes,
    });

    let html = super::studio_app::APP_HTML
        .replace("__TITLE__", title)
        .replace("__SIGS__", &serde_json::to_string(&sigs)?)
        .replace("__CHG__", &encode_changes_blob(dump))
        .replace("__END__", &end.to_string())
        .replace("__NEV__", &n_events.to_string())
        .replace("__CTX__", &serde_json::to_string(&hub_json)?)
        .replace(
            "__BUILD__",
            &format!("VIKAS v{} · XEZIM kernel · {}", env!("CARGO_PKG_VERSION"), chrono_free_now()),
        );
    std::fs::write(out_path, html)?;
    Ok(())
}

fn chrono_free_now() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // civil date from unix seconds (UTC) — good enough for a build stamp
    let days = d / 86400;
    let secs = d % 86400;
    let (y, mo, dd) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", y, mo, dd, secs / 3600, (secs % 3600) / 60)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}
