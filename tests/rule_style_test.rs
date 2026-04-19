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
fn lowercase_keywords_flagged() {
    let ids = lint("select 1;\n");
    assert!(ids.iter().any(|i| i == "drift.style.keyword-case"));
}

#[test]
fn trailing_whitespace_flagged() {
    let ids = lint("SELECT 1;   \n");
    assert!(ids.iter().any(|i| i == "drift.style.trailing-whitespace"));
}

#[test]
fn missing_final_newline_flagged() {
    let ids = lint("SELECT 1;");
    assert!(ids.iter().any(|i| i == "drift.style.trailing-newline"));
}

#[test]
fn missing_semicolon_flagged() {
    let ids = lint("SELECT 1\n");
    assert!(ids.iter().any(|i| i == "drift.style.semicolon-terminator"));
}

#[test]
fn double_paren_flagged() {
    let ids = lint("SELECT ((id)) FROM t;\n");
    assert!(ids.iter().any(|i| i == "drift.style.redundant-parens"));
}

#[test]
fn leading_comma_flagged() {
    let ids = lint("SELECT a\n, b\nFROM t;\n");
    assert!(ids.iter().any(|i| i == "drift.style.leading-comma"));
}

#[test]
fn double_blank_line_flagged() {
    let ids = lint("SELECT 1;\n\n\nSELECT 2;\n");
    assert!(ids.iter().any(|i| i == "drift.style.double-blank-line"));
}

#[test]
fn tab_indent_flagged() {
    let ids = lint("\tSELECT 1;\n");
    assert!(ids.iter().any(|i| i == "drift.style.tab-indent"));
}

#[test]
fn empty_file_flagged() {
    let ids = lint("   \n\n");
    assert!(ids.iter().any(|i| i == "drift.style.empty-file"));
}
