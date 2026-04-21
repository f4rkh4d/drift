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
fn grant_all_flagged() {
    let ids = lint("GRANT ALL ON users TO admin;\n");
    assert!(ids.iter().any(|i| i == "drift.security.grant-all"));
}

#[test]
fn plaintext_password_flagged() {
    let ids = lint("CREATE ROLE bob PASSWORD 'hunter2';\n");
    assert!(ids.iter().any(|i| i == "drift.security.plaintext-password"));
}

#[test]
fn public_schema_write_flagged() {
    let ids = lint("CREATE TABLE public.logs (id int);\n");
    assert!(ids.iter().any(|i| i == "drift.security.public-schema"));
}

#[test]
fn drop_without_if_exists_flagged() {
    let ids = lint("DROP TABLE users;\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.security.drop-without-if-exists"));
}

#[test]
fn select_into_outfile_flagged() {
    let ids = lint("SELECT name FROM users INTO OUTFILE '/tmp/x.txt';\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.security.select-into-outfile"));
}

#[test]
fn select_into_dumpfile_flagged() {
    let ids = lint("SELECT data FROM blobs INTO DUMPFILE '/tmp/x.bin';\n");
    assert!(ids
        .iter()
        .any(|i| i == "drift.security.select-into-outfile"));
}

#[test]
fn plain_select_not_flagged_as_outfile() {
    let ids = lint("SELECT id, name FROM users;\n");
    assert!(!ids
        .iter()
        .any(|i| i == "drift.security.select-into-outfile"));
}
