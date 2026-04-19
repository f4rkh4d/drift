//! cli dispatch.

use crate::config::Config;
use crate::dialect::Dialect;
use crate::fixer::fix;
use crate::formatter::format;
use crate::parse::parse;
use crate::report::{render, FileReport, Format};
use crate::rules::{Registry, Severity};
use anyhow::Result;
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "drift",
    version,
    about = "sql linter and formatter. multi-dialect. single binary.",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// path to config file (default: discover drift.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// override dialect
    #[arg(long, global = true)]
    pub dialect: Option<String>,

    /// disable colour output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// lint files, print violations, exit non-zero if any
    Check {
        files: Vec<PathBuf>,
        #[arg(long, default_value = "pretty")]
        format: String,
        #[arg(long)]
        stdin: bool,
    },
    /// apply safe auto-fixes
    Fix {
        files: Vec<PathBuf>,
        /// don't write, print a diff
        #[arg(long)]
        check: bool,
    },
    /// reformat a file
    Format {
        files: Vec<PathBuf>,
        #[arg(long)]
        in_place: bool,
    },
    /// start language server over stdio
    Lsp,
    /// show full description of a rule
    Explain { rule: String },
    /// list all rules
    Rules {
        #[arg(long)]
        json: bool,
    },
}

pub fn run(cli: Cli) -> Result<i32> {
    let mut cfg = load_config(cli.config.as_deref())?;
    if let Some(ref d) = cli.dialect {
        cfg.dialect = Some(d.parse::<Dialect>().map_err(anyhow::Error::msg)?);
    }

    match cli.cmd {
        Cmd::Check {
            files,
            format: fmt,
            stdin,
        } => cmd_check(&cfg, &files, &fmt, stdin, !cli.no_color),
        Cmd::Fix { files, check } => cmd_fix(&cfg, &files, check),
        Cmd::Format { files, in_place } => cmd_format(&cfg, &files, in_place),
        Cmd::Lsp => {
            crate::lsp::run()?;
            Ok(0)
        }
        Cmd::Explain { rule } => cmd_explain(&rule),
        Cmd::Rules { json } => cmd_rules(json),
    }
}

fn load_config(explicit: Option<&Path>) -> Result<Config> {
    if let Some(p) = explicit {
        return Config::load(p);
    }
    let here = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(p) = Config::discover(&here) {
        return Config::load(&p);
    }
    Ok(Config::default())
}

fn expand(files: &[PathBuf]) -> Vec<PathBuf> {
    if files.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in files {
        let s = f.to_string_lossy();
        if s.contains('*') || s.contains('?') {
            if let Ok(iter) = glob::glob(&s) {
                for entry in iter.flatten() {
                    out.push(entry);
                }
            }
        } else if f.is_dir() {
            for ext in &["sql", "pgsql", "mysql", "sqlite"] {
                let pat = format!("{}/**/*.{}", s, ext);
                if let Ok(iter) = glob::glob(&pat) {
                    for entry in iter.flatten() {
                        out.push(entry);
                    }
                }
            }
        } else {
            out.push(f.clone());
        }
    }
    out
}

fn pick_dialect(cfg: &Config, path: &Path) -> Dialect {
    if let Some(d) = cfg.dialect {
        return d;
    }
    Dialect::detect_from_path(path).unwrap_or_default()
}

fn cmd_check(cfg: &Config, files: &[PathBuf], fmt: &str, stdin: bool, color: bool) -> Result<i32> {
    let format = Format::parse(fmt).ok_or_else(|| anyhow::anyhow!("unknown format: {fmt}"))?;
    let registry = Registry::new();

    if stdin {
        use std::io::Read;
        let mut src = String::new();
        std::io::stdin().read_to_string(&mut src)?;
        let dialect = cfg.dialect.unwrap_or_default();
        let parsed = parse(&src, dialect);
        let viols = registry.run(&parsed, cfg);
        let report = vec![FileReport {
            path: "<stdin>",
            source: &src,
            violations: &viols,
        }];
        print!("{}", render(&report, format, color));
        return Ok(if has_error(&viols) { 1 } else { 0 });
    }

    let files = expand(files);
    if files.is_empty() {
        eprintln!("no files matched");
        return Ok(0);
    }

    let results: Vec<(PathBuf, String, Vec<_>)> = files
        .par_iter()
        .filter_map(|path| {
            let src = std::fs::read_to_string(path).ok()?;
            let dialect = pick_dialect(cfg, path);
            let parsed = parse(&src, dialect);
            let viols = registry.run(&parsed, cfg);
            Some((path.clone(), src, viols))
        })
        .collect();

    let any_error = results.iter().any(|(_, _, v)| has_error(v));
    let borrowed: Vec<(String, String, Vec<_>)> = results
        .into_iter()
        .map(|(p, s, v)| (p.to_string_lossy().into_owned(), s, v))
        .collect();
    let reports: Vec<FileReport> = borrowed
        .iter()
        .map(|(p, s, v)| FileReport {
            path: p,
            source: s,
            violations: v,
        })
        .collect();
    print!("{}", render(&reports, format, color));
    Ok(if any_error { 1 } else { 0 })
}

fn has_error(v: &[crate::Violation]) -> bool {
    v.iter().any(|x| x.severity == Severity::Error)
}

fn cmd_fix(cfg: &Config, files: &[PathBuf], check: bool) -> Result<i32> {
    let files = expand(files);
    let mut any_changed = false;
    for path in &files {
        let src = std::fs::read_to_string(path)?;
        let dialect = pick_dialect(cfg, path);
        let (fixed, _stats) = fix(&src, dialect, cfg);
        if fixed != src {
            any_changed = true;
            if check {
                let diff = similar::TextDiff::from_lines(&src, &fixed);
                println!("--- {}", path.display());
                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        similar::ChangeTag::Delete => "-",
                        similar::ChangeTag::Insert => "+",
                        similar::ChangeTag::Equal => " ",
                    };
                    print!("{}{}", sign, change);
                }
            } else {
                std::fs::write(path, &fixed)?;
                println!("fixed {}", path.display());
            }
        }
    }
    Ok(if check && any_changed { 1 } else { 0 })
}

fn cmd_format(cfg: &Config, files: &[PathBuf], in_place: bool) -> Result<i32> {
    let files = expand(files);
    for path in &files {
        let src = std::fs::read_to_string(path)?;
        let dialect = pick_dialect(cfg, path);
        let out = format(&src, dialect, cfg);
        if in_place {
            std::fs::write(path, &out)?;
        } else {
            print!("{}", out);
        }
    }
    Ok(0)
}

fn cmd_explain(rule: &str) -> Result<i32> {
    let r = Registry::new();
    match r.get(rule) {
        Some(rule) => {
            println!("{}  ({})", rule.id(), rule.category().as_str());
            println!("{}\n", rule.name());
            println!("{}", rule.description());
            if !rule.example_bad().is_empty() {
                println!("\nbad:\n  {}", rule.example_bad());
            }
            if !rule.example_good().is_empty() {
                println!("good:\n  {}", rule.example_good());
            }
            println!(
                "\ndefault severity: {}  fixable: {}",
                rule.default_severity().as_str(),
                rule.fixable()
            );
            Ok(0)
        }
        None => {
            eprintln!("no such rule: {rule}");
            Ok(1)
        }
    }
}

fn cmd_rules(json: bool) -> Result<i32> {
    let r = Registry::new();
    if json {
        #[derive(serde::Serialize)]
        struct R<'a> {
            id: &'a str,
            name: &'a str,
            category: &'a str,
            severity: &'a str,
            fixable: bool,
        }
        let all: Vec<R> = r
            .rules()
            .iter()
            .map(|x| R {
                id: x.id(),
                name: x.name(),
                category: x.category().as_str(),
                severity: x.default_severity().as_str(),
                fixable: x.fixable(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&all).unwrap());
    } else {
        for (cat, rules) in r.by_category() {
            println!("\n{}:", cat.as_str());
            for rl in rules {
                println!(
                    "  {}  {}  [{}]{}",
                    rl.id(),
                    rl.name(),
                    rl.default_severity().as_str(),
                    if rl.fixable() { "  fix" } else { "" },
                );
            }
        }
    }
    Ok(0)
}
