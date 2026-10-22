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
        if is_wsl() {
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
// SELF-AWARENESS (breakthrough #25) — "never a stale binary again".
// The binary knows which commit it was built from (build.rs stamp) and can
// compare itself against the live checkout at run time. The friend-failure
// this kills: `git pull` brings new code, but the old binary in ~/.local/bin
// silently keeps running — no WSL bridge, no fixes, no hint. Now every
// command can tell the user their binary is stale and exactly how to fix it.
// ============================================================================

/// Short git hash of the checkout this binary was compiled from
/// ("source" for git-less builds, e.g. from a zip).
pub fn build_hash() -> &'static str {
    env!("VIKAS_BUILD_HASH")
}

/// Commit date (RFC-3339) of the checkout this binary was compiled from.
pub fn build_date() -> &'static str {
    env!("VIKAS_BUILD_DATE")
}

/// Absolute path of the workspace root the binary was compiled from.
pub fn build_src() -> &'static str {
    env!("VIKAS_BUILD_SRC")
}

/// Human-readable one-line build identity, e.g. "v1.0.0+c971f2a · 2026-10-20".
pub fn build_stamp() -> String {
    if build_hash() == "source" {
        format!("v1.0.0 source snapshot · {}", build_date())
    } else {
        format!("v1.0.0+{} · {}", build_hash(), build_date())
    }
}

/// Are we running inside Windows Subsystem for Linux? The browser lives on
/// the Windows side there — every launch must bridge out (wslpath + interop).
pub fn is_wsl() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/version")
            .map(|v| v.to_ascii_lowercase().contains("microsoft"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Translate a Linux path to its Windows form (`wslpath -w`) when inside WSL.
pub fn windows_path_if_wsl(path: &std::path::Path) -> Option<String> {
    if !is_wsl() {
        return None;
    }
    let out = std::process::Command::new("wslpath")
        .arg("-w")
        .arg(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// A directory that is unmistakably the VIKAS source repo: a git checkout
/// with a workspace Cargo.toml and a crates/ dir. Guards the stale-detector
/// from ever firing against unrelated git repos (e.g. dotfiles at $HOME).
pub fn looks_like_vikas_repo(dir: &std::path::Path) -> bool {
    dir.join(".git").exists() && dir.join("Cargo.toml").is_file() && dir.join("crates").is_dir()
}

/// Short HEAD hash of the git repo at `dir` (None: not a repo / git absent /
/// git hung — 3s hard deadline so a wedged .git can never stall a command).
pub fn repo_head(dir: &std::path::Path) -> Option<String> {
    let mut child = std::process::Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--short", "HEAD"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(15));
            }
        }
    }
    let mut buf = String::new();
    use std::io::Read;
    child.stdout.take()?.read_to_string(&mut buf).ok()?;
    let s = buf.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Walk up from `start` (or the current directory) looking for a VIKAS
/// source checkout.
pub fn find_repo(start: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    let mut cur = match start {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };
    loop {
        if looks_like_vikas_repo(&cur) {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Verdict of comparing the running binary against the live source tree.
pub enum Freshness {
    /// Binary matches the checkout it was built from (and any cwd checkout).
    Fresh,
    /// Binary came from a git-less snapshot — nothing to compare against.
    Untracked,
    /// The checkout this binary was built from has moved on: STALE.
    Stale { repo: std::path::PathBuf, built: String, now: String },
    /// A *different* VIKAS checkout under the cwd is ahead of the binary.
    Foreign { repo: std::path::PathBuf, built: String, now: String },
}

pub fn check_freshness() -> Freshness {
    let built = build_hash();
    if built == "source" || built == "unknown" {
        return Freshness::Untracked;
    }
    let built_src = std::path::PathBuf::from(build_src());
    // 1) the checkout this binary was actually built from
    if looks_like_vikas_repo(&built_src) {
        if let Some(now) = repo_head(&built_src) {
            if now != built {
                return Freshness::Stale { repo: built_src, built: built.into(), now };
            }
            // home repo matches; still, if the user is *working inside a
            // different* checkout that has moved on, say so (softly).
            if let Some(cwd_repo) = find_repo(None) {
                if cwd_repo != built_src {
                    if let Some(now2) = repo_head(&cwd_repo) {
                        if now2 != built {
                            return Freshness::Foreign { repo: cwd_repo, built: built.into(), now: now2 };
                        }
                    }
                }
            }
            return Freshness::Fresh;
        }
    }
    // 2) binary's home checkout is gone (deleted clone, moved zip source):
    // fall back to whatever checkout the user is standing in.
    if let Some(repo) = find_repo(None) {
        if let Some(now) = repo_head(&repo) {
            if now != built {
                return Freshness::Stale { repo, built: built.into(), now };
            }
            return Freshness::Fresh;
        }
    }
    Freshness::Untracked
}

/// One-shot startup advisory printed at most once per invocation. Talks to
/// real terminals only (stderr) so CI/pipes/JSON stay byte-stable; opt-outs:
///   VIKAS_NO_STALE_CHECK=1   silence entirely
///   VIKAS_FORCE_STALE_CHECK=1 test hook: bypass the tty gate (used by the
///                             command matrix to prove the warning fires)
pub fn warn_stale(args: &[String]) {
    if std::env::var_os("VIKAS_NO_STALE_CHECK").is_some() {
        return;
    }
    if args.iter().any(|a| a == "--json") {
        return;
    }
    let forced = std::env::var_os("VIKAS_FORCE_STALE_CHECK").is_some();
    if !forced && !std::io::IsTerminal::is_terminal(&std::io::stderr()) {
        return;
    }
    match check_freshness() {
        Freshness::Stale { repo, built, now } => {
            eprintln!(
                "{} BINARY IS STALE: built from commit {}, but the checkout at {} is now at {}.",
                c("[VIKAS]", YELLOW),
                built,
                repo.display(),
                now
            );
            eprintln!(
                "{} fix: `vikas update`   (or: cd {} && bash install_linux.sh)",
                c("[VIKAS]", YELLOW),
                repo.display()
            );
        }
        Freshness::Foreign { repo, built, now } => {
            eprintln!(
                "{} note: this checkout ({}) is at {}, but your binary was built from {} — `vikas update` rebuilds from the checkout you are in.",
                c("[VIKAS]", DIM),
                repo.display(),
                now,
                built
            );
        }
        _ => {}
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
    "edit", "fork", "shm", "topo", "session", "ci", "wcp", "mcp", "gate-compare", "update",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_stamp_is_wellformed() {
        // whatever the build environment (git checkout or zip snapshot), the
        // stamp must be a non-empty single line containing the version.
        let s = build_stamp();
        assert!(s.starts_with("v1.0.0"));
        assert!(s.contains('·') || s.contains("snapshot"));
        assert!(!s.contains('\n'));
    }

    #[test]
    fn repo_marker_requires_all_three_signs() {
        let tmp = std::env::temp_dir().join("vikas_ut_marker");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("crates")).unwrap();
        std::fs::write(tmp.join("Cargo.toml"), "[workspace]\n").unwrap();
        // Cargo.toml + crates/ but no .git -> NOT a repo
        assert!(!looks_like_vikas_repo(&tmp));
        // add .git -> now it qualifies
        std::fs::create_dir_all(tmp.join(".git")).unwrap();
        assert!(looks_like_vikas_repo(&tmp));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn repo_head_reads_real_hash() {
        // only meaningful where git exists; skip gracefully elsewhere
        if std::process::Command::new("git").arg("--version").output().is_err() {
            return;
        }
        let tmp = std::env::temp_dir().join("vikas_ut_repo");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let g = |a: &[&str]| {
            std::process::Command::new("git")
                .arg("-c").arg("user.email=t@t")
                .arg("-c").arg("user.name=t")
                .current_dir(&tmp)
                .args(a)
                .output()
        };
        g(&["init", "-q"]).unwrap();
        std::fs::write(tmp.join("f"), "x").unwrap();
        g(&["add", "f"]).unwrap();
        g(&["commit", "-qm", "x"]).unwrap();
        let h = repo_head(&tmp).expect("repo_head on a fresh git repo");
        assert_eq!(h.len(), 7, "short hash, got {h}");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        // a plain directory has no HEAD
        let plain = std::env::temp_dir().join("vikas_ut_plain");
        let _ = std::fs::remove_dir_all(&plain);
        std::fs::create_dir_all(&plain).unwrap();
        assert!(repo_head(&plain).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
        let _ = std::fs::remove_dir_all(&plain);
    }

    #[test]
    fn find_repo_walks_upward() {
        let tmp = std::env::temp_dir().join("vikas_ut_walk");
        let _ = std::fs::remove_dir_all(&tmp);
        let deep = tmp.join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::create_dir_all(tmp.join("crates")).unwrap();
        std::fs::write(tmp.join("Cargo.toml"), "[workspace]\n").unwrap();
        std::fs::create_dir_all(tmp.join(".git")).unwrap();
        let found = find_repo(Some(&deep)).expect("walk-up must find the fake repo");
        assert_eq!(found, tmp);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
