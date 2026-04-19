//! security: things that look like a foot-gun for prod.

use super::{Category, Rule, Severity, Violation};
use crate::config::Config;
use crate::parse::Parsed;
use sqlparser::tokenizer::Token;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(GrantAll));
    out.push(Box::new(PublicSchemaWrite));
    out.push(Box::new(PlaintextPasswordLiteral));
    out.push(Box::new(DynamicSqlConcat));
    out.push(Box::new(DropWithoutIfExists));
    out.push(Box::new(TruncateNoCascade));
}

pub struct GrantAll;
impl Rule for GrantAll {
    fn id(&self) -> &'static str {
        "drift.security.grant-all"
    }
    fn name(&self) -> &'static str {
        "GRANT ALL"
    }
    fn category(&self) -> Category {
        Category::Security
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "GRANT ALL is almost never what you want. specify the privileges."
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let lower = p.source.to_lowercase();
        let mut out = Vec::new();
        for pat in &["grant all"] {
            if let Some(idx) = lower.find(pat) {
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "GRANT ALL is overbroad; enumerate privileges".into(),
                    line,
                    col,
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}

pub struct PublicSchemaWrite;
impl Rule for PublicSchemaWrite {
    fn id(&self) -> &'static str {
        "drift.security.public-schema"
    }
    fn name(&self) -> &'static str {
        "write to public schema"
    }
    fn category(&self) -> Category {
        Category::Security
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "public schema writes are an old postgres habit; scoped schemas are safer"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let lower = p.source.to_lowercase();
        if lower.contains("create table public.") || lower.contains("insert into public.") {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "explicit write to public schema".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct PlaintextPasswordLiteral;
impl Rule for PlaintextPasswordLiteral {
    fn id(&self) -> &'static str {
        "drift.security.plaintext-password"
    }
    fn name(&self) -> &'static str {
        "plaintext password literal"
    }
    fn category(&self) -> Category {
        Category::Security
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "password literals in migrations leak to logs, backups, git history"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let lower = p.source.to_lowercase();
        let mut out = Vec::new();
        for pat in &["password '", "encrypted password '", "password \""] {
            let mut start = 0;
            while let Some(off) = lower[start..].find(pat) {
                let idx = start + off;
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "literal password in sql".into(),
                    line,
                    col,
                    span: None,
                    fix: None,
                });
                start = idx + pat.len();
            }
        }
        out
    }
}

pub struct DynamicSqlConcat;
impl Rule for DynamicSqlConcat {
    fn id(&self) -> &'static str {
        "drift.security.dynamic-sql-concat"
    }
    fn name(&self) -> &'static str {
        "dynamic sql concatenation marker"
    }
    fn category(&self) -> Category {
        Category::Security
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "concatenated sql in stored procs is an injection smell"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let lower = p.source.to_lowercase();
        for pat in &["execute '", "exec('"] {
            if let Some(idx) = lower.find(pat) {
                let tail = &lower[idx..idx + (200.min(lower.len() - idx))];
                if tail.contains("||") || tail.contains(" + ") {
                    let (line, col) = p.line_col(idx);
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: "dynamic sql assembled via string concat".into(),
                        line,
                        col,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

pub struct DropWithoutIfExists;
impl Rule for DropWithoutIfExists {
    fn id(&self) -> &'static str {
        "drift.security.drop-without-if-exists"
    }
    fn name(&self) -> &'static str {
        "DROP without IF EXISTS"
    }
    fn category(&self) -> Category {
        Category::Security
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "idempotent migrations should use IF EXISTS"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let tokens: Vec<_> = p
            .tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        for i in 0..tokens.len().saturating_sub(2) {
            if let Token::Word(w) = &tokens[i].token {
                if w.keyword == sqlparser::keywords::Keyword::DROP {
                    let next = &tokens[i + 1].token;
                    let after = &tokens[i + 2].token;
                    let if_exists = matches!(
                        after,
                        Token::Word(x) if x.keyword == sqlparser::keywords::Keyword::IF
                    );
                    if matches!(next, Token::Word(_)) && !if_exists {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "DROP ... should use IF EXISTS for idempotency".into(),
                            line: tokens[i].location.line as usize,
                            col: tokens[i].location.column as usize,
                            span: None,
                            fix: None,
                        });
                    }
                }
            }
        }
        out
    }
}

pub struct TruncateNoCascade;
impl Rule for TruncateNoCascade {
    fn id(&self) -> &'static str {
        "drift.security.truncate-no-cascade"
    }
    fn name(&self) -> &'static str {
        "TRUNCATE without explicit cascade/restrict"
    }
    fn category(&self) -> Category {
        Category::Security
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "TRUNCATE semantics differ across engines; state cascade/restrict explicitly"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let lower = p.source.to_lowercase();
        if lower.contains("truncate ") && !lower.contains("cascade") && !lower.contains("restrict")
        {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "TRUNCATE without CASCADE or RESTRICT".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}
