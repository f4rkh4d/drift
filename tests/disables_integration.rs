//! end-to-end: `-- drift:disable` line comments suppress violations.
//!
//! the `-- drift:disable[-next] RULE_ID` mechanism only filters violations
//! whose reported `line` matches the disable. a handful of pre-existing
//! rules hard-code `line: 1` for every violation; they will not honour
//! disables until that line tracking is fixed. the cases below use
//! `drift.correctness.null-equality`, which uses accurate token-level lines.

use std::io::Write;
use std::process::{Command, Stdio};

fn drift_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_drift"))
}

fn run_stdin(input: &str, args: &[&str]) -> (i32, String) {
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
    )
}

#[test]
fn disable_next_silences_one_line() {
    let sql = "\
-- drift:disable-next drift.correctness.null-equality
SELECT * FROM users WHERE x = NULL;
SELECT * FROM users WHERE x = NULL;
";
    let (_code, stdout) = run_stdin(
        sql,
        &["check", "--dialect", "postgres", "--fail-on", "never"],
    );
    let null_eq_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("drift.correctness.null-equality"))
        .collect();
    // line 2 is suppressed; line 3 still surfaces.
    assert_eq!(null_eq_lines.len(), 1, "got: {null_eq_lines:?}");
    assert!(null_eq_lines[0].contains(":3:"), "{null_eq_lines:?}");
}

#[test]
fn same_line_disable_silences_only_that_line() {
    let sql = "\
SELECT * FROM users WHERE x = NULL; -- drift:disable drift.correctness.null-equality
SELECT * FROM users WHERE x = NULL;
";
    let (_, stdout) = run_stdin(
        sql,
        &["check", "--dialect", "postgres", "--fail-on", "never"],
    );
    let null_eq_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("drift.correctness.null-equality"))
        .collect();
    assert_eq!(null_eq_lines.len(), 1);
    assert!(null_eq_lines[0].contains(":2:"));
}

#[test]
fn empty_rule_list_disables_everything_on_target_line() {
    let sql = "\
-- drift:disable-next
SELECT * FROM users WHERE x = NULL;
SELECT * FROM users WHERE x = NULL;
";
    let (_, stdout) = run_stdin(
        sql,
        &["check", "--dialect", "postgres", "--fail-on", "never"],
    );
    let null_eq_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("drift.correctness.null-equality"))
        .collect();
    // line 2's null-equality is silenced (and any other rule on line 2 too);
    // line 3 still flagged.
    assert_eq!(null_eq_lines.len(), 1);
    assert!(null_eq_lines[0].contains(":3:"));
}

#[test]
fn comment_inside_string_does_not_disable() {
    let sql = "\
SELECT 'oops -- drift:disable drift.correctness.null-equality' AS s, x FROM t WHERE x = NULL;
";
    let (_, stdout) = run_stdin(
        sql,
        &["check", "--dialect", "postgres", "--fail-on", "never"],
    );
    assert!(
        stdout.contains("drift.correctness.null-equality"),
        "directive inside a string literal must not silence the rule, got: {stdout:?}"
    );
}
