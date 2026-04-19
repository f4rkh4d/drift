//! formatter.
//!
//! this is deliberately modest. we re-emit the token stream with normalized
//! whitespace around commas and operators, fix keyword case, and ensure a
//! final newline. the fully structural "wrap long SELECT" rewrite is on the
//! 0.15 roadmap — doing it safely means walking the ast, and the ast loses
//! comments.

use crate::config::{Config, KeywordCase};
use crate::dialect::Dialect;
use crate::fixer::fix;

pub fn format(source: &str, dialect: Dialect, cfg: &Config) -> String {
    // current strategy: run the fixer, then normalize spacing around commas.
    // (proper reflow lives behind a feature flag until 0.15.)
    let (pass1, _) = fix(source, dialect, cfg);
    let pass2 = normalize_commas(&pass1);
    apply_keyword_case(&pass2, cfg.format.keyword_case)
}

fn normalize_commas(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b',' {
            // drop any immediately preceding spaces we just pushed
            while out.ends_with(' ') {
                out.pop();
            }
            out.push(',');
            // ensure single space after, unless next is whitespace/newline/)
            if i + 1 < bytes.len() {
                let n = bytes[i + 1];
                if n != b' ' && n != b'\n' && n != b'\t' && n != b')' {
                    out.push(' ');
                }
            }
        } else {
            out.push(b as char);
        }
        i += 1;
    }
    out
}

fn apply_keyword_case(s: &str, case: KeywordCase) -> String {
    // the fixer already handled this via sqlparser; this is a no-op
    // placeholder for future stylistic passes.
    let _ = case;
    s.to_string()
}
