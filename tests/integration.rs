use drift::config::Config;
use drift::dialect::Dialect;
use drift::fixer::fix;
use drift::parse::parse;
use drift::rules::{Registry, Severity};

fn read_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{name}");
    std::fs::read_to_string(path).expect("fixture missing")
}

#[test]
fn postgres_fixture_flags_many_things() {
    let src = read_fixture("postgres.sql");
    let parsed = parse(&src, Dialect::Postgres);
    let cfg = Config::default();
    let registry = Registry::new();
    let viols = registry.run(&parsed, &cfg);
    assert!(!viols.is_empty(), "expected violations in postgres.sql");
    // at least one error: missing WHERE on UPDATE or DELETE
    let errs = viols
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    assert!(errs >= 1, "no error-level violations found");
}

#[test]
fn mysql_fixture_parses() {
    let src = read_fixture("mysql.sql");
    let parsed = parse(&src, Dialect::MySql);
    assert!(
        parsed.parse_error.is_none(),
        "parse error: {:?}",
        parsed.parse_error
    );
}

#[test]
fn sqlite_fixture_parses() {
    let src = read_fixture("sqlite.sql");
    let parsed = parse(&src, Dialect::Sqlite);
    assert!(parsed.parse_error.is_none());
}

#[test]
fn ansi_fixture_parses() {
    let src = read_fixture("ansi.sql");
    let parsed = parse(&src, Dialect::Ansi);
    assert!(parsed.parse_error.is_none());
}

#[test]
fn fix_is_idempotent() {
    let src = "select id from users";
    let cfg = Config::default();
    let (pass1, _) = fix(src, Dialect::Postgres, &cfg);
    let (pass2, _) = fix(&pass1, Dialect::Postgres, &cfg);
    assert_eq!(pass1, pass2, "fix should be idempotent");
}

#[test]
fn fix_adds_semicolon_and_newline() {
    let src = "SELECT 1";
    let cfg = Config::default();
    let (out, _) = fix(src, Dialect::Postgres, &cfg);
    assert!(out.ends_with(";\n"), "output was: {:?}", out);
}

#[test]
fn fix_uppercases_keywords() {
    let src = "select 1;\n";
    let cfg = Config::default();
    let (out, stats) = fix(src, Dialect::Postgres, &cfg);
    assert!(out.contains("SELECT"), "got: {out}");
    assert!(stats.keyword_case >= 1);
}

#[test]
fn registry_has_many_rules() {
    let r = Registry::new();
    assert!(
        r.rules().len() >= 50,
        "expected >=50 rules, got {}",
        r.rules().len()
    );
}

#[test]
fn all_rule_ids_are_unique() {
    let r = Registry::new();
    let mut ids: Vec<_> = r.rules().iter().map(|r| r.id()).collect();
    ids.sort();
    let n = ids.len();
    ids.dedup();
    assert_eq!(n, ids.len(), "duplicate rule id");
}

#[test]
fn all_rule_ids_follow_namespace() {
    let r = Registry::new();
    for rl in r.rules() {
        assert!(rl.id().starts_with("drift."), "bad id: {}", rl.id());
        let parts: Vec<_> = rl.id().split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "expected drift.<category>.<name>: {}",
            rl.id()
        );
    }
}

#[test]
fn explain_works_on_every_rule() {
    let r = Registry::new();
    for rl in r.rules() {
        assert!(!rl.name().is_empty());
        assert!(
            !rl.description().is_empty(),
            "rule {} has empty description",
            rl.id()
        );
    }
}
