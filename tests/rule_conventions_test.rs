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
fn camelcase_table_flagged() {
    let ids = lint("CREATE TABLE Users (id int);\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.conventions.snake-case-tables"));
}

#[test]
fn singular_table_flagged() {
    let ids = lint("CREATE TABLE user (id int);\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.conventions.plural-table-name"));
}

#[test]
fn uppercase_column_flagged() {
    let ids = lint("CREATE TABLE users (Id int, Email text);\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.conventions.lowercase-columns"));
}

#[test]
fn index_naming_flagged() {
    let ids = lint("CREATE INDEX users_email ON users(email);\n");
    assert!(ids.iter().any(|i| i == "drift.conventions.index-naming"));
}
