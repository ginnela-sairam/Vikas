//! VIKAS v1 — build script: compiles the C++20 mixed-signal twin with
//! -O3 -march=native -std=c++20 (breakthrough 6).

fn main() {
    // rebuild when this file changes
    println!("cargo:rerun-if-changed=csrc/rk4_mixed_signal.cpp");
    let _ = cc::Build::new()
        .cpp(true)
        .opt_level(3)
        .std("c++20")
        .flag("-march=native")
        .file("csrc/rk4_mixed_signal.cpp")
        .compile("vikas_rk4");
}
