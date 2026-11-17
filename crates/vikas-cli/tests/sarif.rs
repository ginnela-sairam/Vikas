use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn run_sarif(input: &str) -> (std::process::ExitStatus, Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(input);
    let output = Command::new(env!("CARGO_BIN_EXE_vikas"))
        .arg("latch")
        .arg("--sarif")
        .arg(path)
        .output()
        .expect("run vikas latch --sarif");
    let report: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("SARIF stdout was not JSON: {error}; stderr: {}", String::from_utf8_lossy(&output.stderr)));
    (output.status, report)
}

fn assert_sarif_envelope(report: &Value) {
    assert_eq!(report["version"], "2.1.0");
    assert!(report["$schema"].as_str().unwrap().contains("sarif-schema-2.1.0.json"));
    assert_eq!(report["runs"][0]["tool"]["driver"]["name"], "VIKAS Latch Signoff");
    assert!(report["runs"][0]["results"].is_array());
}

#[test]
fn sarif_clean_design_has_no_results() {
    let (status, report) = run_sarif("tests/fixtures/latch_clean.sv");
    assert!(status.success());
    assert_sarif_envelope(&report);
    assert_eq!(report["runs"][0]["results"].as_array().unwrap().len(), 0);
}

#[test]
fn sarif_latch_finding_has_rule_message_and_location() {
    let (status, report) = run_sarif("../../examples/demo_latch_trap.sv");
    assert_eq!(status.code(), Some(2));
    assert_sarif_envelope(&report);
    let results = report["runs"][0]["results"].as_array().unwrap();
    assert!(!results.is_empty());
    let mut lines = results
        .iter()
        .map(|result| {
        assert_eq!(result["ruleId"], "VIKAS-LATCH-001");
        assert_eq!(result["level"], "warning");
        assert!(result["message"]["text"].as_str().unwrap().contains("latch"));
        let location = &result["locations"][0]["physicalLocation"];
        assert!(location["artifactLocation"]["uri"].as_str().is_some());
            location["region"]["startLine"].as_u64().unwrap()
        })
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(lines, vec![11, 16]);
}

#[test]
fn vcd_bench_converts_the_checked_in_riscv_fixture() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/riscv_core/rv32i_core_trace.vcd");
    let output = std::env::temp_dir().join(format!(
        "vikas-cli-riscv-{}.vwm",
        std::process::id()
    ));
    let result = Command::new(env!("CARGO_BIN_EXE_vikas"))
        .arg("vcd-bench")
        .arg(&input)
        .arg(&output)
        .output()
        .expect("run vikas vcd-bench");

    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("signals"));
    assert!(stdout.contains("MB/s"));
    let converted = vikas_wave::WaveDump::load(&output).expect("read converted .vwm");
    assert_eq!(converted.signals.len(), 6);
    assert!(!converted.changes.is_empty());
    std::fs::remove_file(output).unwrap();
}
