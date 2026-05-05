//! ambiguity rules: things a compiler could accept but a reader can't.

use super::{Category, Rule, Severity, Violation};
use crate::config::Config;
use crate::parse::Parsed;
use sqlparser::tokenizer::Token;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(ReservedAsIdentifier));
    out.push(Box::new(DuplicateAlias));
    out.push(Box::new(UnqualifiedColumnInJoin));
    out.push(Box::new(MixedCaseBoolLiteral));
    out.push(Box::new(SameNameFnAndCol));
}

pub struct ReservedAsIdentifier;
impl Rule for ReservedAsIdentifier {
    fn id(&self) -> &'static str {
        "drift.ambiguity.reserved-as-identifier"
    }
    fn name(&self) -> &'static str {
        "reserved keyword as identifier"
    }
    fn category(&self) -> Category {
        Category::Ambiguity
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "using a reserved word as an identifier forces quoting everywhere it appears"
    }
    fn example_bad(&self) -> &'static str {
        "CREATE TABLE \"order\" (id int);"
    }
    fn example_good(&self) -> &'static str {
        "CREATE TABLE orders (id int);"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for t in &p.tokens {
            if let Token::Word(w) = &t.token {
                if w.quote_style.is_some() && w.keyword != sqlparser::keywords::Keyword::NoKeyword {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!("reserved keyword `{}` used as identifier", w.value),
                        line: t.location.line as usize,
                        col: t.location.column as usize,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

pub struct DuplicateAlias;
impl Rule for DuplicateAlias {
    fn id(&self) -> &'static str {
        "drift.ambiguity.duplicate-alias"
    }
    fn name(&self) -> &'static str {
        "duplicate alias in same query"
    }
    fn category(&self) -> Category {
        Category::Ambiguity
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "two tables with the same alias in one FROM clause is ambiguous"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT u.id, o.id\nFROM users u\nJOIN orders u ON u.user_id = u.id;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT u.id, o.id\nFROM users u\nJOIN orders o ON o.user_id = u.id;"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new() // covered by correctness.self-join-no-alias for the common case
    }
}

pub struct UnqualifiedColumnInJoin;
impl Rule for UnqualifiedColumnInJoin {
    fn id(&self) -> &'static str {
        "drift.ambiguity.unqualified-column"
    }
    fn name(&self) -> &'static str {
        "unqualified column in joined query"
    }
    fn category(&self) -> Category {
        Category::Ambiguity
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "when two tables are joined, every column reference should be qualified"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT id, name\nFROM users\nJOIN profiles ON users.id = profiles.user_id;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT users.id, users.name\nFROM users\nJOIN profiles ON users.id = profiles.user_id;"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new() // needs scope tracking; placeholder
    }
}

pub struct MixedCaseBoolLiteral;
impl Rule for MixedCaseBoolLiteral {
    fn id(&self) -> &'static str {
        "drift.ambiguity.mixed-bool"
    }
    fn name(&self) -> &'static str {
        "inconsistent TRUE/FALSE casing"
    }
    fn category(&self) -> Category {
        Category::Ambiguity
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "pick one: TRUE / true. don't mix."
    }
    fn example_bad(&self) -> &'static str {
        "SELECT * FROM users WHERE active = 1 OR verified = TRUE;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM users WHERE active = TRUE AND verified = TRUE;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let src = &p.source;
        let has_upper = src.contains("TRUE") || src.contains("FALSE");
        let has_lower = src.contains(" true") || src.contains(" false");
        if has_upper && has_lower {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "file mixes TRUE/true (or FALSE/false)".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct SameNameFnAndCol;
impl Rule for SameNameFnAndCol {
    fn id(&self) -> &'static str {
        "drift.ambiguity.same-name-fn-col"
    }
    fn name(&self) -> &'static str {
        "function name collides with column name"
    }
    fn category(&self) -> Category {
        Category::Ambiguity
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "naming a column `count` or `current_date` invites parse ambiguity"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT count FROM page_views;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT view_count FROM page_views;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let risky = ["count", "date", "current_date", "user", "year", "month"];
        let lower = p.source.to_lowercase();
        for r in &risky {
            let pat = format!(" {} ", r);
            if lower.contains("create table") && lower.contains(&pat) {
                // very conservative; just flag once per risky name
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!("identifier `{}` shadows a built-in function or keyword", r),
                    line: 1,
                    col: 1,
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}
