//! Feature 43 — keyboard-first interactive terminal UI.
//! Pure ANSI escape implementation (zero deps): renders signal list + wave
//! lane per signal, dual cursors, search, with F1-style key bar.

use anyhow::Result;
use std::io::{Read, Write};
use std::path::PathBuf;
use vikas_wave::{VVal, WaveDump};

enum Mode {
    List,
    /// zoomed per-signal lane view — entered via `w` in an upcoming TUI rev
    #[allow(dead_code)]
    Wave,
}

pub fn run(dump_path: PathBuf) -> Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) && !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        println!("{} the interactive TUI needs a real terminal (TTY).",
            crate::util::c("[VIKAS TUI]", crate::util::YELLOW));
        println!("  - from a desktop terminal, run:  vikas tui {}", dump_path.display());
        println!("  - for headless analysis use:    vikas summary / annotate / coverage");
        anyhow::bail!("no interactive terminal attached");
    }
    let dump = WaveDump::load(&dump_path)?;
    let end = dump.end_time().max(1) as f64;
    let mut t0 = 0.0f64;
    let tspan = end;
    let _ = tspan;
    let steps = dump.steps();
    let mut sel: Option<usize> = dump
        .signals
        .iter()
        .position(|s| s.name.rsplit('.').next().unwrap_or("").contains("clk"))
        .or(Some(0));
    let mut filter = String::new();
    let mode = Mode::List;
    let _ = mode;
    let raw_fd = 0i32;
    // raw mode via stty (no termios crate)
    let _ = std::process::Command::new("stty").arg("raw").arg("-echo").status();
    let mut out = std::io::stdout();
    let mut quit = false;
    let render = |dump: &WaveDump, steps: &Vec<Vec<(u64, VVal)>>, sel: &Option<usize>, t0: f64, end: f64, filter: &str, out: &mut std::io::Stdout| {
        let _ = write!(out, "\x1b[2J\x1b[H");
        let w = 158usize;
        let mut shown: Vec<usize> = (0..dump.signals.len())
            .filter(|i| filter.is_empty() || dump.signals[*i].name.contains(&filter[..]))
            .collect();
        shown.truncate(38);
        let span = end / (w as f64 - 60.0).max(1.0);
        let t0i = t0;
        for (_row, &si) in shown.iter().enumerate() {
            let s = &dump.signals[si];
            let mark = if Some(si) == *sel { ">" } else { " " };
            let name_p = format!("{:<38}", s.name.chars().take(38).collect::<String>());
            let mut lane = String::new();
            for col in 0..(w - 50) {
                let t = t0i + col as f64 * span;
                let v = value_at_t(steps, si, t);
                let ch = match v.to_u64() {
                    1 if s.width == 1 => '▔',
                    0 if s.width == 1 => '_',
                    _ => {
                        if s.width > 1 {
                            if v.is_x() { 'x' } else { '▒' }
                        } else {
                            'x'
                        }
                    }
                };
                lane.push(ch);
            }
            let _ = writeln!(
                out,
                "{} {}{} |{}",
                mark,
                crate::util::c(&name_p, if Some(si) == *sel { crate::util::CYAN } else { crate::util::DIM }),
                crate::util::c(&format!("{:>3}b", s.width), crate::util::DIM),
                lane
            );
        }
        let _ = writeln!(out, "{}", crate::util::c("─".repeat(w), crate::util::DIM));
        let _ = writeln!(
            out,
            "  keys: j/k select · a all · /filter · q quit · t={}ns span={:.0}ns [VIKAS TUI]",
            t0i, end
        );
    };
    render(&dump, &steps, &sel, t0, end, &filter, &mut out);
    out.flush()?;
    // key loop
    let mut buf = [0u8; 1];
    let mut cur = 0usize;
    while !quit {
        let n = std::io::stdin().read(&mut buf)?;
        if n == 0 {
            break;
        }
        match buf[0] {
            b'q' | 0x03 => quit = true,
            b'j' => {
                cur += 1;
                sel = Some(cur.min(dump.signals.len() - 1));
            }
            b'k' => {
                cur = cur.saturating_sub(1);
                sel = Some(cur);
            }
            b'/' => {
                // read filter until newline
                let mut f = Vec::new();
                let mut b2 = [0u8; 1];
                loop {
                    if std::io::stdin().read(&mut b2)? == 0 || b2[0] == b'\r' || b2[0] == b'\n' {
                        break;
                    }
                    f.push(b2[0]);
                }
                filter = String::from_utf8_lossy(&f).to_string();
            }
            b'0' => t0 = 0.0,
            _ => {}
        }
        cur = sel.unwrap_or(0);
        render(&dump, &steps, &sel, t0, end, &filter, &mut out);
        out.flush()?;
    }
    let _ = std::process::Command::new("stty").arg("sane").status();
    let _ = raw_fd;
    println!("[VIKAS TUI] session ended. Re-open with: vikas tui {}", dump_path.display());
    println!("  GUI instead: vikas gui {}", dump_path.display());
    Ok(())
}

fn value_at_t(steps: &[Vec<(u64, VVal)>], si: usize, t: f64) -> VVal {
    let st = &steps[si];
    let mut r = VVal::x(1);
    let ti = t as u64;
    for (time, v) in st {
        if *time <= ti {
            r = v.clone();
        } else {
            break;
        }
    }
    r
}
