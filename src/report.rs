//! report formats: pretty (default), json, checkstyle.

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
}

impl Format {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pretty" => Some(Self::Pretty),
            "json" => Some(Self::Json),
            "checkstyle" => Some(Self::Checkstyle),
            "compact" => Some(Self::Compact),
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
