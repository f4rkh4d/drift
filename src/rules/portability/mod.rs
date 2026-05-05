//! portability: vendor features flagged when ansi is expected.

use super::{Category, Rule, Severity, Violation};
use crate::config::Config;
use crate::dialect::Dialect;
use crate::parse::Parsed;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(BacktickQuote));
    out.push(Box::new(DoubleQuoteIdentifier));
    out.push(Box::new(PgLimitOffsetSyntax));
    out.push(Box::new(MysqlOnDupUpdate));
    out.push(Box::new(NonStandardType));
    out.push(Box::new(DialectOnlyFn));
    out.push(Box::new(TopVsLimit));
    out.push(Box::new(RegexOp));
}

fn only_in_ansi(p: &Parsed) -> bool {
    p.dialect == Dialect::Ansi
}

pub struct BacktickQuote;
impl Rule for BacktickQuote {
    fn id(&self) -> &'static str {
        "drift.portability.backtick-quote"
    }
    fn name(&self) -> &'static str {
        "backtick identifier"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "backticks are mysql/bigquery-only; ansi uses double quotes"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT `id`, `name` FROM `users`;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT \"id\", \"name\" FROM \"users\";"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if !only_in_ansi(p) {
            return Vec::new();
        }
        if p.source.contains('`') {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "backtick-quoted identifier is not portable".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct DoubleQuoteIdentifier;
impl Rule for DoubleQuoteIdentifier {
    fn id(&self) -> &'static str {
        "drift.portability.double-quote-ident"
    }
    fn name(&self) -> &'static str {
        "double-quoted identifier"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "double quotes are identifiers in ansi but strings in some mysql configs"
    }
    fn example_bad(&self) -> &'static str {
        "-- in mysql this means string, not identifier:\nSELECT * FROM \"users\";"
    }
    fn example_good(&self) -> &'static str {
        "-- ANSI: double-quoted is identifier; mysql needs backticks or ANSI_QUOTES on:\nSELECT * FROM users;"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct PgLimitOffsetSyntax;
impl Rule for PgLimitOffsetSyntax {
    fn id(&self) -> &'static str {
        "drift.portability.pg-limit-offset"
    }
    fn name(&self) -> &'static str {
        "LIMIT n OFFSET m"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "LIMIT/OFFSET is postgres/mysql; ansi uses FETCH FIRST n ROWS ONLY"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT * FROM events LIMIT 10 OFFSET 100;"
    }
    fn example_good(&self) -> &'static str {
        "-- ANSI form, works on more dialects:\nSELECT * FROM events OFFSET 100 ROWS FETCH FIRST 10 ROWS ONLY;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if p.dialect != Dialect::Ansi {
            return Vec::new();
        }
        if p.source.to_lowercase().contains("limit ") {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "LIMIT is not ansi-standard; use FETCH FIRST n ROWS ONLY".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct MysqlOnDupUpdate;
impl Rule for MysqlOnDupUpdate {
    fn id(&self) -> &'static str {
        "drift.portability.on-duplicate-key"
    }
    fn name(&self) -> &'static str {
        "ON DUPLICATE KEY UPDATE"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "ON DUPLICATE KEY UPDATE is mysql-only; postgres has ON CONFLICT"
    }
    fn example_bad(&self) -> &'static str {
        "INSERT INTO users (id, name) VALUES (1, 'a')\nON DUPLICATE KEY UPDATE name = 'a';"
    }
    fn example_good(&self) -> &'static str {
        "-- ANSI / postgres / sqlite:\nINSERT INTO users (id, name) VALUES (1, 'a')\nON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if p.dialect == Dialect::MySql {
            return Vec::new();
        }
        if p.source.to_lowercase().contains("on duplicate key update") {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "ON DUPLICATE KEY UPDATE is mysql-only".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct NonStandardType;
impl Rule for NonStandardType {
    fn id(&self) -> &'static str {
        "drift.portability.non-standard-type"
    }
    fn name(&self) -> &'static str {
        "non-standard type"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "types like SERIAL, DATETIME, TINYINT are dialect-specific"
    }
    fn example_bad(&self) -> &'static str {
        "CREATE TABLE t (data JSONB);  -- pg-only"
    }
    fn example_good(&self) -> &'static str {
        "CREATE TABLE t (data JSON);  -- portable; pg can still index it"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if !only_in_ansi(p) {
            return Vec::new();
        }
        let lower = p.source.to_lowercase();
        let mut out = Vec::new();
        for t in &["serial", "bigserial", "tinyint", "mediumint", "longtext"] {
            if lower.contains(t) {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!("non-ansi type `{}`", t),
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

pub struct DialectOnlyFn;
impl Rule for DialectOnlyFn {
    fn id(&self) -> &'static str {
        "drift.portability.dialect-fn"
    }
    fn name(&self) -> &'static str {
        "dialect-only function"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "functions like GENERATE_SERIES, IFNULL, IF() are dialect-bound"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT NVL(name, 'unknown') FROM users;  -- oracle"
    }
    fn example_good(&self) -> &'static str {
        "SELECT COALESCE(name, 'unknown') FROM users;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if !only_in_ansi(p) {
            return Vec::new();
        }
        let lower = p.source.to_lowercase();
        let mut out = Vec::new();
        for t in &["generate_series(", "ifnull(", "unix_timestamp("] {
            if lower.contains(t) {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!("non-ansi function `{}`", t.trim_end_matches('(')),
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

pub struct TopVsLimit;
impl Rule for TopVsLimit {
    fn id(&self) -> &'static str {
        "drift.portability.top-vs-limit"
    }
    fn name(&self) -> &'static str {
        "SELECT TOP"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "SELECT TOP is tsql-only; drift doesn't support tsql yet"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT TOP 10 * FROM users;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM users LIMIT 10;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if p.source.to_lowercase().contains("select top") {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "SELECT TOP is not supported by the target dialect".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct RegexOp;
impl Rule for RegexOp {
    fn id(&self) -> &'static str {
        "drift.portability.regex-op"
    }
    fn name(&self) -> &'static str {
        "regex operator"
    }
    fn category(&self) -> Category {
        Category::Portability
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "~ and ~* are postgres-only; mysql uses REGEXP"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT * FROM users WHERE email ~ '^foo';  -- pg-only"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM users WHERE email LIKE 'foo%';\n-- or, if you need real regex, wrap in REGEXP_LIKE etc."
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        if p.dialect == Dialect::Postgres {
            return Vec::new();
        }
        if p.source.contains(" ~ ") || p.source.contains(" ~* ") {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "postgres-style regex operator".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}
