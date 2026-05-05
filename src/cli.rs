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
        /// minimum severity that triggers a non-zero exit code
        #[arg(long, default_value = "error", value_parser = ["error", "warning", "info", "never"])]
        fail_on: String,
        /// suppress violations recorded in a baseline file. typical usage:
        /// `--baseline .drift-baseline.json`.
        #[arg(long)]
        baseline: Option<PathBuf>,
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
    /// create or inspect a baseline file (a snapshot of currently-known
    /// violations to suppress on subsequent `drift check --baseline` runs)
    Baseline {
        #[command(subcommand)]
        sub: BaselineSub,
    },
}

#[derive(Subcommand, Debug)]
pub enum BaselineSub {
    /// run a check and record every violation into a baseline file
    Create {
        files: Vec<PathBuf>,
        /// write the baseline here (default: .drift-baseline.json)
        #[arg(long, default_value = ".drift-baseline.json")]
        output: PathBuf,
    },
    /// print a summary of an existing baseline file
    Show {
        /// baseline path (default: .drift-baseline.json)
        #[arg(long, default_value = ".drift-baseline.json")]
        path: PathBuf,
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
            fail_on,
            baseline,
        } => cmd_check(
            &cfg,
            &files,
            &fmt,
            stdin,
            !cli.no_color,
            &fail_on,
            baseline.as_deref(),
        ),
        Cmd::Fix { files, check } => cmd_fix(&cfg, &files, check),
        Cmd::Format { files, in_place } => cmd_format(&cfg, &files, in_place),
        Cmd::Lsp => {
            crate::lsp::run()?;
            Ok(0)
        }
        Cmd::Explain { rule } => cmd_explain(&rule),
        Cmd::Rules { json } => cmd_rules(json),
        Cmd::Baseline { sub } => match sub {
            BaselineSub::Create { files, output } => cmd_baseline_create(&cfg, &files, &output),
            BaselineSub::Show { path } => cmd_baseline_show(&path),
        },
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

fn cmd_check(
    cfg: &Config,
    files: &[PathBuf],
    fmt: &str,
    stdin: bool,
    color: bool,
    fail_on: &str,
    baseline_path: Option<&Path>,
) -> Result<i32> {
    let format = Format::parse(fmt).ok_or_else(|| anyhow::anyhow!("unknown format: {fmt}"))?;
    let threshold = parse_fail_on(fail_on)?;
    let registry = Registry::new();

    let baseline = match baseline_path {
        Some(p) => Some(crate::baseline::Baseline::load(p)?),
        None => None,
    };

    if stdin {
        use std::io::Read;
        let started = std::time::Instant::now();
        let mut src = String::new();
        std::io::stdin().read_to_string(&mut src)?;
        let dialect = cfg.dialect.unwrap_or_default();
        let parsed = parse(&src, dialect);
        let viols_raw = registry.run(&parsed, cfg);
        // baseline keyed by path; stdin uses "<stdin>" which is unlikely to be in
        // a baseline file. apply anyway for symmetry.
        let viols: Vec<_> = match &baseline {
            Some(bl) => bl.filter_violations("<stdin>", &viols_raw),
            None => viols_raw.clone(),
        };
        let report = vec![FileReport {
            path: "<stdin>",
            source: &src,
            violations: &viols,
        }];
        print!("{}", render(&report, format, color));
        if matches!(format, Format::Pretty | Format::Compact) {
            let elapsed_ms = started.elapsed().as_millis();
            eprintln!("{}", crate::report::summary_line(&report, elapsed_ms));
        }
        return Ok(if breaches(&viols, threshold) { 1 } else { 0 });
    }

    let started = std::time::Instant::now();
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

    // apply baseline (if any) before counting against the fail threshold and
    // before rendering. suppressed violations vanish from output entirely so
    // pretty / json / sarif all see the same filtered set.
    let mut suppressed_count = 0usize;
    let borrowed: Vec<(String, String, Vec<_>)> = results
        .into_iter()
        .map(|(p, s, v)| {
            let path_str = p.to_string_lossy().into_owned();
            let v = match &baseline {
                Some(bl) => {
                    let kept = bl.filter_violations(&path_str, &v);
                    suppressed_count += v.len() - kept.len();
                    kept
                }
                None => v,
            };
            (path_str, s, v)
        })
        .collect();

    let breached = borrowed.iter().any(|(_, _, v)| breaches(v, threshold));
    let reports: Vec<FileReport> = borrowed
        .iter()
        .map(|(p, s, v)| FileReport {
            path: p,
            source: s,
            violations: v,
        })
        .collect();
    print!("{}", render(&reports, format, color));

    if matches!(format, Format::Pretty | Format::Compact) {
        let elapsed_ms = started.elapsed().as_millis();
        let mut line = crate::report::summary_line(&reports, elapsed_ms);
        if suppressed_count > 0 {
            line.push_str(&format!(" ({suppressed_count} suppressed by baseline)"));
        }
        eprintln!("{line}");
    }

    Ok(if breached { 1 } else { 0 })
}

fn cmd_baseline_create(cfg: &Config, files: &[PathBuf], output: &Path) -> Result<i32> {
    let registry = Registry::new();
    let files = expand(files);
    if files.is_empty() {
        eprintln!("no files matched, baseline would be empty. aborting.");
        return Ok(2);
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

    let baseline = crate::baseline::Baseline::from_reports(&reports);
    baseline.save(output)?;

    eprintln!(
        "wrote {} ({} files, {} violations recorded)",
        output.display(),
        baseline.files.len(),
        baseline.total(),
    );
    Ok(0)
}

fn cmd_baseline_show(path: &Path) -> Result<i32> {
    let bl = crate::baseline::Baseline::load(path)?;
    println!("baseline: {}", path.display());
    println!("  schema:        {}", bl.schema);
    println!("  drift_version: {}", bl.drift_version);
    println!("  created_at:    {}", bl.created_at);
    println!("  files:         {}", bl.files.len());
    println!("  violations:    {}", bl.total());
    if bl.is_empty() {
        return Ok(0);
    }
    println!();
    for (file, rules) in &bl.files {
        let total: usize = rules.values().sum();
        println!("  {file} ({total})");
        for (rule, n) in rules {
            println!("    {rule}  x{n}");
        }
    }
    Ok(0)
}

/// "error|warning|info|never" -> threshold severity. anything at or above the
/// threshold makes `drift check` exit non-zero. "never" disables the gate.
#[derive(Copy, Clone, Debug)]
enum FailThreshold {
    Error,
    Warning,
    Info,
    Never,
}

fn parse_fail_on(s: &str) -> Result<FailThreshold> {
    Ok(match s {
        "error" => FailThreshold::Error,
        "warning" => FailThreshold::Warning,
        "info" => FailThreshold::Info,
        "never" => FailThreshold::Never,
        other => anyhow::bail!("unknown --fail-on: {other} (use error|warning|info|never)"),
    })
}

fn breaches(v: &[crate::Violation], t: FailThreshold) -> bool {
    let rank = |s: Severity| match s {
        Severity::Error => 3,
        Severity::Warning => 2,
        Severity::Info => 1,
        Severity::Off => 0,
    };
    let threshold_rank = match t {
        FailThreshold::Error => 3,
        FailThreshold::Warning => 2,
        FailThreshold::Info => 1,
        FailThreshold::Never => return false,
    };
    v.iter().any(|x| rank(x.severity) >= threshold_rank)
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
