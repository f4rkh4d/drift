use drift::config::Config;
use drift::dialect::Dialect;
use drift::parse::parse;
use drift::rules::Registry;

fn lint(src: &str) -> Vec<String> {
    let parsed = parse(src, Dialect::Postgres);
    let cfg = Config::default();
    let r = Registry::new();
    r.run(&parsed, &cfg)
        .into_iter()
        .map(|v| v.rule_id.to_string())
        .collect()
}

#[test]
fn update_no_where_flagged() {
    let ids = lint("UPDATE users SET last_seen = now();\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.missing-where-update"));
}

#[test]
fn delete_no_where_flagged() {
    let ids = lint("DELETE FROM sessions;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.missing-where-delete"));
}

#[test]
fn equal_null_flagged() {
    let ids = lint("SELECT 1 FROM t WHERE x = NULL;\n");
    assert!(ids.iter().any(|i| i == "drift.correctness.null-equality"));
}

#[test]
fn plain_union_flagged() {
    let ids = lint("SELECT id FROM a UNION SELECT id FROM b;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.union-vs-union-all"));
}

#[test]
fn div_zero_flagged() {
    let ids = lint("SELECT 1 / 0;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.div-zero-literal"));
}

#[test]
fn order_by_ordinal_flagged() {
    let ids = lint("SELECT a, b FROM t ORDER BY 1;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.order-by-ordinal"));
}

#[test]
fn cartesian_flagged() {
    let ids = lint("SELECT 1 FROM a, b;\n");
    assert!(ids.iter().any(|i| i == "drift.correctness.cartesian-join"));
}

#[test]
fn distinct_on_no_order() {
    let ids = lint("SELECT DISTINCT ON (id) id FROM t;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.distinct-on-no-order"));
}

#[test]
fn between_dates_flagged() {
    let ids = lint("SELECT 1 FROM t WHERE d BETWEEN '2025-01-01' AND '2025-01-31';\n");
    assert!(ids.iter().any(|i| i == "drift.correctness.between-on-date"));
}

#[test]
fn implicit_coercion_flagged() {
    let ids = lint("SELECT 1 FROM users WHERE id = '42';\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.implicit-coercion"));
}

#[test]
fn case_without_else_flagged() {
    let ids = lint("SELECT CASE WHEN x = 1 THEN 'a' END FROM t;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.correctness.case-without-else"));
}
