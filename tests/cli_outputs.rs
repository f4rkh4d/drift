//! end-to-end tests for cli output formats and exit codes.
//!
//! covers the contract surface that the github action and pre-commit hooks
//! depend on: --format sarif, --fail-on threshold, summary on stderr.

use std::io::Write;
use std::process::{Command, Stdio};

fn drift_bin() -> std::path::PathBuf {
    // cargo defines CARGO_BIN_EXE_<bin name> for every binary in the package, set
    // to the path of the freshly compiled binary. it is the canonical way for an
    // integration test to find its own crate's executable.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_drift"))
}

fn run_drift_stdin(input: &str, args: &[&str]) -> (i32, String, String) {
    let mut cmd = Command::new(drift_bin())
        .args(args)
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn drift");
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = cmd.wait_with_output().expect("wait drift");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn sarif_output_is_valid_json_with_results() {
    let sql = "SELECT * FROM users WHERE x = NULL;\n";
    let (_code, stdout, _stderr) =
        run_drift_stdin(sql, &["check", "--format", "sarif", "--dialect", "postgres"]);
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("sarif stdout should parse as json");
    assert_eq!(v["version"], "2.1.0");
    assert_eq!(v["runs"][0]["tool"]["driver"]["name"], "drift");
    let results = v["runs"][0]["results"]
        .as_array()
        .expect("results array present");
    assert!(!results.is_empty(), "SELECT * + = NULL should produce findings");
    // every result must carry a level vocabulary string and a physical location.
    for r in results {
        let level = r["level"].as_str().unwrap();
        assert!(matches!(level, "error" | "warning" | "note" | "none"));
        assert!(r["locations"][0]["physicalLocation"]["region"]["startLine"].is_number());
    }
}

#[test]
fn fail_on_threshold_controls_exit_code() {
    // a select-star is a warning; a missing-where-update is an error; a comparison
    // to null is an error. so this fixture has both warnings and errors.
    let sql = "SELECT * FROM users WHERE x = NULL;\n";

    // default = "error"; should fail.
    let (code, _, _) = run_drift_stdin(sql, &["check", "--dialect", "postgres"]);
    assert_eq!(code, 1, "default fail-on=error and we have errors");

    // never = no gate, exit code is 0 even with errors.
    let (code, _, _) = run_drift_stdin(
        sql,
        &["check", "--dialect", "postgres", "--fail-on", "never"],
    );
    assert_eq!(code, 0, "fail-on=never always exits 0");

    // info threshold also fails on errors and warnings.
    let (code, _, _) = run_drift_stdin(
        sql,
        &["check", "--dialect", "postgres", "--fail-on", "info"],
    );
    assert_eq!(code, 1);
}

#[test]
fn fail_on_invalid_value_rejected_by_clap() {
    let (code, _, stderr) = run_drift_stdin(
        "SELECT 1;\n",
        &["check", "--dialect", "postgres", "--fail-on", "bogus"],
    );
    // clap rejects unknown values with exit code 2 and writes a usage error.
    assert_eq!(code, 2);
    assert!(stderr.contains("invalid value") || stderr.contains("--fail-on"));
}

#[test]
fn summary_appears_on_stderr_for_pretty_format() {
    let sql = "SELECT * FROM users WHERE x = NULL;\n";
    let (_code, _stdout, stderr) = run_drift_stdin(sql, &["check", "--dialect", "postgres"]);
    assert!(
        stderr.contains("checked"),
        "summary should land on stderr, got: {stderr:?}"
    );
    assert!(stderr.contains("error"));
}

#[test]
fn summary_suppressed_for_machine_formats() {
    let sql = "SELECT * FROM users WHERE x = NULL;\n";
    let (_code, _stdout, stderr) =
        run_drift_stdin(sql, &["check", "--format", "json", "--dialect", "postgres"]);
    assert!(
        !stderr.contains("checked"),
        "json output should keep stderr clean for piping; got: {stderr:?}"
    );
    let (_code, _stdout, stderr) =
        run_drift_stdin(sql, &["check", "--format", "sarif", "--dialect", "postgres"]);
    assert!(!stderr.contains("checked"));
}
