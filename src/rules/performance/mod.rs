//! performance heuristics. these are opinions, not proofs — severity defaults
//! to warning.

use super::{Category, Rule, Severity, Violation};
use crate::config::Config;
use crate::parse::Parsed;
use sqlparser::ast::{SelectItem, SetExpr, Statement};
use sqlparser::tokenizer::Token;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(SelectStar));
    out.push(Box::new(LeadingWildcardLike));
    out.push(Box::new(FnOnIndexedColumn));
    out.push(Box::new(NestedSubqueryCouldBeJoin));
    out.push(Box::new(OrderByRand));
    out.push(Box::new(CountStarVsCountCol));
    out.push(Box::new(InSubqueryCouldBeExists));
    out.push(Box::new(OffsetPaging));
}

pub struct SelectStar;
impl Rule for SelectStar {
    fn id(&self) -> &'static str {
        "drift.performance.select-star"
    }
    fn name(&self) -> &'static str {
        "SELECT *"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "SELECT * fetches columns you don't need and breaks when the schema changes"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::Query(q) = stmt {
                if let SetExpr::Select(select) = &*q.body {
                    for item in &select.projection {
                        if matches!(item, SelectItem::Wildcard(_)) {
                            out.push(Violation {
                                rule_id: self.id(),
                                severity: self.default_severity(),
                                message: "avoid SELECT *; list the columns you need".into(),
                                line: 1,
                                col: 1,
                                span: None,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }
        out
    }
}

pub struct LeadingWildcardLike;
impl Rule for LeadingWildcardLike {
    fn id(&self) -> &'static str {
        "drift.performance.like-leading-wildcard"
    }
    fn name(&self) -> &'static str {
        "LIKE with leading %"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "LIKE '%foo' can't use a btree index. full table scan territory."
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let tokens: Vec<_> = p
            .tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        for i in 0..tokens.len().saturating_sub(1) {
            if let Token::Word(w) = &tokens[i].token {
                if w.keyword == sqlparser::keywords::Keyword::LIKE
                    || w.keyword == sqlparser::keywords::Keyword::ILIKE
                {
                    if let Token::SingleQuotedString(s) = &tokens[i + 1].token {
                        if s.starts_with('%') {
                            out.push(Violation {
                                rule_id: self.id(),
                                severity: self.default_severity(),
                                message: "LIKE pattern starts with %, defeats btree indexes".into(),
                                line: tokens[i + 1].location.line as usize,
                                col: tokens[i + 1].location.column as usize,
                                span: None,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }
        out
    }
}

pub struct FnOnIndexedColumn;
impl Rule for FnOnIndexedColumn {
    fn id(&self) -> &'static str {
        "drift.performance.fn-on-column"
    }
    fn name(&self) -> &'static str {
        "function on column in WHERE"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "calling a function on a column in WHERE prevents the index from being used"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let src = &p.source;
        let lower = src.to_lowercase();
        for pat in &["where lower(", "where upper(", "where date(", "where cast("] {
            let mut start = 0;
            while let Some(off) = lower[start..].find(pat) {
                let idx = start + off;
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!("`{}...)` in WHERE may block index use", &pat[6..]),
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

pub struct NestedSubqueryCouldBeJoin;
impl Rule for NestedSubqueryCouldBeJoin {
    fn id(&self) -> &'static str {
        "drift.performance.nested-subquery"
    }
    fn name(&self) -> &'static str {
        "nested subquery"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "deeply nested subqueries often rewrite to JOINs cleanly"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        // count nested SELECTs via paren depth
        let mut out = Vec::new();
        let mut depth = 0i32;
        let mut max_depth = 0i32;
        let mut max_pos = (1, 1);
        for t in &p.tokens {
            match &t.token {
                Token::LParen => depth += 1,
                Token::RParen => depth -= 1,
                Token::Word(w)
                    if w.keyword == sqlparser::keywords::Keyword::SELECT && depth > max_depth =>
                {
                    max_depth = depth;
                    max_pos = (t.location.line as usize, t.location.column as usize);
                }
                _ => {}
            }
        }
        if max_depth >= 3 {
            out.push(Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: format!("subquery nested {} levels deep", max_depth),
                line: max_pos.0,
                col: max_pos.1,
                span: None,
                fix: None,
            });
        }
        out
    }
}

pub struct OrderByRand;
impl Rule for OrderByRand {
    fn id(&self) -> &'static str {
        "drift.performance.order-by-rand"
    }
    fn name(&self) -> &'static str {
        "ORDER BY random()"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "ORDER BY random() / RAND() sorts the whole table to pick N rows"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let lower = p.source.to_lowercase();
        let mut out = Vec::new();
        for pat in &["order by random()", "order by rand()"] {
            if let Some(idx) = lower.find(pat) {
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "ORDER BY random() full-sorts before it picks; use TABLESAMPLE or offset by id".into(),
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

pub struct CountStarVsCountCol;
impl Rule for CountStarVsCountCol {
    fn id(&self) -> &'static str {
        "drift.performance.count-star-vs-col"
    }
    fn name(&self) -> &'static str {
        "COUNT(column) vs COUNT(*)"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "COUNT(col) filters nulls; if you want total rows, use COUNT(*)"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct InSubqueryCouldBeExists;
impl Rule for InSubqueryCouldBeExists {
    fn id(&self) -> &'static str {
        "drift.performance.in-subquery-exists"
    }
    fn name(&self) -> &'static str {
        "IN (subquery) vs EXISTS"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "IN (subquery) with a large result is often faster as EXISTS"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct OffsetPaging;
impl Rule for OffsetPaging {
    fn id(&self) -> &'static str {
        "drift.performance.offset-paging"
    }
    fn name(&self) -> &'static str {
        "OFFSET for paging"
    }
    fn category(&self) -> Category {
        Category::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "OFFSET is O(n) in most engines; prefer keyset paging for deep pages"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let lower = p.source.to_lowercase();
        if let Some(idx) = lower.find("offset ") {
            let tail = &lower[idx + 7..];
            let num: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = num.parse::<usize>() {
                if n >= 1000 {
                    let (line, col) = p.line_col(idx);
                    return vec![Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!("OFFSET {} will scan all skipped rows", n),
                        line,
                        col,
                        span: None,
                        fix: None,
                    }];
                }
            }
        }
        Vec::new()
    }
}
