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
fn select_star_flagged() {
    let ids = lint("SELECT * FROM users;\n");
    assert!(ids.iter().any(|i| i == "drift.performance.select-star"));
}

#[test]
fn leading_wildcard_flagged() {
    let ids = lint("SELECT 1 FROM t WHERE email LIKE '%@ex.com';\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.performance.like-leading-wildcard"));
}

#[test]
fn fn_on_column_flagged() {
    let ids = lint("SELECT 1 FROM t WHERE lower(email) = 'x';\n");
    assert!(ids.iter().any(|i| i == "drift.performance.fn-on-column"));
}

#[test]
fn order_by_rand_flagged() {
    let ids = lint("SELECT 1 FROM t ORDER BY random();\n");
    assert!(ids.iter().any(|i| i == "drift.performance.order-by-rand"));
}

#[test]
fn offset_paging_flagged() {
    let ids = lint("SELECT 1 FROM t LIMIT 10 OFFSET 5000;\n");
    assert!(ids.iter().any(|i| i == "drift.performance.offset-paging"));
}

#[test]
fn nested_subquery_flagged() {
    let ids = lint("SELECT 1 FROM (SELECT id FROM (SELECT id FROM (SELECT id FROM t) a) b) c;\n");
    assert!(ids.iter().any(|i| i == "drift.performance.nested-subquery"));
}
