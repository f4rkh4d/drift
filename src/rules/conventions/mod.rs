//! convention rules: naming + ordering.

use super::{Category, Rule, Severity, Violation};
use crate::config::Config;
use crate::parse::Parsed;
use sqlparser::ast::Statement;
use sqlparser::tokenizer::Token;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(SnakeCaseTables));
    out.push(Box::new(PluralTableName));
    out.push(Box::new(UpperKeywords));
    out.push(Box::new(LowercaseColumns));
    out.push(Box::new(PrefixPkColumn));
    out.push(Box::new(FkNaming));
    out.push(Box::new(IndexNaming));
    out.push(Box::new(NoHungarian));
}

pub struct SnakeCaseTables;
impl Rule for SnakeCaseTables {
    fn id(&self) -> &'static str {
        "drift.conventions.snake-case-tables"
    }
    fn name(&self) -> &'static str {
        "snake_case tables"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "table names should be snake_case"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::CreateTable(ct) = stmt {
                let name = ct.name.to_string();
                if name.chars().any(|c| c.is_ascii_uppercase()) {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!("table `{}` is not snake_case", name),
                        line: 1,
                        col: 1,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

pub struct PluralTableName;
impl Rule for PluralTableName {
    fn id(&self) -> &'static str {
        "drift.conventions.plural-table-name"
    }
    fn name(&self) -> &'static str {
        "plural table names"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "table names should be plural (e.g. users, not user)"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::CreateTable(ct) = stmt {
                let name = ct.name.to_string().to_lowercase();
                let last = name.split('.').next_back().unwrap_or("");
                if !last.ends_with('s')
                    && !last.ends_with("_data")
                    && !last.contains("_log")
                    && !last.is_empty()
                {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!("table `{}` should probably be plural", last),
                        line: 1,
                        col: 1,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

pub struct UpperKeywords;
impl Rule for UpperKeywords {
    fn id(&self) -> &'static str {
        "drift.conventions.upper-keywords"
    }
    fn name(&self) -> &'static str {
        "uppercase keywords"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "the project convention is UPPERCASE keywords (duplicate of style.keyword-case for teams that prefer it here)"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct LowercaseColumns;
impl Rule for LowercaseColumns {
    fn id(&self) -> &'static str {
        "drift.conventions.lowercase-columns"
    }
    fn name(&self) -> &'static str {
        "lowercase column names"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "column names should be lowercase snake_case"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::CreateTable(ct) = stmt {
                for col in &ct.columns {
                    if col.name.value.chars().any(|c| c.is_ascii_uppercase()) {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: format!("column `{}` mixes case", col.name.value),
                            line: 1,
                            col: 1,
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

pub struct PrefixPkColumn;
impl Rule for PrefixPkColumn {
    fn id(&self) -> &'static str {
        "drift.conventions.pk-column-id"
    }
    fn name(&self) -> &'static str {
        "primary key column name"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "primary key column should be `id` (not `user_id` inside users table)"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct FkNaming;
impl Rule for FkNaming {
    fn id(&self) -> &'static str {
        "drift.conventions.fk-naming"
    }
    fn name(&self) -> &'static str {
        "foreign key naming"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "foreign key columns should be `<referenced>_id`"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct IndexNaming;
impl Rule for IndexNaming {
    fn id(&self) -> &'static str {
        "drift.conventions.index-naming"
    }
    fn name(&self) -> &'static str {
        "index naming"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "index names should be `ix_<table>_<cols>`"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let lower = p.source.to_lowercase();
        let idx_iter = lower.match_indices("create index ");
        for (idx, _) in idx_iter {
            let tail = &lower[idx + 13..];
            let name: String = tail
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && !name.starts_with("ix_") && !name.starts_with("idx_") {
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!("index `{}` should start with ix_ or idx_", name),
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

pub struct NoHungarian;
impl Rule for NoHungarian {
    fn id(&self) -> &'static str {
        "drift.conventions.no-hungarian"
    }
    fn name(&self) -> &'static str {
        "no hungarian notation"
    }
    fn category(&self) -> Category {
        Category::Conventions
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "don't prefix columns with type (e.g. `str_name`, `int_count`)"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for t in &p.tokens {
            if let Token::Word(w) = &t.token {
                if w.keyword == sqlparser::keywords::Keyword::NoKeyword {
                    for pref in &["str_", "int_", "bln_", "dt_"] {
                        if w.value.to_lowercase().starts_with(pref) {
                            out.push(Violation {
                                rule_id: self.id(),
                                severity: self.default_severity(),
                                message: format!("hungarian prefix on `{}`", w.value),
                                line: t.location.line as usize,
                                col: t.location.column as usize,
                                span: None,
                                fix: None,
                            });
                            break;
                        }
                    }
                }
            }
        }
        out
    }
}
