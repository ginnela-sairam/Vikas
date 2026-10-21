//! VIKAS v1 — terminal color helpers & shared utils.

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
/// full ANSI palette - BLUE kept so scripts can address every hue uniformly
#[allow(dead_code)]
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

pub fn c<S: AsRef<str>>(s: S, color: &str) -> String {
    if std::env::var("VIKAS_NO_BANNER").is_ok() || !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        s.as_ref().to_string()
    } else {
        format!("{}{}{}", color, s.as_ref(), RESET)
    }
}

pub fn banner() {
    if std::env::var("VIKAS_NO_BANNER").is_ok() {
        return;
    }
    let b = r#"
██╗   ██╗██╗██╗ ██╗ █████╗ ███████╗   ██╗   ██╗ ██╗
 ██║ ██╔╝██║██║ ██╔╝██╔══██╗██╔════╝   ██║   ██╗███║
  █████╔╝ ██████╔╝███████║███████╗   ╚██╗ ██╔╝╚██║
  ██╔═██╗ ██╔══██╗██╔══██║╚════██║    ╚████╔╝  ██║
  ██║  ██╗██║  ██║██║  ██║███████║     ╚═══╝   ██║
  ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝            ╚═╝
  [VIKAS v1] Open Linux-Native RTL Platform — XEZIM Kernel + Open-KDB + Wave Engine
  100% Rust core · C++20 mixed-signal twin · offline · zero Python
"#;
    println!("{}", c(b.trim_start_matches('\n'), CYAN));
}

// ============================================================================
// GUI AUTO-OPEN — zero-dependency browser bridge (std::process cascade).
// `vikas gui` writes a fully self-contained HTML file, then pops it open in
// the OS default browser (a new window/tab) so the user never has to hunt
// for the file by hand. Bridges, in order:
//   WSL     : wslview -> powershell.exe -> cmd.exe start -> explorer.exe
//   Linux   : xdg-open -> gio open
//   macOS   : open
//   Windows : cmd /C start -> explorer
// ============================================================================
pub enum BrowserOpen {
    /// Browser launched through this bridge (e.g. "xdg-open", "wslview").
    Opened(String),
    /// No desktop/browser bridge on this system (headless box, container).
    NoBridge,
}

/// Should `vikas gui` auto-launch a browser? Only when stdout is a real
/// terminal — never inside CI, pipes, or scripted `$( ... )` captures — and
/// the user has not opted out via VIKAS_NO_OPEN=1.
pub fn wants_browser() -> bool {
    std::env::var_os("VIKAS_NO_OPEN").is_none()
        && std::io::IsTerminal::is_terminal(&std::io::stdout())
}

/// Result of one bridge attempt. The poll-with-deadline loop below is the
/// whole point: `gio open` on a dbus-less headless box HANGS forever, and
/// `vikas gui` must never block the user's terminal. Each bridge gets a
/// 2-second window: exit 0 -> opened; exit non-zero -> next bridge; still
/// running at the deadline -> the browser is being launched -> success.
enum Bridge {
    ExitedOk,
    ExitedErr,
    StillRunning,
    SpawnFailed,
}

fn spawn_bridge(program: &str, args: &[&str]) -> Bridge {
    let child = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(_) => return Bridge::SpawnFailed,
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(st)) => {
                return if st.success() { Bridge::ExitedOk } else { Bridge::ExitedErr };
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    // still alive after 2s: the browser handoff has either
                    // long finished or this bridge is stuck (e.g. gio waiting
                    // on a dbus-less box). Either way we must never leak a
                    // hung process or hold the user's terminal: kill & reap.
                    let _ = child.kill();
                    let _ = child.wait();
                    return Bridge::StillRunning;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(_) => return Bridge::SpawnFailed,
        }
    }
}

/// Bridge won (exit 0, or still busy launching at the 2s deadline).
fn bridge_ok(b: Bridge) -> bool {
    matches!(b, Bridge::ExitedOk | Bridge::StillRunning)
}

pub fn open_in_browser(path: &std::path::Path) -> BrowserOpen {
    if !path.is_file() {
        return BrowserOpen::NoBridge;
    }
    let here = path.display().to_string();

    #[cfg(target_os = "linux")]
    {
        // WSL? Then the browser lives on the Windows side — hand the file
        // over through the interop bridges (path translated via wslpath -w).
        let is_wsl = std::fs::read_to_string("/proc/version")
            .map(|v| v.to_ascii_lowercase().contains("microsoft"))
            .unwrap_or(false);
        if is_wsl {
            let mut winp = here.clone();
            if let Ok(o) = std::process::Command::new("wslpath").arg("-w").arg(&here).output() {
                if o.status.success() {
                    let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !t.is_empty() {
                        winp = t;
                    }
                }
            }
            if bridge_ok(spawn_bridge("wslview", &[&here])) {
                return BrowserOpen::Opened("wslview".into());
            }
            if bridge_ok(spawn_bridge(
                "powershell.exe",
                &["-NoProfile", "Start-Process", "-FilePath", &winp],
            )) {
                return BrowserOpen::Opened("powershell.exe Start-Process".into());
            }
            if bridge_ok(spawn_bridge("cmd.exe", &["/C", "start", "", &winp])) {
                return BrowserOpen::Opened("cmd.exe start".into());
            }
            // explorer.exe always exits 1, even on success — accept any spawn.
            if !matches!(spawn_bridge("explorer.exe", &[&winp]), Bridge::SpawnFailed) {
                return BrowserOpen::Opened("explorer.exe".into());
            }
            return BrowserOpen::NoBridge;
        }
        if bridge_ok(spawn_bridge("xdg-open", &[&here])) {
            return BrowserOpen::Opened("xdg-open".into());
        }
        if bridge_ok(spawn_bridge("gio", &["open", &here])) {
            return BrowserOpen::Opened("gio open".into());
        }
        return BrowserOpen::NoBridge;
    }

    #[cfg(target_os = "macos")]
    {
        if bridge_ok(spawn_bridge("open", &[&here])) {
            return BrowserOpen::Opened("open".into());
        }
        return BrowserOpen::NoBridge;
    }

    #[cfg(target_os = "windows")]
    {
        if bridge_ok(spawn_bridge("cmd", &["/C", "start", "", &here])) {
            return BrowserOpen::Opened("cmd start".into());
        }
        if !matches!(spawn_bridge("explorer", &[&here]), Bridge::SpawnFailed) {
            return BrowserOpen::Opened("explorer".into());
        }
        BrowserOpen::NoBridge
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        BrowserOpen::NoBridge
    }
}

// ============================================================================
// TYPO-HELP — "did you mean?" suggestions for unknown subcommands.
// Classic Levenshtein distance over the known command table, so a friend
// typing `vikas gui`-adjacent slips (`vikas gu`, `vikas runn`, `vikas gum`)
// gets pointed at the right door instead of a dead end.
// ============================================================================
pub const KNOWN_CMDS: &[&str] = &[
    "run", "quickrun", "watch", "latch", "help", "version", "help-formats", "kdb", "trace",
    "xtrace", "cdc", "fsm", "sva", "loops", "recommend", "why", "doctor", "schematic",
    "coverage", "tui", "gui", "summary", "annotate", "find", "expr", "diff", "decode",
    "disasm", "spectrum", "perf", "radix", "memdump", "export-doc", "export-tb",
    "info-formats", "convert", "anomaly", "delta", "cycles", "physics", "energy", "sweep",
    "mutant", "power", "signoff", "cosim", "autogen-tb", "taint", "pbit", "hdc", "egraph",
    "edit", "fork", "shm", "topo", "session", "ci", "wcp", "mcp", "gate-compare",
];

/// Classic edit distance (Levenshtein), DP over two rolling rows.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut cur = vec![0usize; m + 1];
    for i in 1..=n {
        cur[0] = i;
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[m]
}

/// Closest known command within edit distance 2, if any ("gum" -> "gui").
pub fn suggest_command(typo: &str) -> Option<&'static str> {
    let typo = typo.to_ascii_lowercase();
    let mut best: Option<(usize, &'static str)> = None;
    for &k in KNOWN_CMDS {
        let d = levenshtein(&typo, k);
        if d <= 2 && best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, k));
        }
    }
    best.map(|(_, k)| k)
}
