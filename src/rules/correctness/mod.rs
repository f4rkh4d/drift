//! correctness: things that are probably wrong or dangerous.
//!
//! many of these need the ast. we fall back to token-level heuristics when
//! the ast isn't available.

use super::{Category, Rule, Severity, Violation};
use crate::config::Config;
use crate::parse::Parsed;
use sqlparser::ast::{SetExpr, Statement, TableFactor};
use sqlparser::tokenizer::Token;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(MissingWhereUpdate));
    out.push(Box::new(MissingWhereDelete));
    out.push(Box::new(SelfJoinNoAlias));
    out.push(Box::new(CartesianJoin));
    out.push(Box::new(BetweenOnDate));
    out.push(Box::new(ImplicitTypeCoercion));
    out.push(Box::new(CaseWithoutElse));
    out.push(Box::new(NullEqualityRule));
    out.push(Box::new(DistinctOnWithoutOrderBy));
    out.push(Box::new(UnionVsUnionAll));
    out.push(Box::new(DivZeroLiteral));
    out.push(Box::new(DuplicateColumn));
    out.push(Box::new(OrderByOrdinal));
    out.push(Box::new(GroupByWithNoAgg));
    out.push(Box::new(UsingReservedFnName));
}

pub struct MissingWhereUpdate;
impl Rule for MissingWhereUpdate {
    fn id(&self) -> &'static str {
        "drift.correctness.missing-where-update"
    }
    fn name(&self) -> &'static str {
        "UPDATE without WHERE"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "UPDATE without WHERE rewrites every row. almost always a mistake."
    }
    fn example_bad(&self) -> &'static str {
        "UPDATE users SET active = 0;"
    }
    fn example_good(&self) -> &'static str {
        "UPDATE users SET active = 0 WHERE last_login < '2024-01-01';"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::Update {
                selection: None, ..
            } = stmt
            {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "UPDATE statement has no WHERE clause".into(),
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

pub struct MissingWhereDelete;
impl Rule for MissingWhereDelete {
    fn id(&self) -> &'static str {
        "drift.correctness.missing-where-delete"
    }
    fn name(&self) -> &'static str {
        "DELETE without WHERE"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "DELETE without WHERE empties the table. use TRUNCATE if that's what you meant."
    }
    fn example_bad(&self) -> &'static str {
        "DELETE FROM sessions;"
    }
    fn example_good(&self) -> &'static str {
        "DELETE FROM sessions WHERE created_at < NOW() - INTERVAL '30 days';"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::Delete(delete) = stmt {
                if delete.selection.is_none() {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: "DELETE statement has no WHERE clause".into(),
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

pub struct SelfJoinNoAlias;
impl Rule for SelfJoinNoAlias {
    fn id(&self) -> &'static str {
        "drift.correctness.self-join-no-alias"
    }
    fn name(&self) -> &'static str {
        "self-join without alias"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "a table joined with itself needs aliases on both sides"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT *\nFROM employees\nJOIN employees ON employees.manager_id = employees.id;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT e.name, m.name AS manager\nFROM employees e\nJOIN employees m ON e.manager_id = m.id;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::Query(q) = stmt {
                if let SetExpr::Select(select) = &*q.body {
                    let mut seen: Vec<String> = Vec::new();
                    let mut push = |tbl: String, alias: Option<String>| {
                        let key = alias.unwrap_or(tbl);
                        seen.push(key);
                    };
                    for twj in &select.from {
                        collect(&twj.relation, &mut push);
                        for join in &twj.joins {
                            collect(&join.relation, &mut push);
                        }
                    }
                    if has_dup(&seen) {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "same table joined more than once without unique aliases"
                                .into(),
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

fn collect(tf: &TableFactor, push: &mut dyn FnMut(String, Option<String>)) {
    if let TableFactor::Table { name, alias, .. } = tf {
        push(
            name.to_string(),
            alias.as_ref().map(|a| a.name.value.clone()),
        );
    }
}

fn has_dup(v: &[String]) -> bool {
    let mut s = v.to_vec();
    s.sort();
    s.windows(2).any(|w| w[0] == w[1])
}

pub struct CartesianJoin;
impl Rule for CartesianJoin {
    fn id(&self) -> &'static str {
        "drift.correctness.cartesian-join"
    }
    fn name(&self) -> &'static str {
        "probable cartesian product"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "multiple tables in FROM with no WHERE or JOIN predicate"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT u.name, o.total\nFROM users u, orders o;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT u.name, o.total\nFROM users u\nJOIN orders o ON o.user_id = u.id;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for stmt in &p.statements {
            if let Statement::Query(q) = stmt {
                if let SetExpr::Select(select) = &*q.body {
                    if select.from.len() > 1 && select.selection.is_none() {
                        let unjoined = select.from.iter().all(|tw| tw.joins.is_empty());
                        if unjoined {
                            out.push(Violation {
                                rule_id: self.id(),
                                severity: self.default_severity(),
                                message: "multiple tables in FROM without join predicate".into(),
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

pub struct BetweenOnDate;
impl Rule for BetweenOnDate {
    fn id(&self) -> &'static str {
        "drift.correctness.between-on-date"
    }
    fn name(&self) -> &'static str {
        "BETWEEN on dates is inclusive of the upper bound"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "BETWEEN '2025-01-01' AND '2025-01-31' excludes the last day of january when cast to timestamp"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT * FROM events\nWHERE created_at BETWEEN '2025-01-01' AND '2025-01-31';"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM events\nWHERE created_at >= '2025-01-01'\n  AND created_at <  '2025-02-01';"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        // heuristic: look for BETWEEN followed by two date-looking literals
        let mut out = Vec::new();
        let src = &p.source;
        for (idx, _) in src
            .match_indices("BETWEEN")
            .chain(src.match_indices("between"))
        {
            let slice = &src[idx..idx + 80.min(src.len() - idx)];
            if slice.contains("'20") && slice.contains("-") {
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "BETWEEN on dates is inclusive, double-check the upper bound".into(),
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

pub struct ImplicitTypeCoercion;
impl Rule for ImplicitTypeCoercion {
    fn id(&self) -> &'static str {
        "drift.correctness.implicit-coercion"
    }
    fn name(&self) -> &'static str {
        "implicit type coercion"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "comparing a number column to a string literal ('5' vs 5) forces a coercion"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT * FROM orders WHERE total = '100';"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM orders WHERE total = 100;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        // token pattern: `= '<digits>'` or `= "<digits>"` next to a bare ident
        let mut out = Vec::new();
        let tokens: Vec<_> = p
            .tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        for i in 0..tokens.len().saturating_sub(1) {
            let eq = matches!(tokens[i].token, Token::Eq);
            if !eq {
                continue;
            }
            if let Token::SingleQuotedString(s) = &tokens[i + 1].token {
                if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!(
                            "numeric-looking string '{}' compared with =, may force coercion",
                            s
                        ),
                        line: tokens[i + 1].location.line as usize,
                        col: tokens[i + 1].location.column as usize,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

pub struct CaseWithoutElse;
impl Rule for CaseWithoutElse {
    fn id(&self) -> &'static str {
        "drift.correctness.case-without-else"
    }
    fn name(&self) -> &'static str {
        "CASE without ELSE"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "CASE without ELSE returns NULL for unmatched rows — usually unintended"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT\n  CASE status\n    WHEN 'paid'    THEN 1\n    WHEN 'pending' THEN 2\n  END AS rank\nFROM orders;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT\n  CASE status\n    WHEN 'paid'    THEN 1\n    WHEN 'pending' THEN 2\n    ELSE 0\n  END AS rank\nFROM orders;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        // token scan: CASE ... END without ELSE
        let mut out = Vec::new();
        let toks: Vec<(&Token, (usize, usize))> = p
            .tokens
            .iter()
            .filter_map(|t| match &t.token {
                Token::Whitespace(_) => None,
                _ => Some((
                    &t.token,
                    (t.location.line as usize, t.location.column as usize),
                )),
            })
            .collect();
        let mut i = 0;
        while i < toks.len() {
            if let Token::Word(w) = toks[i].0 {
                if w.keyword == sqlparser::keywords::Keyword::CASE {
                    let case_pos = toks[i].1;
                    // find matching END at same depth, track if ELSE seen
                    let mut depth = 1;
                    let mut saw_else = false;
                    let mut j = i + 1;
                    while j < toks.len() && depth > 0 {
                        if let Token::Word(w2) = toks[j].0 {
                            match w2.keyword {
                                sqlparser::keywords::Keyword::CASE => depth += 1,
                                sqlparser::keywords::Keyword::END => depth -= 1,
                                sqlparser::keywords::Keyword::ELSE if depth == 1 => saw_else = true,
                                _ => {}
                            }
                        }
                        j += 1;
                    }
                    if !saw_else {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "CASE expression has no ELSE branch".into(),
                            line: case_pos.0,
                            col: case_pos.1,
                            span: None,
                            fix: None,
                        });
                    }
                    i = j;
                    continue;
                }
            }
            i += 1;
        }
        out
    }
}

pub struct NullEqualityRule;
impl Rule for NullEqualityRule {
    fn id(&self) -> &'static str {
        "drift.correctness.null-equality"
    }
    fn name(&self) -> &'static str {
        "= NULL instead of IS NULL"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "`x = NULL` is always unknown. use `x IS NULL`."
    }
    fn example_bad(&self) -> &'static str {
        "SELECT * FROM users WHERE deleted_at = NULL;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM users WHERE deleted_at IS NULL;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        // token pattern: = NULL or <> NULL, != NULL
        let tokens: Vec<_> = p
            .tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        for i in 0..tokens.len().saturating_sub(1) {
            let is_eq = matches!(tokens[i].token, Token::Eq | Token::Neq);
            if !is_eq {
                continue;
            }
            if let Token::Word(w) = &tokens[i + 1].token {
                if w.keyword == sqlparser::keywords::Keyword::NULL {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: "comparing to NULL with = or <>; use IS NULL / IS NOT NULL".into(),
                        line: tokens[i].location.line as usize,
                        col: tokens[i].location.column as usize,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

pub struct DistinctOnWithoutOrderBy;
impl Rule for DistinctOnWithoutOrderBy {
    fn id(&self) -> &'static str {
        "drift.correctness.distinct-on-no-order"
    }
    fn name(&self) -> &'static str {
        "DISTINCT ON without ORDER BY"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "DISTINCT ON without a matching ORDER BY returns arbitrary rows"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT DISTINCT ON (user_id) *\nFROM events;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT DISTINCT ON (user_id) *\nFROM events\nORDER BY user_id, created_at DESC;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let src = p.source.to_lowercase();
        if src.contains("distinct on") && !src.contains("order by") {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "DISTINCT ON without an ORDER BY; row selection is nondeterministic"
                    .into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

pub struct UnionVsUnionAll;
impl Rule for UnionVsUnionAll {
    fn id(&self) -> &'static str {
        "drift.correctness.union-vs-union-all"
    }
    fn name(&self) -> &'static str {
        "UNION vs UNION ALL"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "plain UNION deduplicates, which you rarely want. be explicit."
    }
    fn example_bad(&self) -> &'static str {
        "SELECT id FROM customers\nUNION\nSELECT id FROM partners;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT id FROM customers\nUNION ALL\nSELECT id FROM partners;"
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
                if w.keyword == sqlparser::keywords::Keyword::UNION {
                    let next_is_all = matches!(
                        &tokens[i + 1].token,
                        Token::Word(n) if n.keyword == sqlparser::keywords::Keyword::ALL
                    );
                    if !next_is_all {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "UNION deduplicates; prefer UNION ALL unless you need it"
                                .into(),
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

pub struct DivZeroLiteral;
impl Rule for DivZeroLiteral {
    fn id(&self) -> &'static str {
        "drift.correctness.div-zero-literal"
    }
    fn name(&self) -> &'static str {
        "literal division by zero"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn description(&self) -> &'static str {
        "`/ 0` as a literal is a guaranteed runtime error"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT total / 0 FROM invoices;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT CASE WHEN tax_rate = 0 THEN 0 ELSE total / tax_rate END\nFROM invoices;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let tokens: Vec<_> = p
            .tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        for i in 0..tokens.len().saturating_sub(1) {
            if matches!(tokens[i].token, Token::Div) {
                if let Token::Number(n, _) = &tokens[i + 1].token {
                    if n == "0" || n == "0.0" {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "division by literal zero".into(),
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

pub struct DuplicateColumn;
impl Rule for DuplicateColumn {
    fn id(&self) -> &'static str {
        "drift.correctness.duplicate-column"
    }
    fn name(&self) -> &'static str {
        "duplicate column in select"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "same column appearing twice in SELECT without aliasing"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT id, name, name FROM users;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT id, name, display_name FROM users;"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new() // placeholder; requires projection normalization
    }
}

pub struct OrderByOrdinal;
impl Rule for OrderByOrdinal {
    fn id(&self) -> &'static str {
        "drift.correctness.order-by-ordinal"
    }
    fn name(&self) -> &'static str {
        "ORDER BY ordinal"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "ORDER BY 1, 2 is fragile; use explicit column names"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT id, created_at, name FROM users ORDER BY 2 DESC;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT id, created_at, name FROM users ORDER BY created_at DESC;"
    }
    fn check(&self, p: &Parsed, _c: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let src = &p.source;
        for (idx, _) in src
            .match_indices("ORDER BY")
            .chain(src.match_indices("order by"))
        {
            let tail = &src[idx + 8..];
            let next = tail.trim_start();
            if next
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                let (line, col) = p.line_col(idx);
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "ORDER BY <ordinal> is fragile, use column names".into(),
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

pub struct GroupByWithNoAgg;
impl Rule for GroupByWithNoAgg {
    fn id(&self) -> &'static str {
        "drift.correctness.group-by-no-agg"
    }
    fn name(&self) -> &'static str {
        "GROUP BY with no aggregation"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "GROUP BY with no aggregate is equivalent to DISTINCT"
    }
    fn example_bad(&self) -> &'static str {
        "SELECT user_id\nFROM events\nGROUP BY user_id;"
    }
    fn example_good(&self) -> &'static str {
        "SELECT DISTINCT user_id FROM events;"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}

pub struct UsingReservedFnName;
impl Rule for UsingReservedFnName {
    fn id(&self) -> &'static str {
        "drift.correctness.reserved-fn-name"
    }
    fn name(&self) -> &'static str {
        "function declared with reserved name"
    }
    fn category(&self) -> Category {
        Category::Correctness
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "CREATE FUNCTION with a reserved name will shadow built-ins"
    }
    fn example_bad(&self) -> &'static str {
        "CREATE FUNCTION count(text) RETURNS int AS $$\n  SELECT length($1)\n$$ LANGUAGE sql;"
    }
    fn example_good(&self) -> &'static str {
        "CREATE FUNCTION text_length(text) RETURNS int AS $$\n  SELECT length($1)\n$$ LANGUAGE sql;"
    }
    fn check(&self, _p: &Parsed, _c: &Config) -> Vec<Violation> {
        Vec::new()
    }
}
