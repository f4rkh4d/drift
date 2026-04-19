//! rule trait + registry.
//!
//! every rule implements `Rule` and is registered in `all_rules()`. the
//! registry holds name → instance and applies the user's config (severity
//! override, per-rule options) before running.

use crate::config::Config;
use crate::parse::Parsed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod ambiguity;
pub mod conventions;
pub mod correctness;
pub mod performance;
pub mod portability;
pub mod security;
pub mod style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Off,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Off => "off",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Style,
    Correctness,
    Performance,
    Security,
    Portability,
    Conventions,
    Ambiguity,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Style => "style",
            Category::Correctness => "correctness",
            Category::Performance => "performance",
            Category::Security => "security",
            Category::Portability => "portability",
            Category::Conventions => "conventions",
            Category::Ambiguity => "ambiguity",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub col: usize,
    /// byte offset range in source. used by the fixer.
    pub span: Option<(usize, usize)>,
    /// proposed replacement text if rule is fixable in-place.
    pub fix: Option<String>,
}

pub trait Rule: Sync + Send {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;
    fn default_severity(&self) -> Severity;
    fn description(&self) -> &'static str;
    fn example_bad(&self) -> &'static str {
        ""
    }
    fn example_good(&self) -> &'static str {
        ""
    }
    fn fixable(&self) -> bool {
        false
    }
    fn check(&self, parsed: &Parsed, config: &Config) -> Vec<Violation>;
}

pub struct Registry {
    rules: Vec<Box<dyn Rule>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { rules: all_rules() }
    }

    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    pub fn get(&self, id: &str) -> Option<&dyn Rule> {
        self.rules.iter().find(|r| r.id() == id).map(|r| r.as_ref())
    }

    pub fn by_category(&self) -> BTreeMap<Category, Vec<&dyn Rule>> {
        let mut map: BTreeMap<Category, Vec<&dyn Rule>> = BTreeMap::new();
        for r in &self.rules {
            map.entry(r.category()).or_default().push(r.as_ref());
        }
        map
    }

    pub fn run(&self, parsed: &Parsed, config: &Config) -> Vec<Violation> {
        let mut out = Vec::new();
        for r in &self.rules {
            let sev = config.effective_severity(r.id(), r.default_severity());
            if sev == Severity::Off {
                continue;
            }
            for mut v in r.check(parsed, config) {
                v.severity = sev;
                out.push(v);
            }
        }
        out.sort_by_key(|v| (v.line, v.col, v.rule_id));
        out
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Category {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

/// every rule ships here. order defines ID stability but not runtime order.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut v: Vec<Box<dyn Rule>> = Vec::new();
    style::register(&mut v);
    correctness::register(&mut v);
    performance::register(&mut v);
    security::register(&mut v);
    portability::register(&mut v);
    conventions::register(&mut v);
    ambiguity::register(&mut v);
    v
}
