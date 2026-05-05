//! report formats: pretty (default), json, checkstyle, compact, sarif.

use crate::rules::{Severity, Violation};
use colored::Colorize;
use serde::Serialize;
use std::fmt::Write;

#[derive(Copy, Clone, Debug)]
pub enum Format {
    Pretty,
    Json,
    Checkstyle,
    Compact,
    Sarif,
}

impl Format {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pretty" => Some(Self::Pretty),
            "json" => Some(Self::Json),
            "checkstyle" => Some(Self::Checkstyle),
            "compact" => Some(Self::Compact),
            "sarif" => Some(Self::Sarif),
            _ => None,
        }
    }
}

pub struct FileReport<'a> {
    pub path: &'a str,
    pub source: &'a str,
    pub violations: &'a [Violation],
}

pub fn render(reports: &[FileReport], fmt: Format, use_color: bool) -> String {
    match fmt {
        Format::Pretty => render_pretty(reports, use_color),
        Format::Compact => render_compact(reports),
        Format::Json => render_json(reports),
        Format::Checkstyle => render_checkstyle(reports),
        Format::Sarif => render_sarif(reports),
    }
}

fn sev_label(sev: Severity, color: bool) -> String {
    let label = sev.as_str();
    if !color {
        return label.into();
    }
    match sev {
        Severity::Error => label.red().bold().to_string(),
        Severity::Warning => label.yellow().bold().to_string(),
        Severity::Info => label.blue().to_string(),
        Severity::Off => label.into(),
    }
}

fn render_pretty(reports: &[FileReport], color: bool) -> String {
    let mut out = String::new();
    for r in reports {
        for v in r.violations {
            let loc = format!("{}:{}:{}", r.path, v.line, v.col);
            let loc_s = if color { loc.cyan().to_string() } else { loc };
            let _ = writeln!(
                out,
                "{} {} [{}] {}",
                loc_s,
                sev_label(v.severity, color),
                v.rule_id,
                v.message,
            );
        }
    }
    out
}

fn render_compact(reports: &[FileReport]) -> String {
    let mut out = String::new();
    for r in reports {
        for v in r.violations {
            let _ = writeln!(
                out,
                "{}:{}:{}: {} {} {}",
                r.path,
                v.line,
                v.col,
                v.severity.as_str(),
                v.rule_id,
                v.message
            );
        }
    }
    out
}

#[derive(Serialize)]
struct JsonViolation<'a> {
    file: &'a str,
    rule: &'a str,
    severity: &'a str,
    line: usize,
    col: usize,
    message: &'a str,
}

fn render_json(reports: &[FileReport]) -> String {
    let mut all: Vec<JsonViolation> = Vec::new();
    for r in reports {
        for v in r.violations {
            all.push(JsonViolation {
                file: r.path,
                rule: v.rule_id,
                severity: v.severity.as_str(),
                line: v.line,
                col: v.col,
                message: &v.message,
            });
        }
    }
    serde_json::to_string_pretty(&all).unwrap_or_else(|_| "[]".into())
}

fn render_checkstyle(reports: &[FileReport]) -> String {
    let mut out =
        String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<checkstyle version=\"4.3\">\n");
    for r in reports {
        let _ = writeln!(out, "  <file name=\"{}\">", xml_escape(r.path));
        for v in r.violations {
            let _ = writeln!(
                out,
                "    <error line=\"{}\" column=\"{}\" severity=\"{}\" message=\"{}\" source=\"{}\"/>",
                v.line,
                v.col,
                v.severity.as_str(),
                xml_escape(&v.message),
                v.rule_id,
            );
        }
        let _ = writeln!(out, "  </file>");
    }
    out.push_str("</checkstyle>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// SARIF 2.1.0 — github code scanning ingests this directly, surfacing each
// violation as an inline annotation on the pull request. spec:
// https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html
fn render_sarif(reports: &[FileReport]) -> String {
    use std::collections::BTreeSet;

    let mut rule_ids: BTreeSet<&'static str> = BTreeSet::new();
    for r in reports {
        for v in r.violations {
            rule_ids.insert(v.rule_id);
        }
    }

    let rules_json: Vec<serde_json::Value> = rule_ids
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "name": id,
                "shortDescription": { "text": *id },
                "helpUri": format!("https://github.com/f4rkh4d/drift/blob/main/docs/rules/{id}.md"),
            })
        })
        .collect();

    let results_json: Vec<serde_json::Value> = reports
        .iter()
        .flat_map(|r| {
            r.violations.iter().map(move |v| {
                serde_json::json!({
                    "ruleId": v.rule_id,
                    "level": sarif_level(v.severity),
                    "message": { "text": v.message },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": r.path },
                            "region": {
                                "startLine": v.line,
                                "startColumn": v.col,
                            }
                        }
                    }]
                })
            })
        })
        .collect();

    let doc = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "drift",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/f4rkh4d/drift",
                    "rules": rules_json,
                }
            },
            "results": results_json,
        }]
    });

    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into())
}

fn sarif_level(sev: Severity) -> &'static str {
    // sarif vocabulary: error | warning | note | none.
    match sev {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
        Severity::Off => "none",
    }
}

// summary line: "X errors, Y warnings, Z infos in N files in T ms".
// printed to stderr by the cli at the end of `drift check` for human formats,
// suppressed for json / sarif / checkstyle so machine consumers stay clean.
pub fn summary_line(reports: &[FileReport], elapsed_ms: u128) -> String {
    let (mut e, mut w, mut i) = (0usize, 0usize, 0usize);
    for r in reports {
        for v in r.violations {
            match v.severity {
                Severity::Error => e += 1,
                Severity::Warning => w += 1,
                Severity::Info => i += 1,
                Severity::Off => {}
            }
        }
    }
    format!(
        "checked {} file{} in {} ms: {} error{}, {} warning{}, {} info",
        reports.len(),
        if reports.len() == 1 { "" } else { "s" },
        elapsed_ms,
        e,
        if e == 1 { "" } else { "s" },
        w,
        if w == 1 { "" } else { "s" },
        i,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Severity, Violation};

    fn v(rule: &'static str, sev: Severity, msg: &str, line: usize, col: usize) -> Violation {
        Violation {
            rule_id: rule,
            severity: sev,
            message: msg.into(),
            line,
            col,
            span: None,
            fix: None,
        }
    }

    #[test]
    fn sarif_is_valid_json_and_carries_rules() {
        let viols = vec![
            v(
                "drift.correctness.null-equality",
                Severity::Error,
                "= NULL",
                12,
                1,
            ),
            v(
                "drift.performance.select-star",
                Severity::Warning,
                "use cols",
                5,
                8,
            ),
        ];
        let r = FileReport {
            path: "migrations/0042.sql",
            source: "",
            violations: &viols,
        };
        let out = render(&[r], Format::Sarif, false);
        let val: serde_json::Value = serde_json::from_str(&out).expect("sarif must be valid json");
        assert_eq!(val["version"], "2.1.0");
        assert_eq!(val["runs"][0]["tool"]["driver"]["name"], "drift");
        let rules = val["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 2, "two distinct rule ids");
        let results = val["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "warning");
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["region"]["startLine"],
            12
        );
    }

    #[test]
    fn sarif_severity_mapping() {
        assert_eq!(sarif_level(Severity::Error), "error");
        assert_eq!(sarif_level(Severity::Warning), "warning");
        assert_eq!(sarif_level(Severity::Info), "note");
        assert_eq!(sarif_level(Severity::Off), "none");
    }

    #[test]
    fn summary_line_pluralization() {
        let one = vec![v("r", Severity::Error, "", 1, 1)];
        let r1 = FileReport {
            path: "a.sql",
            source: "",
            violations: &one,
        };
        let s = summary_line(&[r1], 12);
        assert!(s.contains("1 file in"));
        assert!(s.contains("1 error,"));
    }
}
