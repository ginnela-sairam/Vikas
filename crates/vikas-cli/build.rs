//! Build-time identity stamping — the backbone of VIKAS self-awareness
//! (breakthrough #25: "never a stale binary again").
//!
//! Every `cargo build` embeds into the binary:
//!   VIKAS_BUILD_HASH — short git hash of the source checkout ("source" when
//!                      built from a git-less snapshot, e.g. a zip)
//!   VIKAS_BUILD_DATE — commit date (RFC-3339) of that checkout
//!   VIKAS_BUILD_SRC  — absolute path of the workspace root it was built from
//!
//! At run time the CLI compares the embedded hash against the live checkout,
//! so a friend who pulled new code but forgot to rebuild gets told exactly
//! that — with the one-line fix — instead of silently running last year's
//! kernel (the #1 support failure observed in the wild).

use std::path::{Path, PathBuf};
use std::process::Command;

fn run_git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").current_dir(dir).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn main() {
    // workspace root = two levels above this crate's manifest dir
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir)
        .to_path_buf();

    let (hash, date) = match run_git(&root, &["rev-parse", "--short", "HEAD"]) {
        Some(h) => {
            let d = run_git(&root, &["log", "-1", "--format=%cI"]).unwrap_or_else(|| "unknown".into());
            (h, d)
        }
        // zip / non-git build: honest about what we know
        None => ("source".into(), "unknown".into()),
    };

    println!("cargo:rustc-env=VIKAS_BUILD_HASH={}", hash);
    println!("cargo:rustc-env=VIKAS_BUILD_DATE={}", date);
    println!("cargo:rustc-env=VIKAS_BUILD_SRC={}", root.display());

    // Re-stamp when the checkout identity moves (git pull / rebase / amend /
    // branch switch) even if no tracked file content changed — HEAD is the
    // identity, and the binary must follow it. `.git/HEAD` is only a symref
    // ("ref: refs/heads/main") that never changes content on a new commit,
    // so the actual ref files must be watched as well (loose ref first,
    // packed-refs as fallback). Proven necessary by the freshness replay:
    // a pull without source changes must still re-stamp the binary.
    let git_dir = root.join(".git");
    for candidate in [
        git_dir.join("HEAD"),
        git_dir.join("refs/heads/main"),
        git_dir.join("refs/heads/master"),
        git_dir.join("packed-refs"),
    ] {
        if candidate.exists() {
            println!("cargo:rerun-if-changed={}", candidate.display());
        }
    }
}
