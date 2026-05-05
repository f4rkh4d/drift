//! end-to-end coverage for the baseline workflow:
//!   drift baseline create -> drift check --baseline -> file edits -> exit codes
//!
//! the baseline subsystem has unit tests inside src/baseline.rs; this file
//! drives the full cli to make sure the wiring (subcommand dispatch, suppressed
//! count display, exit code computation) all behaves the way the readme claims.

use std::process::{Command, Stdio};

fn drift_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_drift"))
}

fn run(cwd: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(drift_bin())
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn drift");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn write(p: &std::path::Path, contents: &str) {
    std::fs::write(p, contents).expect("write fixture");
}

#[test]
fn baseline_create_then_check_silences_all_known_violations() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        &root.join("a.sql"),
        "SELECT * FROM users WHERE x = NULL;\nUPDATE users SET active = 0;\n",
    );
    write(&root.join("b.sql"), "SELECT * FROM orders;\n");

    // without baseline, errors exist -> exit 1.
    let (code, _, _) = run(root, &["check", "--dialect", "postgres", "a.sql", "b.sql"]);
    assert_eq!(code, 1, "without baseline, errors should fail the run");

    // create baseline.
    let (code, _, stderr) = run(
        root,
        &[
            "baseline",
            "create",
            "--output",
            "bl.json",
            "a.sql",
            "b.sql",
        ],
    );
    assert_eq!(code, 0);
    assert!(stderr.contains("wrote bl.json"));
    assert!(root.join("bl.json").exists());

    // baseline file is valid json with the recorded counts.
    let bl: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("bl.json")).unwrap()).unwrap();
    assert_eq!(bl["schema"], 1);
    assert!(bl["files"]["a.sql"].is_object());

    // check with baseline: every known violation suppressed -> exit 0.
    let (code, stdout, stderr) = run(
        root,
        &[
            "check",
            "--dialect",
            "postgres",
            "--baseline",
            "bl.json",
            "a.sql",
            "b.sql",
        ],
    );
    assert_eq!(code, 0, "all violations covered by baseline -> clean run");
    assert!(stdout.is_empty(), "stdout should be empty when nothing surfaces, got: {stdout:?}");
    assert!(stderr.contains("suppressed by baseline"));
}

#[test]
fn check_with_baseline_surfaces_only_new_violations() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("legacy.sql"), "SELECT * FROM old_table;\n");

    // baseline records the legacy file's single warning.
    let (code, _, _) = run(
        root,
        &[
            "baseline",
            "create",
            "--output",
            "bl.json",
            "legacy.sql",
        ],
    );
    assert_eq!(code, 0);

    // a brand new file is added that introduces a new violation. it MUST surface.
    write(&root.join("new.sql"), "SELECT * FROM other_table;\n");
    let (code, stdout, stderr) = run(
        root,
        &[
            "check",
            "--dialect",
            "postgres",
            "--baseline",
            "bl.json",
            "--fail-on",
            "warning",
            "legacy.sql",
            "new.sql",
        ],
    );
    assert_eq!(code, 1, "new file's warning must reach --fail-on warning");
    assert!(stdout.contains("new.sql"));
    assert!(!stdout.contains("legacy.sql"), "legacy violations are suppressed");
    assert!(stderr.contains("1 warning"));
    assert!(stderr.contains("suppressed by baseline"));
}

#[test]
fn baseline_show_reports_counts() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        &root.join("a.sql"),
        "SELECT * FROM users WHERE x = NULL;\n",
    );

    let (_, _, _) = run(
        root,
        &["baseline", "create", "--output", "bl.json", "a.sql"],
    );
    let (code, stdout, _) = run(root, &["baseline", "show", "--path", "bl.json"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("a.sql"));
    assert!(stdout.contains("drift.correctness.null-equality"));
    assert!(stdout.contains("violations:"));
}

#[test]
fn check_with_baseline_canonical_filename() {
    // .drift-baseline.json is the convention documented in the readme. this
    // test confirms the workflow `baseline create -> commit -> check` works
    // when the file is named that way.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("a.sql"), "SELECT * FROM users;\n");
    let (_, _, _) = run(
        root,
        &[
            "baseline",
            "create",
            "--output",
            ".drift-baseline.json",
            "a.sql",
        ],
    );

    let (code, _stdout, stderr) = run(
        root,
        &[
            "check",
            "--dialect",
            "postgres",
            "--baseline",
            ".drift-baseline.json",
            "a.sql",
        ],
    );
    assert_eq!(code, 0);
    assert!(stderr.contains("suppressed by baseline"));
}

#[test]
fn check_with_missing_baseline_file_errors_clearly() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("a.sql"), "SELECT 1;\n");
    let (code, _, stderr) = run(
        root,
        &[
            "check",
            "--dialect",
            "postgres",
            "--baseline",
            "no-such-file.json",
            "a.sql",
        ],
    );
    assert_eq!(code, 2);
    assert!(stderr.contains("baseline") || stderr.contains("no-such-file"));
}
