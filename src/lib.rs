//! drift — sql linter and formatter
//!
//! public surface is intentionally small. most callers use the cli.

pub mod cli;
pub mod config;
pub mod dialect;
pub mod fixer;
pub mod formatter;
pub mod lsp;
pub mod parse;
pub mod report;
pub mod rules;

pub use config::Config;
pub use dialect::Dialect;
pub use rules::{Registry, Rule, Severity, Violation};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
