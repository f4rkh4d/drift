//! sqlparser wrapper with token + ast + recovery.
//!
//! the ast from sqlparser is used by the semantic rules. the token stream is
//! used by the style rules (so we can see whitespace + case) and the
//! formatter.

use crate::dialect::Dialect;
use sqlparser::ast::Statement;
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::{Token, TokenWithLocation, Tokenizer};

#[derive(Debug)]
pub struct Parsed {
    pub dialect: Dialect,
    pub source: String,
    pub tokens: Vec<TokenWithLocation>,
    pub statements: Vec<Statement>,
    /// sqlparser error, if parsing failed. tokens still present.
    pub parse_error: Option<String>,
}

impl Parsed {
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1usize;
        let mut col = 1usize;
        for (i, b) in self.source.bytes().enumerate() {
            if i >= offset {
                break;
            }
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

pub fn parse(source: &str, dialect: Dialect) -> Parsed {
    let d = dialect.as_parser();
    let mut tokenizer = Tokenizer::new(&*d, source);
    let tokens = tokenizer.tokenize_with_location().unwrap_or_default();

    let statements_res = Parser::parse_sql(&*d, source);
    let (statements, parse_error) = match statements_res {
        Ok(s) => (s, None),
        Err(e) => (Vec::new(), Some(format_parser_error(&e))),
    };

    Parsed {
        dialect,
        source: source.to_string(),
        tokens,
        statements,
        parse_error,
    }
}

fn format_parser_error(e: &ParserError) -> String {
    match e {
        ParserError::TokenizerError(s) => format!("tokenizer: {s}"),
        ParserError::ParserError(s) => s.clone(),
        ParserError::RecursionLimitExceeded => "recursion limit exceeded".into(),
    }
}

/// keyword-ish lookup. we compare against token kind rather than the raw
/// string so we don't misflag identifiers that happen to spell a keyword.
pub fn is_keyword(token: &Token) -> bool {
    matches!(token, Token::Word(w) if w.keyword != sqlparser::keywords::Keyword::NoKeyword)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_select() {
        let p = parse("SELECT 1;", Dialect::Postgres);
        assert!(p.parse_error.is_none());
        assert_eq!(p.statements.len(), 1);
        assert!(!p.tokens.is_empty());
    }

    #[test]
    fn keeps_tokens_on_parse_error() {
        let p = parse("SELECT FROM WHERE", Dialect::Postgres);
        // tokens still tokenized even when parser rejects
        assert!(!p.tokens.is_empty());
    }
}
