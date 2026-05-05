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
        /// re-run the check whenever a file under the given paths changes
        #[arg(long)]
        watch: bool,
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
    /// run a check and aggregate the results: which rules fire most, where
    /// the noise lives, what to consider disabling. emits a recommended
    /// drift.toml fragment at the end.
    Profile {
        files: Vec<PathBuf>,
        /// how many top-firing rules to show
        #[arg(long, default_value = "20")]
        top: usize,
        /// output as JSON for further processing
        #[arg(long)]
        json: bool,
    },
    /// generate per-rule markdown pages into docs/rules/. each page is
    /// derived from the rule trait (id, category, severity, description,
    /// example_bad, example_good) so docs cannot drift from the code.
    Docs {
        /// output directory (default: docs/rules)
        #[arg(long, default_value = "docs/rules")]
        output: PathBuf,
        /// fail with non-zero exit code if any generated page differs
        /// from the file currently on disk. for ci.
        #[arg(long)]
        check: bool,
    },
    /// scaffold a starter drift.toml into the current directory. picks
    /// sensible defaults: postgres dialect, warn preset, correctness
    /// rules promoted to error, conventions off. open the file and
    /// keep what fits your team.
    Init {
        /// overwrite if the output file exists
        #[arg(long)]
        force: bool,
        /// where to write the config (default: drift.toml in cwd)
        #[arg(long, default_value = "drift.toml")]
        output: PathBuf,
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

    // honor the NO_COLOR convention (https://no-color.org/): when NO_COLOR
    // is present in the environment with any value (including the empty
    // string), color output is disabled. the explicit --no-color flag is
    // still honored on top of that.
    let use_color = !cli.no_color && std::env::var_os("NO_COLOR").is_none();

    match cli.cmd {
        Cmd::Check {
            files,
            format: fmt,
            stdin,
            fail_on,
            baseline,
            watch,
        } => {
            if watch {
                cmd_check_watch(&cfg, &files, &fmt, use_color, &fail_on, baseline.as_deref())
            } else {
                cmd_check(
                    &cfg,
                    &files,
                    &fmt,
                    stdin,
                    use_color,
                    &fail_on,
                    baseline.as_deref(),
                )
            }
        }
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
        Cmd::Profile { files, top, json } => cmd_profile(&cfg, &files, top, json),
        Cmd::Docs { output, check } => cmd_docs(&output, check),
        Cmd::Init { force, output } => cmd_init(&output, force),
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
            for ext in &[
                "sql",
                "pgsql",
                "mysql",
                "sqlite",
                "snowflake",
                "snowsql",
                "tsql",
                "mssql",
            ] {
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
        // honour `-- drift:disable[-next] rule_id` line comments first; user
        // intent always beats the rule registry.
        let disables = crate::disables::scan(&src);
        let viols_post_disable = crate::disables::filter_violations(&disables, &viols_raw);
        // baseline keyed by path; stdin uses "<stdin>" which is unlikely to be in
        // a baseline file. apply anyway for symmetry.
        let viols: Vec<_> = match &baseline {
            Some(bl) => bl.filter_violations("<stdin>", &viols_post_disable),
            None => viols_post_disable,
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

/// re-run `drift check` whenever a file under `paths` changes. uses
/// notify-debouncer-mini so a flurry of editor saves coalesces into a single
/// re-lint within ~250 ms instead of once-per-fsync.
fn cmd_check_watch(
    cfg: &Config,
    files: &[PathBuf],
    fmt: &str,
    color: bool,
    fail_on: &str,
    baseline_path: Option<&Path>,
) -> Result<i32> {
    use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
    use std::sync::mpsc;
    use std::time::Duration;

    // print once eagerly so the first run lands without waiting on a file event.
    let _ = cmd_check(cfg, files, fmt, false, color, fail_on, baseline_path);
    eprintln!("\ndrift: watching for changes (ctrl-c to stop)");

    let watch_paths: Vec<PathBuf> = if files.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        files.to_vec()
    };

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(250), move |res| {
        let _ = tx.send(res);
    })?;
    for p in &watch_paths {
        let target = if p.is_dir() {
            p.clone()
        } else {
            PathBuf::from(p.parent().unwrap_or(Path::new(".")))
        };
        let _ = debouncer.watcher().watch(&target, RecursiveMode::Recursive);
    }

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let touched_sql = events.iter().any(|e| {
                    e.path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| {
                            matches!(
                                ext,
                                "sql"
                                    | "pgsql"
                                    | "mysql"
                                    | "sqlite"
                                    | "snowflake"
                                    | "snowsql"
                                    | "tsql"
                                    | "mssql"
                            )
                        })
                        .unwrap_or(false)
                });
                if !touched_sql {
                    continue;
                }
                println!();
                let _ = cmd_check(cfg, files, fmt, false, color, fail_on, baseline_path);
                eprintln!("drift: watching for changes (ctrl-c to stop)");
            }
            Ok(Err(e)) => {
                eprintln!("drift: watch error: {e}");
            }
            Err(_) => break, // channel closed
        }
    }
    Ok(0)
}

/// `drift profile` — aggregate violations by rule across a corpus.
/// the use case is "drift just emitted 4000 warnings, what's the noise floor
/// and what could i disable?". prints a top-N table and a drift.toml fragment
/// so adoption is one paste away.
fn cmd_profile(cfg: &Config, files: &[PathBuf], top: usize, json: bool) -> Result<i32> {
    use std::collections::BTreeMap;

    let registry = Registry::new();
    let files = expand(files);
    if files.is_empty() {
        eprintln!("no files matched");
        return Ok(0);
    }

    let started = std::time::Instant::now();
    let results: Vec<Vec<crate::Violation>> = files
        .par_iter()
        .filter_map(|path| {
            let src = std::fs::read_to_string(path).ok()?;
            let dialect = pick_dialect(cfg, path);
            let parsed = parse(&src, dialect);
            Some(registry.run(&parsed, cfg))
        })
        .collect();
    let elapsed_ms = started.elapsed().as_millis();

    let mut counts: BTreeMap<&'static str, (usize, Severity)> = BTreeMap::new();
    let mut total = 0usize;
    let mut by_severity: BTreeMap<&'static str, usize> = BTreeMap::new();
    for viols in &results {
        for v in viols {
            counts
                .entry(v.rule_id)
                .and_modify(|e| e.0 += 1)
                .or_insert((1, v.severity));
            total += 1;
            *by_severity.entry(v.severity.as_str()).or_insert(0) += 1;
        }
    }

    let mut rows: Vec<(&'static str, usize, Severity)> =
        counts.into_iter().map(|(k, (n, s))| (k, n, s)).collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1));

    if json {
        #[derive(serde::Serialize)]
        struct Row {
            rule: &'static str,
            count: usize,
            severity: &'static str,
        }
        let out: Vec<Row> = rows
            .iter()
            .take(top)
            .map(|(r, n, s)| Row {
                rule: r,
                count: *n,
                severity: s.as_str(),
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "files": files.len(),
                "elapsed_ms": elapsed_ms,
                "total_violations": total,
                "by_severity": by_severity,
                "top_rules": out,
            }))
            .unwrap()
        );
        return Ok(0);
    }

    println!("scanned {} files in {} ms", files.len(), elapsed_ms);
    println!("total violations: {total}");
    if !by_severity.is_empty() {
        let parts: Vec<String> = by_severity
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect();
        println!("by severity:     {}", parts.join(", "));
    }
    println!();

    if rows.is_empty() {
        println!("nothing to profile.");
        return Ok(0);
    }

    println!("top {} rules:", rows.len().min(top));
    println!("  count  severity    rule");
    for (rule, n, sev) in rows.iter().take(top) {
        println!("  {:>5}  {:<10}  {}", n, sev.as_str(), rule);
    }

    // suggest a drift.toml block. anything firing >5% of total goes to a
    // "noisy, consider warning instead of error" pile. zero-firing rules
    // are silent (no recommendation; their default may still be useful).
    let noisy_threshold = (total / 20).max(20);
    let noisy: Vec<_> = rows
        .iter()
        .filter(|(_, n, s)| *n >= noisy_threshold && matches!(s, Severity::Error))
        .collect();
    if !noisy.is_empty() {
        println!();
        println!("suggested drift.toml (rules above 5% of all hits, demoted to warning):");
        println!();
        println!("  [rules]");
        for (rule, n, _) in &noisy {
            println!(r#"  "{rule}" = "warning"   # {n} hits"#);
        }
    }

    Ok(0)
}

/// `drift docs [--check]` — write one markdown page per rule into the output
/// directory, derived 1:1 from the Rule trait. `--check` makes the command
/// fail with non-zero exit if any file on disk differs from what would be
/// generated, so ci can fence the docs against drift in either direction.
fn cmd_docs(output: &Path, check: bool) -> Result<i32> {
    use std::fmt::Write as _;

    let registry = Registry::new();
    let rules = registry.rules();
    std::fs::create_dir_all(output)?;

    let mut differing = 0usize;
    let mut written = 0usize;

    for rule in rules.iter() {
        let mut md = String::new();
        let id = rule.id();
        let _ = writeln!(md, "# `{id}`");
        let _ = writeln!(md);
        let _ = writeln!(
            md,
            "| field    | value |\n|----------|-------|\n| category | `{}` |\n| default  | `{}` |\n| fixable  | `{}` |",
            rule.category().as_str(),
            rule.default_severity().as_str(),
            if rule.fixable() { "yes" } else { "no" },
        );
        let _ = writeln!(md);
        let _ = writeln!(md, "## what");
        let _ = writeln!(md);
        let _ = writeln!(md, "{}", rule.description().trim());
        let _ = writeln!(md);
        if !rule.example_bad().is_empty() {
            let _ = writeln!(md, "## bad");
            let _ = writeln!(md);
            let _ = writeln!(md, "```sql");
            let _ = writeln!(md, "{}", rule.example_bad().trim_end());
            let _ = writeln!(md, "```");
            let _ = writeln!(md);
        }
        if !rule.example_good().is_empty() {
            let _ = writeln!(md, "## good");
            let _ = writeln!(md);
            let _ = writeln!(md, "```sql");
            let _ = writeln!(md, "{}", rule.example_good().trim_end());
            let _ = writeln!(md);
            let _ = writeln!(md, "```");
            let _ = writeln!(md);
        }
        let _ = writeln!(
            md,
            "_generated by `drift docs`. edit the rule definition in `src/rules/`, not this file._"
        );

        let path = output.join(format!("{id}.md"));
        if check {
            let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
            if on_disk != md {
                differing += 1;
                eprintln!("docs out of date: {}", path.display());
            }
        } else {
            std::fs::write(&path, md)?;
            written += 1;
        }
    }

    // also write an index.
    let mut index = String::new();
    let _ = writeln!(&mut index, "# rule index\n\n{} rules.\n", rules.len());
    let mut by_cat: std::collections::BTreeMap<&'static str, Vec<&dyn crate::Rule>> =
        std::collections::BTreeMap::new();
    for r in rules.iter() {
        by_cat.entry(r.category().as_str()).or_default().push(&**r);
    }
    for (cat, rs) in &by_cat {
        let _ = writeln!(&mut index, "## {}\n", cat);
        for r in rs {
            let _ = writeln!(
                &mut index,
                "- [`{}`](./{}.md) — {}",
                r.id(),
                r.id(),
                r.description().trim().lines().next().unwrap_or("")
            );
        }
        let _ = writeln!(&mut index);
    }
    let index_path = output.join("README.md");
    if check {
        let on_disk = std::fs::read_to_string(&index_path).unwrap_or_default();
        if on_disk != index {
            differing += 1;
            eprintln!("docs out of date: {}", index_path.display());
        }
    } else {
        std::fs::write(&index_path, index)?;
        written += 1;
    }

    if check {
        if differing > 0 {
            eprintln!(
                "{differing} rule doc(s) differ from generator output. run `drift docs` and commit."
            );
            return Ok(1);
        }
        eprintln!("all rule docs in sync ({} pages checked)", rules.len() + 1);
    } else {
        eprintln!("wrote {written} pages into {}", output.display());
    }
    Ok(0)
}

/// `drift init [--force] [--output FILE]` — scaffold a starter config.
/// the template is hand-tuned to be useful out of the box (correctness
/// rules promoted to error, opinion-heavy categories left commented out).
fn cmd_init(output: &Path, force: bool) -> Result<i32> {
    if output.exists() && !force {
        anyhow::bail!(
            "{} already exists. pass --force to overwrite, or --output FILE to write somewhere else.",
            output.display()
        );
    }
    std::fs::write(output, STARTER_TOML)?;
    eprintln!(
        "wrote {} ({} bytes). open it, keep what fits your team.",
        output.display(),
        STARTER_TOML.len()
    );
    eprintln!("then: drift check **/*.sql");
    Ok(0)
}

const STARTER_TOML: &str = r#"# drift.toml — generated by `drift init`
# docs:  https://drift.frkhd.com
# rules: https://github.com/f4rkh4d/drift/tree/main/docs/rules
#
# tweak this file, commit it, then `drift check **/*.sql`.

[drift]
# postgres | mysql | sqlite | bigquery | snowflake | tsql | ansi
dialect = "postgres"
# strict | warn | compat — sets baseline severity before per-rule overrides.
preset  = "warn"

[rules]
# severity = "error" | "warning" | "info" | "off"
# wildcards work at the leaf: "drift.correctness.*" = "error"

# correctness rules are usually worth promoting to errors.
"drift.correctness.missing-where-update" = "error"
"drift.correctness.missing-where-delete" = "error"
"drift.correctness.null-equality"        = "error"
"drift.correctness.cartesian-join"       = "error"

# style: keep light, fix mode handles most of these.
"drift.style.keyword-case"     = { severity = "warning", case = "upper" }
"drift.style.trailing-newline" = "warning"

# performance: warnings, not errors. context matters.
"drift.performance.select-star"           = "warning"
"drift.performance.like-leading-wildcard" = "warning"

# security: errors when in scope. comment out if you have your own scanner.
"drift.security.dynamic-sql-concat" = "error"
"drift.security.plaintext-password" = "error"

# conventions ship off by default. opt in only when your team agrees.
# "drift.conventions.snake-case-tables" = "warning"
# "drift.conventions.plural-table-name" = "warning"

[format]
indent       = 2
max-line     = 100
keyword-case = "upper"
"#;
