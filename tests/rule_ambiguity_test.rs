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
fn mixed_bool_case_flagged() {
    let ids = lint("SELECT TRUE, false FROM t;\n");
    assert!(ids.iter().any(|i| i == "drift.ambiguity.mixed-bool"));
}

#[test]
fn mixed_bool_variant() {
    let ids = lint("SELECT TRUE FROM t UNION ALL SELECT false FROM t2;\n");
    assert!(ids.iter().any(|i| i == "drift.ambiguity.mixed-bool"));
}
