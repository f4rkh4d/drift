//! baseline file: a snapshot of currently-known violations.
//!
//! the use case is adopting drift on a legacy codebase. running `drift check`
//! cold against thousands of legacy SQL files emits thousands of warnings;
//! nobody fixes them, the team decides drift is annoying, drift dies. with a
//! baseline, the team runs `drift baseline create` once, commits the resulting
//! `.drift-baseline.json`, and from then on `drift check --baseline` only
//! flags NEW violations. the legacy debt is locked in, but it cannot grow.
//!
//! ## matching strategy
//!
//! a count-per-(file, rule_id) keyed dictionary. if the baseline says
//! `migrations/0042.sql` had 3 hits of `drift.style.keyword-case`, the next
//! run silences up to 3 hits of that rule in that file and fails on the 4th
//! and beyond. line numbers are intentionally NOT used as a key: code edits
//! shift them and we would either suppress real new violations or surface
//! noise on the very first reformat. the count-based approach is what ruff,
//! biome, and eslint use for the same reason.
//!
//! ## file format
//!
//! ```json
//! {
//!   "schema": 1,
//!   "drift_version": "0.15.0",
//!   "created_at": "2026-05-05T12:00:00Z",
//!   "files": {
//!     "migrations/0042.sql": {
//!       "drift.correctness.null-equality": 1,
//!       "drift.performance.select-star": 2
//!     }
//!   }
//! }
//! ```

use crate::report::FileReport;
use crate::rules::Violation;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const DEFAULT_BASELINE_PATH: &str = ".drift-baseline.json";
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub schema: u32,
    #[serde(default)]
    pub drift_version: String,
    #[serde(default)]
    pub created_at: String,
    pub files: BTreeMap<String, BTreeMap<String, usize>>,
}

impl Baseline {
    /// build a baseline from the violations of a check run.
    pub fn from_reports(reports: &[FileReport]) -> Self {
        let mut files: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for r in reports {
            if r.violations.is_empty() {
                continue;
            }
            let bucket = files.entry(r.path.to_string()).or_default();
            for v in r.violations {
                *bucket.entry(v.rule_id.to_string()).or_insert(0) += 1;
            }
        }
        Self {
            schema: SCHEMA_VERSION,
            drift_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now_iso8601(),
            files,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading baseline at {}", path.display()))?;
        let bl: Self = serde_json::from_str(&raw)
            .with_context(|| format!("parsing baseline at {}", path.display()))?;
        if bl.schema != SCHEMA_VERSION {
            anyhow::bail!(
                "baseline schema {} is not supported (this drift expects {}). \
                 run `drift baseline create` to regenerate.",
                bl.schema,
                SCHEMA_VERSION
            );
        }
        Ok(bl)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let raw = serde_json::to_string_pretty(self)
            .context("serializing baseline to json")?;
        std::fs::write(path, raw)
            .with_context(|| format!("writing baseline to {}", path.display()))?;
        Ok(())
    }

    /// returns true if this baseline has at least one entry (any file, any rule).
    pub fn is_empty(&self) -> bool {
        self.files.values().all(|m| m.is_empty())
    }

    /// total number of suppressed violations the baseline accounts for.
    pub fn total(&self) -> usize {
        self.files.values().flat_map(|m| m.values()).sum()
    }

    /// drop violations covered by the baseline. for each (path, rule) pair
    /// where the baseline has count `n`, the first `n` matching violations
    /// in `viols` are removed. surplus violations remain.
    pub fn filter_violations(&self, path: &str, viols: &[Violation]) -> Vec<Violation> {
        let Some(file_bucket) = self.files.get(path) else {
            return viols.to_vec();
        };
        let mut remaining: BTreeMap<&str, usize> =
            file_bucket.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        let mut out = Vec::with_capacity(viols.len());
        for v in viols {
            if let Some(slot) = remaining.get_mut(v.rule_id) {
                if *slot > 0 {
                    *slot -= 1;
                    continue; // suppressed
                }
            }
            out.push(v.clone());
        }
        out
    }
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // simple iso8601 in utc; avoids pulling in chrono just for a header field.
    let (year, month, day, hour, minute, second) = unix_to_ymdhms(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn unix_to_ymdhms(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    // proleptic gregorian, days since 1970-01-01.
    let days = (secs / 86_400) as i64;
    let rem = (secs % 86_400) as u32;
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;
    let second = rem % 60;

    // howard hinnant's date algorithm: days since civil 1970-01-01 to (y, m, d).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { (mp + 3) as u32 } else { (mp - 9) as u32 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, hour, minute, second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Severity, Violation};

    fn v(rule: &'static str) -> Violation {
        Violation {
            rule_id: rule,
            severity: Severity::Warning,
            message: String::new(),
            line: 1,
            col: 1,
            span: None,
            fix: None,
        }
    }

    fn report<'a>(path: &'a str, viols: &'a [Violation]) -> FileReport<'a> {
        FileReport {
            path,
            source: "",
            violations: viols,
        }
    }

    #[test]
    fn from_reports_counts_per_file_and_rule() {
        let v_a = vec![v("rule.a"), v("rule.a"), v("rule.b")];
        let v_b = vec![v("rule.a")];
        let r_a = report("a.sql", &v_a);
        let r_b = report("b.sql", &v_b);
        let bl = Baseline::from_reports(&[r_a, r_b]);

        assert_eq!(bl.files.len(), 2);
        assert_eq!(bl.files["a.sql"]["rule.a"], 2);
        assert_eq!(bl.files["a.sql"]["rule.b"], 1);
        assert_eq!(bl.files["b.sql"]["rule.a"], 1);
        assert_eq!(bl.total(), 4);
        assert!(!bl.is_empty());
    }

    #[test]
    fn filter_suppresses_exactly_count_violations() {
        let snapshot = vec![v("rule.a"), v("rule.a"), v("rule.b")];
        let snap_report = report("x.sql", &snapshot);
        let bl = Baseline::from_reports(&[snap_report]);

        // identical run: every violation suppressed.
        assert!(bl.filter_violations("x.sql", &snapshot).is_empty());

        // one extra rule.a: that extra surfaces.
        let mut more = snapshot.clone();
        more.push(v("rule.a"));
        let surplus = bl.filter_violations("x.sql", &more);
        assert_eq!(surplus.len(), 1);
        assert_eq!(surplus[0].rule_id, "rule.a");

        // a brand new rule.c: surfaces in full because baseline has no slot for it.
        let mut new_rule = snapshot.clone();
        new_rule.push(v("rule.c"));
        let surplus = bl.filter_violations("x.sql", &new_rule);
        assert_eq!(surplus.len(), 1);
        assert_eq!(surplus[0].rule_id, "rule.c");
    }

    #[test]
    fn filter_passes_through_unknown_files() {
        let bl_input = vec![v("r")];
        let r = report("known.sql", &bl_input);
        let bl = Baseline::from_reports(&[r]);

        let new = vec![v("r")];
        let pass = bl.filter_violations("unknown.sql", &new);
        assert_eq!(pass.len(), 1, "unknown files are not suppressed");
    }

    #[test]
    fn save_then_load_roundtrip() {
        let viols = vec![v("rule.x"), v("rule.x"), v("rule.y")];
        let r = report("p.sql", &viols);
        let bl = Baseline::from_reports(&[r]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("baseline.json");
        bl.save(&path).unwrap();

        let loaded = Baseline::load(&path).unwrap();
        assert_eq!(loaded.schema, SCHEMA_VERSION);
        assert_eq!(loaded.files["p.sql"]["rule.x"], 2);
        assert_eq!(loaded.files["p.sql"]["rule.y"], 1);
    }

    #[test]
    fn rejects_unknown_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("baseline.json");
        std::fs::write(
            &path,
            r#"{"schema": 999, "drift_version": "x", "created_at": "x", "files": {}}"#,
        )
        .unwrap();
        let err = Baseline::load(&path).unwrap_err();
        assert!(err.to_string().contains("schema 999"));
    }

    #[test]
    fn iso8601_formatter_known_epoch() {
        let (y, m, d, hh, mm, ss) = unix_to_ymdhms(0);
        assert_eq!((y, m, d, hh, mm, ss), (1970, 1, 1, 0, 0, 0));
        let (y, m, d, hh, mm, ss) = unix_to_ymdhms(1_577_836_800); // 2020-01-01T00:00:00Z
        assert_eq!((y, m, d, hh, mm, ss), (2020, 1, 1, 0, 0, 0));
        let (y, m, d, hh, mm, ss) = unix_to_ymdhms(1_577_836_800 + 86_400 + 3661);
        assert_eq!((y, m, d, hh, mm, ss), (2020, 1, 2, 1, 1, 1));
    }
}
