//! deterministic auto-fix pass.
//!
//! intentionally small. we only touch things the formatter would also touch.
//! anything that could alter query semantics is rejected up front.

use crate::config::{Config, KeywordCase};
use crate::dialect::Dialect;
use crate::parse::parse;
use sqlparser::tokenizer::Token;

#[derive(Debug, Clone, Copy, Default)]
pub struct FixStats {
    pub keyword_case: usize,
    pub trailing_ws: usize,
    pub final_newline: usize,
    pub semicolon: usize,
}

pub fn fix(source: &str, dialect: Dialect, cfg: &Config) -> (String, FixStats) {
    let mut stats = FixStats::default();
    let mut out = source.to_string();
    out = fix_keyword_case(&out, dialect, cfg, &mut stats);
    out = fix_trailing_whitespace(&out, &mut stats);
    out = fix_semicolon(&out, dialect, &mut stats);
    out = fix_final_newline(&out, &mut stats);
    (out, stats)
}

fn fix_keyword_case(source: &str, dialect: Dialect, cfg: &Config, stats: &mut FixStats) -> String {
    let desired = cfg
        .rule_case("drift.style.keyword-case")
        .unwrap_or(cfg.format.keyword_case);
    let parsed = parse(source, dialect);
    // collect (byte-start, byte-end, replacement) edits
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for t in &parsed.tokens {
        if let Token::Word(w) = &t.token {
            if w.keyword != sqlparser::keywords::Keyword::NoKeyword && w.quote_style.is_none() {
                let want = match desired {
                    KeywordCase::Upper => w.value.to_uppercase(),
                    KeywordCase::Lower => w.value.to_lowercase(),
                };
                if want != w.value {
                    if let Some((s, e)) = byte_range(source, t) {
                        edits.push((s, e, want));
                    }
                }
            }
        }
    }
    apply_edits(source, edits, &mut stats.keyword_case)
}

fn byte_range(source: &str, t: &sqlparser::tokenizer::TokenWithLocation) -> Option<(usize, usize)> {
    // sqlparser gives us line/col only; translate.
    // we only get a single point — use the token's displayed length as width.
    let start = line_col_to_byte(source, t.location.line as usize, t.location.column as usize)?;
    let width = token_display_len(&t.token);
    let end = (start + width).min(source.len());
    if end >= start {
        Some((start, end))
    } else {
        None
    }
}

fn token_display_len(t: &Token) -> usize {
    match t {
        Token::Word(w) => w.value.chars().count(),
        _ => t.to_string().chars().count(),
    }
}

fn line_col_to_byte(source: &str, line: usize, col: usize) -> Option<usize> {
    if line == 0 || col == 0 {
        return None;
    }
    let mut cur_line = 1;
    let mut cur_col = 1;
    for (i, ch) in source.char_indices() {
        if cur_line == line && cur_col == col {
            return Some(i);
        }
        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
    }
    Some(source.len())
}

fn apply_edits(
    source: &str,
    mut edits: Vec<(usize, usize, String)>,
    counter: &mut usize,
) -> String {
    edits.sort_by_key(|e| e.0);
    // drop overlapping
    let mut kept: Vec<(usize, usize, String)> = Vec::new();
    for e in edits {
        if let Some(last) = kept.last() {
            if e.0 < last.1 {
                continue;
            }
        }
        kept.push(e);
    }
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    for (s, e, r) in kept {
        if s >= source.len() {
            break;
        }
        out.push_str(&source[cursor..s]);
        out.push_str(&r);
        cursor = e.min(source.len());
        *counter += 1;
    }
    out.push_str(&source[cursor..]);
    out
}

fn fix_trailing_whitespace(source: &str, stats: &mut FixStats) -> String {
    let mut out = String::with_capacity(source.len());
    let mut count = 0;
    for line in source.split_inclusive('\n') {
        let (content, nl) = match line.strip_suffix('\n') {
            Some(c) => (c, "\n"),
            None => (line, ""),
        };
        let trimmed = content.trim_end_matches([' ', '\t']);
        if trimmed.len() != content.len() {
            count += 1;
        }
        out.push_str(trimmed);
        out.push_str(nl);
    }
    stats.trailing_ws += count;
    out
}

fn fix_final_newline(source: &str, stats: &mut FixStats) -> String {
    if source.is_empty() {
        return source.to_string();
    }
    if !source.ends_with('\n') {
        stats.final_newline += 1;
        let mut s = source.to_string();
        s.push('\n');
        return s;
    }
    source.to_string()
}

fn fix_semicolon(source: &str, dialect: Dialect, stats: &mut FixStats) -> String {
    let parsed = parse(source, dialect);
    if parsed.statements.is_empty() {
        return source.to_string();
    }
    let trimmed = source.trim_end();
    if trimmed.is_empty() {
        return source.to_string();
    }
    if trimmed.ends_with(';') {
        return source.to_string();
    }
    // insert a semicolon before the trailing whitespace
    stats.semicolon += 1;
    let tail = &source[trimmed.len()..];
    format!("{};{}", trimmed, tail)
}
