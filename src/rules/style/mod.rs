//! style rules: cosmetic things a formatter could fix.
//!
//! most of these operate on the token stream. the ast is only useful once
//! semantics matter.

use super::{Category, Rule, Severity, Violation};
use crate::config::{Config, KeywordCase};
use crate::parse::{is_keyword, Parsed};
use sqlparser::tokenizer::Token;

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(KeywordCaseRule));
    out.push(Box::new(IdentifierCaseRule));
    out.push(Box::new(IndentRule));
    out.push(Box::new(TrailingWhitespaceRule));
    out.push(Box::new(TrailingNewlineRule));
    out.push(Box::new(SemicolonTerminatorRule));
    out.push(Box::new(LeadingCommaRule));
    out.push(Box::new(DoubleBlankRule));
    out.push(Box::new(TabIndentRule));
    out.push(Box::new(SpaceBeforeCommaRule));
    out.push(Box::new(SpaceAfterCommaRule));
    out.push(Box::new(SpaceAroundOperatorRule));
    out.push(Box::new(AliasAsRule));
    out.push(Box::new(SingleQuoteStringRule));
    out.push(Box::new(UpperKeywordReservedRule));
    out.push(Box::new(LineLengthRule));
    out.push(Box::new(RedundantParensRule));
    out.push(Box::new(EmptyFileRule));
    out.push(Box::new(TrailingCommaRule));
    out.push(Box::new(CrlfRule));
}

// drift.style.keyword-case
pub struct KeywordCaseRule;
impl Rule for KeywordCaseRule {
    fn id(&self) -> &'static str {
        "drift.style.keyword-case"
    }
    fn name(&self) -> &'static str {
        "keyword case"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "sql keywords should use a consistent case (upper by default)"
    }
    fn example_bad(&self) -> &'static str {
        "select * from users"
    }
    fn example_good(&self) -> &'static str {
        "SELECT * FROM users"
    }
    fn fixable(&self) -> bool {
        true
    }
    fn check(&self, p: &Parsed, cfg: &Config) -> Vec<Violation> {
        let case = cfg.rule_case(self.id()).unwrap_or(cfg.format.keyword_case);
        let mut out = Vec::new();
        for t in &p.tokens {
            if let Token::Word(w) = &t.token {
                if w.keyword != sqlparser::keywords::Keyword::NoKeyword && w.quote_style.is_none() {
                    let expected = match case {
                        KeywordCase::Upper => w.value.to_uppercase(),
                        KeywordCase::Lower => w.value.to_lowercase(),
                    };
                    if w.value != expected {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: format!("keyword `{}` should be `{}`", w.value, expected),
                            line: t.location.line as usize,
                            col: t.location.column as usize,
                            span: None,
                            fix: Some(expected),
                        });
                    }
                }
            }
        }
        out
    }
}

// drift.style.identifier-case
pub struct IdentifierCaseRule;
impl Rule for IdentifierCaseRule {
    fn id(&self) -> &'static str {
        "drift.style.identifier-case"
    }
    fn name(&self) -> &'static str {
        "identifier case"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "unquoted identifiers should be lowercase"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for t in &p.tokens {
            if let Token::Word(w) = &t.token {
                if w.keyword == sqlparser::keywords::Keyword::NoKeyword
                    && w.quote_style.is_none()
                    && w.value.chars().any(|c| c.is_ascii_uppercase())
                {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!("identifier `{}` mixes case", w.value),
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

// drift.style.indent
pub struct IndentRule;
impl Rule for IndentRule {
    fn id(&self) -> &'static str {
        "drift.style.indent"
    }
    fn name(&self) -> &'static str {
        "indent width"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "indentation should be a multiple of the configured indent width"
    }
    fn check(&self, p: &Parsed, cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            let leading = line.chars().take_while(|c| *c == ' ').count();
            if leading > 0 && !line.trim().is_empty() && leading % cfg.format.indent != 0 {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!(
                        "indent of {} spaces is not a multiple of {}",
                        leading, cfg.format.indent
                    ),
                    line: i + 1,
                    col: 1,
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}

// drift.style.trailing-whitespace
pub struct TrailingWhitespaceRule;
impl Rule for TrailingWhitespaceRule {
    fn id(&self) -> &'static str {
        "drift.style.trailing-whitespace"
    }
    fn name(&self) -> &'static str {
        "trailing whitespace"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "lines should not end with whitespace"
    }
    fn fixable(&self) -> bool {
        true
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "trailing whitespace".into(),
                    line: i + 1,
                    col: line.trim_end().len() + 1,
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}

// drift.style.trailing-newline
pub struct TrailingNewlineRule;
impl Rule for TrailingNewlineRule {
    fn id(&self) -> &'static str {
        "drift.style.trailing-newline"
    }
    fn name(&self) -> &'static str {
        "final newline"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "file should end with a single newline"
    }
    fn fixable(&self) -> bool {
        true
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        if p.source.is_empty() {
            return Vec::new();
        }
        if !p.source.ends_with('\n') {
            let last_line = p.source.lines().count().max(1);
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "file must end with a newline".into(),
                line: last_line,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

// drift.style.semicolon-terminator
pub struct SemicolonTerminatorRule;
impl Rule for SemicolonTerminatorRule {
    fn id(&self) -> &'static str {
        "drift.style.semicolon-terminator"
    }
    fn name(&self) -> &'static str {
        "semicolon terminator"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "every statement should end with a semicolon"
    }
    fn fixable(&self) -> bool {
        true
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        if p.statements.is_empty() {
            return Vec::new();
        }
        let trimmed = p.source.trim_end();
        if trimmed.is_empty() {
            return Vec::new();
        }
        if !trimmed.ends_with(';') {
            let (line, col) = p.line_col(trimmed.len());
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "missing trailing semicolon".into(),
                line,
                col,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

// drift.style.leading-comma
pub struct LeadingCommaRule;
impl Rule for LeadingCommaRule {
    fn id(&self) -> &'static str {
        "drift.style.leading-comma"
    }
    fn name(&self) -> &'static str {
        "leading commas"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "flags lines that start with a comma when the style is trailing"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            if line.trim_start().starts_with(',') {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "leading comma (expected trailing)".into(),
                    line: i + 1,
                    col: line.find(',').map(|x| x + 1).unwrap_or(1),
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}

// drift.style.double-blank-line
pub struct DoubleBlankRule;
impl Rule for DoubleBlankRule {
    fn id(&self) -> &'static str {
        "drift.style.double-blank-line"
    }
    fn name(&self) -> &'static str {
        "double blank line"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "no more than one consecutive blank line"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        let mut prev_blank = false;
        for (i, line) in p.source.lines().enumerate() {
            let blank = line.trim().is_empty();
            if blank && prev_blank {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "more than one consecutive blank line".into(),
                    line: i + 1,
                    col: 1,
                    span: None,
                    fix: None,
                });
            }
            prev_blank = blank;
        }
        out
    }
}

// drift.style.tab-indent
pub struct TabIndentRule;
impl Rule for TabIndentRule {
    fn id(&self) -> &'static str {
        "drift.style.tab-indent"
    }
    fn name(&self) -> &'static str {
        "tab indent"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "tabs are forbidden for indentation"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            if line.starts_with('\t') {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "tab indentation".into(),
                    line: i + 1,
                    col: 1,
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}

// drift.style.space-before-comma
pub struct SpaceBeforeCommaRule;
impl Rule for SpaceBeforeCommaRule {
    fn id(&self) -> &'static str {
        "drift.style.space-before-comma"
    }
    fn name(&self) -> &'static str {
        "space before comma"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "do not put whitespace before commas"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            let bytes = line.as_bytes();
            for (j, b) in bytes.iter().enumerate() {
                if *b == b',' && j > 0 && bytes[j - 1] == b' ' {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: "space before comma".into(),
                        line: i + 1,
                        col: j + 1,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

// drift.style.space-after-comma
pub struct SpaceAfterCommaRule;
impl Rule for SpaceAfterCommaRule {
    fn id(&self) -> &'static str {
        "drift.style.space-after-comma"
    }
    fn name(&self) -> &'static str {
        "space after comma"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "commas must be followed by whitespace"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            let bytes = line.as_bytes();
            for (j, b) in bytes.iter().enumerate() {
                if *b == b',' && j + 1 < bytes.len() {
                    let nxt = bytes[j + 1];
                    if nxt != b' ' && nxt != b'\t' && nxt != b'\n' && nxt != b')' {
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "missing space after comma".into(),
                            line: i + 1,
                            col: j + 2,
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

// drift.style.space-around-operator
pub struct SpaceAroundOperatorRule;
impl Rule for SpaceAroundOperatorRule {
    fn id(&self) -> &'static str {
        "drift.style.space-around-operator"
    }
    fn name(&self) -> &'static str {
        "space around operator"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "binary operators should have whitespace on both sides"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        // only flag the cheap cases: `a=b`, `a<b`, `a>b` where both sides are alnum
        for (i, line) in p.source.lines().enumerate() {
            let bytes = line.as_bytes();
            for j in 1..bytes.len().saturating_sub(1) {
                let c = bytes[j];
                if matches!(c, b'=' | b'<' | b'>')
                    && bytes[j - 1].is_ascii_alphanumeric()
                    && bytes[j + 1].is_ascii_alphanumeric()
                {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: "missing space around operator".into(),
                        line: i + 1,
                        col: j + 1,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

// drift.style.alias-as
pub struct AliasAsRule;
impl Rule for AliasAsRule {
    fn id(&self) -> &'static str {
        "drift.style.alias-as"
    }
    fn name(&self) -> &'static str {
        "explicit AS for column aliases"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "use explicit AS for column aliases (select a AS b, not select a b)"
    }
    fn check(&self, _p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        // structural check needs more context; sentinel until 0.15
        Vec::new()
    }
}

// drift.style.single-quote-string
pub struct SingleQuoteStringRule;
impl Rule for SingleQuoteStringRule {
    fn id(&self) -> &'static str {
        "drift.style.single-quote-string"
    }
    fn name(&self) -> &'static str {
        "single-quoted string literals"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "string literals should use single quotes (double quotes are identifiers in ansi sql)"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        // scan for "..." that looks like a string literal, excluding actual identifier use
        let bytes = p.source.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'"' {
                if let Some(end) = bytes[i + 1..].iter().position(|&b| b == b'"') {
                    let inner = &p.source[i + 1..i + 1 + end];
                    // heuristic: if it contains spaces or punctuation unlikely in idents, flag
                    if inner.contains(' ') || inner.contains('?') || inner.contains('!') {
                        let (line, col) = p.line_col(i);
                        out.push(Violation {
                            rule_id: self.id(),
                            severity: self.default_severity(),
                            message: "double-quoted string looks like a literal, use single quotes"
                                .into(),
                            line,
                            col,
                            span: None,
                            fix: None,
                        });
                    }
                    i += end + 2;
                    continue;
                }
            }
            i += 1;
        }
        out
    }
}

// drift.style.reserved-as-identifier-case
pub struct UpperKeywordReservedRule;
impl Rule for UpperKeywordReservedRule {
    fn id(&self) -> &'static str {
        "drift.style.reserved-word-quoted"
    }
    fn name(&self) -> &'static str {
        "quoted reserved word identifier"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "avoid using reserved keywords as identifiers, even quoted"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for t in &p.tokens {
            if let Token::Word(w) = &t.token {
                if w.quote_style.is_some() && w.keyword != sqlparser::keywords::Keyword::NoKeyword {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: format!("`{}` is a reserved word used as an identifier", w.value),
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

// drift.style.line-length
pub struct LineLengthRule;
impl Rule for LineLengthRule {
    fn id(&self) -> &'static str {
        "drift.style.line-length"
    }
    fn name(&self) -> &'static str {
        "line length"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "lines should be within the configured max length"
    }
    fn check(&self, p: &Parsed, cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for (i, line) in p.source.lines().enumerate() {
            if line.chars().count() > cfg.format.max_line {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: format!(
                        "line exceeds {} chars ({})",
                        cfg.format.max_line,
                        line.chars().count()
                    ),
                    line: i + 1,
                    col: cfg.format.max_line + 1,
                    span: None,
                    fix: None,
                });
            }
        }
        out
    }
}

// drift.style.redundant-parens
pub struct RedundantParensRule;
impl Rule for RedundantParensRule {
    fn id(&self) -> &'static str {
        "drift.style.redundant-parens"
    }
    fn name(&self) -> &'static str {
        "redundant parentheses"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "flags `((expr))` — one level is enough"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        // naive token walk for ( ( pattern
        let mut i = 0;
        while i + 1 < p.tokens.len() {
            if matches!(p.tokens[i].token, Token::LParen)
                && matches!(p.tokens[i + 1].token, Token::LParen)
            {
                out.push(Violation {
                    rule_id: self.id(),
                    severity: self.default_severity(),
                    message: "double parentheses".into(),
                    line: p.tokens[i].location.line as usize,
                    col: p.tokens[i].location.column as usize,
                    span: None,
                    fix: None,
                });
            }
            i += 1;
        }
        out
    }
}

// drift.style.empty-file
pub struct EmptyFileRule;
impl Rule for EmptyFileRule {
    fn id(&self) -> &'static str {
        "drift.style.empty-file"
    }
    fn name(&self) -> &'static str {
        "empty file"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "empty or whitespace-only files probably aren't intended"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        if p.source.trim().is_empty() {
            return vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "file is empty or whitespace only".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }];
        }
        Vec::new()
    }
}

// drift.style.trailing-comma-in-list
pub struct TrailingCommaRule;
impl Rule for TrailingCommaRule {
    fn id(&self) -> &'static str {
        "drift.style.trailing-comma"
    }
    fn name(&self) -> &'static str {
        "trailing comma in select list"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn description(&self) -> &'static str {
        "trailing commas before `from` / `)` are a parse error in most dialects"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for i in 0..p.tokens.len().saturating_sub(1) {
            if matches!(p.tokens[i].token, Token::Comma) {
                // skip whitespace
                let mut j = i + 1;
                while j < p.tokens.len() && matches!(p.tokens[j].token, Token::Whitespace(_)) {
                    j += 1;
                }
                if j < p.tokens.len()
                    && matches!(
                        &p.tokens[j].token,
                        Token::Word(w) if w.keyword == sqlparser::keywords::Keyword::FROM
                    )
                {
                    out.push(Violation {
                        rule_id: self.id(),
                        severity: self.default_severity(),
                        message: "trailing comma before FROM".into(),
                        line: p.tokens[i].location.line as usize,
                        col: p.tokens[i].location.column as usize,
                        span: None,
                        fix: None,
                    });
                }
            }
        }
        out
    }
}

// drift.style.crlf
pub struct CrlfRule;
impl Rule for CrlfRule {
    fn id(&self) -> &'static str {
        "drift.style.crlf"
    }
    fn name(&self) -> &'static str {
        "crlf line endings"
    }
    fn category(&self) -> Category {
        Category::Style
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }
    fn description(&self) -> &'static str {
        "files should use LF line endings, not CRLF"
    }
    fn check(&self, p: &Parsed, _cfg: &Config) -> Vec<Violation> {
        if p.source.contains("\r\n") {
            vec![Violation {
                rule_id: self.id(),
                severity: self.default_severity(),
                message: "file uses CRLF line endings".into(),
                line: 1,
                col: 1,
                span: None,
                fix: None,
            }]
        } else {
            Vec::new()
        }
    }
}

// suppress unused import warning
#[allow(dead_code)]
fn _keep() -> bool {
    is_keyword(&Token::Comma)
}
