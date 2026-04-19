use drift::config::Config;
use drift::dialect::Dialect;
use drift::fixer::fix;
use drift::parse::parse;
use drift::rules::Registry;

fn roundtrip(src: &str) {
    let cfg = Config::default();
    let (pass1, _) = fix(src, Dialect::Postgres, &cfg);
    let (pass2, _) = fix(&pass1, Dialect::Postgres, &cfg);
    assert_eq!(pass1, pass2, "not idempotent");
    // output should still parse
    let parsed = parse(&pass1, Dialect::Postgres);
    assert!(
        parsed.parse_error.is_none(),
        "fix produced unparseable sql: {:?}",
        parsed.parse_error
    );
}

#[test]
fn fix_roundtrip_simple_select() {
    roundtrip("select id from users where active = true");
}

#[test]
fn fix_roundtrip_multiple_statements() {
    roundtrip("select 1 ; select 2");
}

#[test]
fn fix_roundtrip_mixed_case() {
    roundtrip("Select Id From Users Where Active = True");
}

#[test]
fn fix_roundtrip_with_trailing_whitespace() {
    roundtrip("select 1   \n  \n");
}

#[test]
fn fix_preserves_identifier_case() {
    let cfg = Config::default();
    let src = "select CustomerId from Orders;\n";
    let (out, _) = fix(src, Dialect::Postgres, &cfg);
    assert!(out.contains("CustomerId"), "identifier got modified: {out}");
    assert!(out.contains("Orders"));
}

#[test]
fn fix_then_lint_clean_on_style_rules() {
    let cfg = Config::default();
    let src = "select id from users";
    let (fixed, _) = fix(src, Dialect::Postgres, &cfg);
    let parsed = parse(&fixed, Dialect::Postgres);
    let r = Registry::new();
    let viols = r.run(&parsed, &cfg);
    // semicolon + keyword case + final newline all resolved
    for v in &viols {
        assert_ne!(
            v.rule_id, "drift.style.keyword-case",
            "case still flagged: {fixed}"
        );
        assert_ne!(v.rule_id, "drift.style.semicolon-terminator");
        assert_ne!(v.rule_id, "drift.style.trailing-newline");
    }
}
