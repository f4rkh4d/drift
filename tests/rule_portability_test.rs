use drift::config::Config;
use drift::dialect::Dialect;
use drift::parse::parse;
use drift::rules::Registry;

fn lint_as(src: &str, d: Dialect) -> Vec<String> {
    let parsed = parse(src, d);
    let cfg = Config::default();
    let r = Registry::new();
    r.run(&parsed, &cfg)
        .into_iter()
        .map(|v| v.rule_id.to_string())
        .collect()
}

#[test]
fn backtick_in_ansi_flagged() {
    let ids = lint_as("SELECT `a` FROM `t`;\n", Dialect::Ansi);
    assert!(ids.iter().any(|i| i == "drift.portability.backtick-quote"));
}

#[test]
fn on_duplicate_in_postgres_flagged() {
    let ids = lint_as(
        "INSERT INTO t (a) VALUES (1) ON DUPLICATE KEY UPDATE a = a + 1;\n",
        Dialect::Postgres,
    );
    assert!(ids
        .iter()
        .any(|i| i == "drift.portability.on-duplicate-key"));
}

#[test]
fn select_top_flagged_everywhere() {
    let ids = lint_as("SELECT TOP 10 id FROM t;\n", Dialect::Postgres);
    assert!(ids.iter().any(|i| i == "drift.portability.top-vs-limit"));
}

#[test]
fn non_ansi_type_flagged_in_ansi() {
    let ids = lint_as("CREATE TABLE t (id SERIAL);\n", Dialect::Ansi);
    assert!(ids
        .iter()
        .any(|i| i == "drift.portability.non-standard-type"));
}
