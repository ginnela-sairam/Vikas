//! VIKAS v1 — terminal color helpers & shared utils.

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
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
