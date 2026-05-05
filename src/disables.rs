//! per-line rule disable comments. lets users silence a rule on one line
//! (or one statement) without touching the global drift.toml — same idea as
//! eslint's `// eslint-disable-next-line` and clippy's `#[allow(...)]`.
//!
//! supported forms (each is a regular SQL `--` line comment):
//!
//!   -- drift:disable                          all rules, this line if mixed
//!                                             with sql, otherwise next line
//!   -- drift:disable rule.a, rule.b           specific rules
//!   -- drift:disable-next                     all rules, next line
//!   -- drift:disable-next rule.a, rule.b      specific rules, next line
//!
//! a comment that lives on a line that ALSO has SQL applies to that line.
//! a comment that lives on a line by itself applies to the NEXT line. an
//! explicit `-next` always applies to the next line regardless. an empty
//! rule list means "every rule".

use crate::rules::Violation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisableKind {
    /// disable applies to the same source line as the comment
    SameLine,
    /// disable applies to source line = comment_line + 1
    NextLine,
}

#[derive(Debug, Clone)]
pub struct Disable {
    pub kind: DisableKind,
    /// source line number (1-based) the comment was on
    pub line: usize,
    /// rule ids to silence. empty vec = "all rules".
    pub rules: Vec<String>,
}

impl Disable {
    fn target_line(&self) -> usize {
        match self.kind {
            DisableKind::SameLine => self.line,
            DisableKind::NextLine => self.line + 1,
        }
    }

    fn covers(&self, rule_id: &str) -> bool {
        self.rules.is_empty() || self.rules.iter().any(|r| r == rule_id)
    }
}

/// scan a SQL source string for `-- drift:disable[-next] [rules]` comments.
pub fn scan(source: &str) -> Vec<Disable> {
    let mut out = Vec::new();
    for (i, raw) in source.lines().enumerate() {
        let lineno = i + 1;
        // find a `--` that isn't inside a string literal. cheap approximation:
        // walk the line tracking single-quote state. block comments `/* */`
        // are handled below; we treat them like line comments for our purpose.
        let Some(comment_idx) = find_line_comment(raw) else {
            continue;
        };
        let comment_body = raw[comment_idx + 2..].trim_start();
        let before = raw[..comment_idx].trim();

        if let Some(rest) = strip_directive(comment_body, "drift:disable-next") {
            out.push(Disable {
                kind: DisableKind::NextLine,
                line: lineno,
                rules: parse_rule_list(rest),
            });
        } else if let Some(rest) = strip_directive(comment_body, "drift:disable") {
            // a comment ALONE on its line targets the next line (more useful);
            // a comment that follows actual SQL on the same line targets that
            // line.
            let kind = if before.is_empty() {
                DisableKind::NextLine
            } else {
                DisableKind::SameLine
            };
            out.push(Disable {
                kind,
                line: lineno,
                rules: parse_rule_list(rest),
            });
        }
    }
    out
}

/// drop violations covered by a disable directive. preserves order.
pub fn filter_violations(disables: &[Disable], viols: &[Violation]) -> Vec<Violation> {
    if disables.is_empty() {
        return viols.to_vec();
    }
    viols
        .iter()
        .filter(|v| {
            !disables
                .iter()
                .any(|d| d.target_line() == v.line && d.covers(v.rule_id))
        })
        .cloned()
        .collect()
}

// --- internals ---------------------------------------------------------------

/// look for `--` in a SQL line, skipping `--` that appear inside a single-
/// quoted string. crude but covers 99% of real-world SQL. returns the byte
/// offset of the first `-` of the comment, or None.
fn find_line_comment(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i + 1 < bytes.len() {
        let c = bytes[i];
        if c == b'\'' {
            // honor doubled '' as escape inside a string (postgres style)
            if in_str && i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                i += 2;
                continue;
            }
            in_str = !in_str;
        } else if !in_str && c == b'-' && bytes[i + 1] == b'-' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn strip_directive<'a>(body: &'a str, directive: &str) -> Option<&'a str> {
    if !body.starts_with(directive) {
        return None;
    }
    let rest = &body[directive.len()..];
    if rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace()) {
        Some(rest.trim_start())
    } else {
        // matched a longer name like "drift:disable-next" when looking for "drift:disable"
        None
    }
}

fn parse_rule_list(rest: &str) -> Vec<String> {
    let s = rest.trim();
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Severity, Violation};

    fn v(rule: &'static str, line: usize) -> Violation {
        Violation {
            rule_id: rule,
            severity: Severity::Warning,
            message: String::new(),
            line,
            col: 1,
            span: None,
            fix: None,
        }
    }

    #[test]
    fn same_line_disable_silences_just_that_line() {
        let src = "SELECT * FROM users; -- drift:disable drift.performance.select-star\n";
        let d = scan(src);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].kind, DisableKind::SameLine);
        assert_eq!(d[0].line, 1);
        assert_eq!(d[0].rules, vec!["drift.performance.select-star"]);

        let viols = vec![
            v("drift.performance.select-star", 1),
            v("drift.performance.select-star", 2), // unrelated, must survive
        ];
        let kept = filter_violations(&d, &viols);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].line, 2);
    }

    #[test]
    fn alone_on_line_means_next_line() {
        let src = "-- drift:disable drift.style.keyword-case\nselect 1;\n";
        let d = scan(src);
        assert_eq!(d[0].kind, DisableKind::NextLine);
        assert_eq!(d[0].line, 1);
        let viols = vec![v("drift.style.keyword-case", 2)];
        assert!(filter_violations(&d, &viols).is_empty());
    }

    #[test]
    fn explicit_disable_next() {
        let src = "-- drift:disable-next drift.correctness.null-equality\nSELECT * FROM users WHERE x = NULL;\n";
        let d = scan(src);
        assert_eq!(d[0].kind, DisableKind::NextLine);
        let viols = vec![
            v("drift.correctness.null-equality", 2),
            v("drift.correctness.null-equality", 3),
        ];
        let kept = filter_violations(&d, &viols);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].line, 3);
    }

    #[test]
    fn empty_rule_list_means_all_rules() {
        let src = "SELECT * FROM users WHERE x = NULL; -- drift:disable\n";
        let d = scan(src);
        assert!(d[0].rules.is_empty());
        let viols = vec![
            v("drift.performance.select-star", 1),
            v("drift.correctness.null-equality", 1),
        ];
        assert!(filter_violations(&d, &viols).is_empty());
    }

    #[test]
    fn multiple_rules_in_one_directive() {
        let src = "SELECT * FROM users WHERE x=NULL; -- drift:disable drift.performance.select-star, drift.correctness.null-equality\n";
        let d = scan(src);
        assert_eq!(d[0].rules.len(), 2);
        let viols = vec![
            v("drift.performance.select-star", 1),
            v("drift.correctness.null-equality", 1),
            v("drift.style.keyword-case", 1), // not in the list, must survive
        ];
        let kept = filter_violations(&d, &viols);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].rule_id, "drift.style.keyword-case");
    }

    #[test]
    fn comment_inside_string_is_not_a_directive() {
        let src = "SELECT 'oops -- drift:disable rule.x' AS s;\n";
        let d = scan(src);
        assert!(d.is_empty(), "must not parse comment inside string literal");
    }

    #[test]
    fn directive_must_be_isolated_word() {
        // strip_directive must not match e.g. "drift:disablement" as "drift:disable"
        let src = "-- drift:disablement of judgement\n";
        let d = scan(src);
        assert!(d.is_empty());
    }
}
