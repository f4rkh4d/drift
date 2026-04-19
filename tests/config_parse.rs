use drift::config::{Config, KeywordCase, Preset};
use drift::rules::Severity;

#[test]
fn default_is_empty() {
    let c = Config::default();
    assert!(matches!(c.preset, Preset::None));
    assert_eq!(c.format.indent, 2);
}

#[test]
fn short_severity_form() {
    let toml = r#"
[rules]
"drift.performance.select-star" = "error"
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(
        c.effective_severity("drift.performance.select-star", Severity::Warning),
        Severity::Error
    );
}

#[test]
fn wildcard_severity() {
    let toml = r#"
[rules]
"drift.portability.*" = "off"
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(
        c.effective_severity("drift.portability.anything", Severity::Warning),
        Severity::Off
    );
}

#[test]
fn preset_strict_makes_everything_error() {
    let toml = r#"
[drift]
preset = "strict"
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(
        c.effective_severity("drift.style.line-length", Severity::Info),
        Severity::Error
    );
}

#[test]
fn preset_compat_silences_style() {
    let toml = r#"
[drift]
preset = "compat"
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(
        c.effective_severity("drift.style.keyword-case", Severity::Warning),
        Severity::Off
    );
    assert_eq!(
        c.effective_severity("drift.correctness.missing-where-update", Severity::Error),
        Severity::Error
    );
}

#[test]
fn format_options() {
    let toml = r#"
[format]
indent = 4
max-line = 120
keyword-case = "lower"
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(c.format.indent, 4);
    assert_eq!(c.format.max_line, 120);
    assert!(matches!(c.format.keyword_case, KeywordCase::Lower));
}

#[test]
fn full_form_with_case() {
    let toml = r#"
[rules]
"drift.style.keyword-case" = { severity = "warning", case = "lower" }
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(
        c.effective_severity("drift.style.keyword-case", Severity::Warning),
        Severity::Warning
    );
    assert!(matches!(
        c.rule_case("drift.style.keyword-case"),
        Some(KeywordCase::Lower)
    ));
}
