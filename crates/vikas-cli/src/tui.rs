//! Feature 43 — VIKAS TUI v1.1: keyboard-first interactive terminal viewer.
//! Pure ANSI escape implementation (zero deps): branded header, kind-colored
//! signal list, hex-segment bus lanes, time ruler, cursor + measure column,
//! zoom/pan, filter, and a persistent key bar. PTY-safe.

use anyhow::Result;
use std::io::{Read, Write};
use std::path::PathBuf;
use vikas_wave::{VVal, WaveDump};

use crate::util::{c, BOLD, CYAN, DIM, GREEN, RED, YELLOW};

fn kind_color(kind: &str) -> &'static str {
    match kind {
        "clock" => CYAN,
        "reset" => RED,
        "physics" => YELLOW,
        "causality" => RED,
        "latch" => RED,
        "taint" => GREEN,
        _ => GREEN,
    }
}

pub fn run(dump_path: PathBuf) -> Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) && !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        println!("{} the interactive TUI needs a real terminal (TTY).",
            c("[VIKAS TUI]", YELLOW));
        println!("  - from a desktop terminal, run:  vikas tui {}", dump_path.display());
        println!("  - for headless analysis use:      vikas summary / annotate / coverage");
        println!("  - for the full workbench use:    vikas gui {}", dump_path.display());
        anyhow::bail!("no interactive terminal attached");
    }
    let dump = WaveDump::load(&dump_path)?;
    let end = dump.end_time().max(1) as f64;
    let steps = dump.steps();
    let total_events: usize = steps.iter().map(|s| s.len()).sum();

    let mut t0 = 0.0f64;
    let mut span = end;
    let mut cursor: Option<f64> = None;
    let mut cursor_mode = false;
    let mut filter = String::new();
    let mut sel: Option<usize> = dump
        .signals
        .iter()
        .position(|s| s.name.rsplit('.').next().unwrap_or("").contains("clk"))
        .or(Some(0));
    let mut cur = sel.unwrap_or(0);

    let _ = std::process::Command::new("stty").arg("raw").arg("-echo").status();
    let mut out = std::io::stdout();
    let mut quit = false;

    let name_w = 34usize;
    let wave_w = 96usize;

    let render = |dump: &WaveDump, steps: &Vec<Vec<(u64, VVal)>>, sel: &Option<usize>,
                  t0: f64, span: f64, cursor: &Option<f64>, cursor_mode: bool,
                  filter: &str, out: &mut std::io::Stdout| {
        let _ = write!(out, "\x1b[2J\x1b[H");
        let w = 34 + name_w + wave_w + 8;
        let end = dump.end_time().max(1) as f64;

        // ---- header box ----
        let _ = writeln!(out, "{}", c(&format!("╔{}╗", "═".repeat(w + 10)), DIM));
        let hdr1 = format!(
            "║ {} — {} · {} signals · {} events · t ∈ [0 … {}] ns",
            c("VIKAS TUI", CYAN),
            dump_path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default(),
            dump.signals.len(),
            total_events,
            end
        );
        let _ = writeln!(out, "{:<w$}{}", hdr1, c("║", DIM), w = w + 10);
        let mode_chip = if cursor_mode { c("CURSOR", YELLOW) } else { c("PAN", DIM) };
        let sel_chip = sel
            .map(|si| format!("sel: {}", c(&dump.signals[si].name.chars().take(30).collect::<String>(), CYAN)))
            .unwrap_or_default();
        let hdr2 = format!(
            "║ view [{:>7.1} … {:>7.1}] ns · zoom {:>3.0}% · mode {} · {} · cursor {}",
            t0,
            t0 + span,
            100.0 * end / span,
            mode_chip,
            sel_chip,
            cursor.map(|t| format!("{} ns", c(&format!("{:.1}", t), YELLOW))).unwrap_or("—".into())
        );
        let _ = writeln!(out, "{:<w$}{}", hdr2, c("║", DIM), w = w + 10);
        let _ = writeln!(out, "{}", c(&format!("╚{}╝", "═".repeat(w + 10)), DIM));

        // ---- time ruler ----
        let step = nice_step(span / (wave_w as f64 / 8.0));
        let mut ruler = String::from(" ".repeat(name_w + 4));
        let mut tt = ((t0 / step).ceil() as i64).max(0) as f64 * step;
        while tt <= t0 + span {
            let label = format!("{}", tt as u64);
            let x = ((tt - t0) / span * wave_w as f64) as usize;
            if x + label.len() < wave_w && x >= 0 {
                if ruler.len() < name_w + 4 + x {
                    ruler.push_str(&" ".repeat(name_w + 4 + x - ruler.len()));
                }
                ruler.push_str(&label);
            }
            tt += step;
        }
        let _ = writeln!(out, "{} ({})", c(&ruler, DIM), c("ns", DIM));

        // ---- signal rows ----
        let shown: Vec<usize> = (0..dump.signals.len())
            .filter(|i| filter.is_empty() || dump.signals[*i].name.contains(&filter[..]))
            .take(40)
            .collect();
        for &si in shown.iter() {
            let s = &dump.signals[si];
            let is_sel = Some(si) == *sel;
            let mark = if is_sel { c("▸", CYAN) } else { " ".to_string() };
            let col = kind_color(s.kind.as_str());
            let name: String = s.name.chars().take(name_w).collect::<String>();
            let val_chip = cursor
                .map(|t| {
                    let v = value_at_t(steps, si, t as u64);
                    if s.width == 1 {
                        format!("{}", v.to_u64())
                    } else if v.is_x() {
                        "x".into()
                    } else {
                        format!("0x{}", v.to_hex_str())
                    }
                })
                .unwrap_or_else(|| "·".into());

            // lane: hex-segment buses / square traces for 1-bit
            let lane = render_lane(steps, si, s.width, t0, span, wave_w);
            let _ = writeln!(
                out,
                " {} {}{}{} │{}",
                mark,
                c(&format!("{:<nw$}", name, nw = name_w), if is_sel { BOLD } else { col }),
                c(&format!("{:>3}b", s.width), DIM),
                c(&format!("{:>8}", val_chip), if cursor.is_some() { YELLOW } else { DIM }),
                lane
            );
        }
        if shown.is_empty() {
            let _ = writeln!(out, "{}", c("  (filter matches nothing — press / to edit, ESC to clear)", YELLOW));
        }

        // ---- key bar ----
        let _ = writeln!(out, "{}", c("─".repeat(w + 10), DIM));
        let _ = writeln!(
            out,
            " {} {} · {} {} · {} {} · {} {} · {} {} · {} {} · {} {} · {} {}",
            c("[j/k]", CYAN), c("select", DIM),
            c("[h/l ←→]", CYAN), if cursor_mode { c("cursor", YELLOW) } else { c("pan", DIM) },
            c("[+/-]", CYAN), c("zoom", DIM),
            c("[f]", CYAN), c("fit", DIM),
            c("[c]", CYAN), if cursor_mode { c("cursor on", YELLOW) } else { c("cursor mode", DIM) },
            c("[/]", CYAN), c("filter", DIM),
            c("[a]", CYAN), c("all", DIM),
            c("[q]", CYAN), c("quit", DIM)
        );
    };

    render(&dump, &steps, &sel, t0, span, &cursor, cursor_mode, &filter, &mut out);
    out.flush()?;

    let mut buf = [0u8; 1];
    while !quit {
        if std::io::stdin().read(&mut buf)? == 0 {
            break;
        }
        match buf[0] {
            b'q' | 0x03 => quit = true,
            b'j' => {
                cur = (cur + 1).min(dump.signals.len().saturating_sub(1));
                sel = Some(cur);
            }
            b'k' => {
                cur = cur.saturating_sub(1);
                sel = Some(cur);
            }
            b'h' | 0x1b | b'D' => {
                // h or arrow-left (ESC [ D arrives as separate bytes)
                if buf[0] == 0x1b {
                    let mut seq = [0u8; 2];
                    let _ = std::io::stdin().read(&mut seq);
                    if seq[1] == b'D' {
                        pan_or_cursor(&mut t0, &mut cursor, cursor_mode, span, -1.0, end);
                    } else if seq[1] == b'C' {
                        pan_or_cursor(&mut t0, &mut cursor, cursor_mode, span, 1.0, end);
                    }
                } else {
                    pan_or_cursor(&mut t0, &mut cursor, cursor_mode, span, -1.0, end);
                }
            }
            b'l' => pan_or_cursor(&mut t0, &mut cursor, cursor_mode, span, 1.0, end),
            b'+' | b'=' => {
                let center = cursor.unwrap_or(t0 + span / 2.0);
                span = (span / 1.5).max(end / 2000.0);
                t0 = (center - span / 2.0).max(0.0);
            }
            b'-' | b'_' => {
                let center = cursor.unwrap_or(t0 + span / 2.0);
                span = (span * 1.5).min(end);
                t0 = (center - span / 2.0).max(0.0).min((end - span).max(0.0));
            }
            b'f' => {
                t0 = 0.0;
                span = end;
                cursor = None;
            }
            b'c' => {
                cursor_mode = !cursor_mode;
                if cursor_mode && cursor.is_none() {
                    cursor = Some(t0 + span / 2.0);
                }
            }
            b'0' => {
                t0 = 0.0;
                cursor = None;
            }
            b'/' => {
                // restore echo for typing, read line, back to raw
                let _ = std::process::Command::new("stty").arg("sane").status();
                print!("{}", c("filter> ", CYAN));
                let _ = out.flush();
                let mut f = String::new();
                let _ = std::io::stdin().read_line(&mut f);
                filter = f.trim().to_string();
                let _ = std::process::Command::new("stty").arg("raw").arg("-echo").status();
            }
            0x1b => { /* handled above via h/0x1b arm collision safety */ }
            b'a' => filter.clear(),
            _ => {}
        }
        render(&dump, &steps, &sel, t0, span, &cursor, cursor_mode, &filter, &mut out);
        out.flush()?;
    }
    let _ = std::process::Command::new("stty").arg("sane").status();
    println!("[VIKAS TUI] session ended. Re-open with: vikas tui {}", dump_path.display());
    println!("  Full workbench: vikas gui {}  (VIKAS Studio HTML5)", dump_path.display());
    Ok(())
}

fn pan_or_cursor(t0: &mut f64, cursor: &mut Option<f64>, cursor_mode: bool, span: f64, dir: f64, end: f64) {
    let d = span / 20.0 * dir;
    if cursor_mode {
        let t = cursor.unwrap_or(*t0 + span / 2.0) + d;
        *cursor = Some(t.clamp(0.0, end));
    } else {
        *t0 = (*t0 + d).clamp(0.0, (end - span).max(0.0));
    }
}

fn nice_step(raw: f64) -> f64 {
    let p = 10f64.powf(raw.log10().floor());
    for m in [1.0, 2.0, 5.0, 10.0] {
        if m * p >= raw {
            return m * p;
        }
    }
    10.0 * p
}

fn render_lane(steps: &[Vec<(u64, VVal)>], si: usize, width: usize, t0: f64, span: f64, lane_w: usize) -> String {
    let st = &steps[si];
    if st.is_empty() {
        return c(&"x".repeat(lane_w), RED);
    }
    let x_of = |t: f64| ((t - t0) / span * lane_w as f64) as isize;
    let mut lane = String::new();
    if width == 1 {
        // square trace with ▔ ▁ glyphs
        let mut prev_x = 0usize;
        let mut cur_v = value_at_t(steps, si, t0 as u64).to_u64();
        let mut push_to = |x: usize, v: u64, lane: &mut String| {
            let n = x.saturating_sub(prev_x);
            let ch = if v == 1 { '▔' } else { '_' };
            for _ in 0..n {
                lane.push(ch);
            }
            prev_x = x;
        };
        let mut started = false;
        for (t, v) in st.iter() {
            let tf = *t as f64;
            if tf < t0 {
                cur_v = v.to_u64();
                continue;
            }
            if tf > t0 + span {
                break;
            }
            if !started {
                push_to(0, cur_v, &mut lane);
                started = true;
            }
            let x = x_of(tf).max(0) as usize;
            push_to(x.min(lane_w), v.to_u64(), &mut lane);
            cur_v = v.to_u64();
        }
        if !started {
            push_to(lane_w, cur_v, &mut lane);
        } else {
            push_to(lane_w, cur_v, &mut lane);
        }
    } else {
        // hex-segment bus lane: 0xNN╲╱0xMM
        let mut prev_x = 0usize;
        let mut cur = value_at_t(steps, si, t0 as u64);
        let mut label = if cur.is_x() { "x".to_string() } else { format!("0x{}", cur.to_hex_str()) };
        let mut started = false;
        for (t, v) in st.iter() {
            let tf = *t as f64;
            if tf <= t0 {
                cur = v.clone();
                label = if cur.is_x() { "x".to_string() } else { format!("0x{}", cur.to_hex_str()) };
                continue;
            }
            if tf > t0 + span {
                break;
            }
            let x = (x_of(tf).max(0) as usize).min(lane_w);
            let room = x.saturating_sub(prev_x);
            if room >= label.len() + 2 {
                lane.push_str(&c(&label, crate::util::BOLD));
                for _ in 0..(room - label.len()) {
                    lane.push(' ');
                }
                lane.push_str(&c("╲╱", crate::util::DIM));
                prev_x = x;
            }
            cur = v.clone();
            label = if cur.is_x() { "x".to_string() } else { format!("0x{}", cur.to_hex_str()) };
            started = true;
        }
        let room = lane_w.saturating_sub(prev_x);
        if room >= label.len() && !started || room > label.len() {
            lane.push_str(&c(&label, crate::util::BOLD));
        }
        while lane.chars().count() < lane_w {
            lane.push(' ');
        }
    }
    if lane.chars().count() > lane_w {
        lane = lane.chars().take(lane_w).collect::<String>();
    }
    lane
}

fn value_at_t(steps: &[Vec<(u64, VVal)>], si: usize, t: u64) -> VVal {
    let st = &steps[si];
    let mut r = VVal::x(1);
    for (time, v) in st {
        if *time <= t {
            r = v.clone();
        } else {
            break;
        }
    }
    r
}
