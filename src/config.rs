//! config: toml file, presets, per-rule overrides.

use crate::dialect::Dialect;
use crate::rules::Severity;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub drift: RawDrift,
    #[serde(default)]
    pub rules: BTreeMap<String, RawRule>,
    pub format: RawFormat,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RawDrift {
    pub dialect: Option<String>,
    pub preset: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RawFormat {
    pub indent: Option<usize>,
    #[serde(rename = "max-line")]
    pub max_line: Option<usize>,
    #[serde(rename = "keyword-case")]
    pub keyword_case: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawRule {
    Short(String),
    Full {
        severity: Option<String>,
        #[serde(default, rename = "case")]
        case: Option<String>,
        #[serde(default)]
        #[serde(flatten)]
        extra: BTreeMap<String, toml::Value>,
    },
}

#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub severity: Option<Severity>,
    pub case: Option<KeywordCase>,
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordCase {
    Upper,
    Lower,
}

impl KeywordCase {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "upper" | "uppercase" => Some(Self::Upper),
            "lower" | "lowercase" => Some(Self::Lower),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Preset {
    Strict,
    Warn,
    Compat,
    None,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dialect: Option<Dialect>,
    pub preset: Preset,
    pub rules: BTreeMap<String, RuleConfig>,
    pub format: FormatConfig,
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub indent: usize,
    pub max_line: usize,
    pub keyword_case: KeywordCase,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            max_line: 100,
            keyword_case: KeywordCase::Upper,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dialect: None,
            preset: Preset::None,
            rules: BTreeMap::new(),
            format: FormatConfig::default(),
        }
    }
}

impl Config {
    pub fn from_toml_str(s: &str) -> anyhow::Result<Self> {
        let raw: RawConfig = toml::from_str(s)?;
        Self::from_raw(raw)
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Self::from_toml_str(&s)
    }

    /// walk upward looking for `drift.toml`.
    pub fn discover(start: &Path) -> Option<std::path::PathBuf> {
        let mut cur = Some(start.to_path_buf());
        while let Some(p) = cur {
            let candidate = p.join("drift.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            cur = p.parent().map(|x| x.to_path_buf());
        }
        None
    }

    fn from_raw(raw: RawConfig) -> anyhow::Result<Self> {
        let dialect = raw
            .drift
            .dialect
            .as_deref()
            .map(|s| s.parse::<Dialect>().map_err(anyhow::Error::msg))
            .transpose()?;

        let preset = match raw.drift.preset.as_deref() {
            Some("strict") => Preset::Strict,
            Some("warn") => Preset::Warn,
            Some("compat") => Preset::Compat,
            Some(other) => anyhow::bail!("unknown preset: {other}"),
            None => Preset::None,
        };

        let mut rules = BTreeMap::new();
        for (k, v) in raw.rules {
            rules.insert(k, decode_rule(v)?);
        }

        let format = FormatConfig {
            indent: raw.format.indent.unwrap_or(2),
            max_line: raw.format.max_line.unwrap_or(100),
            keyword_case: raw
                .format
                .keyword_case
                .as_deref()
                .and_then(KeywordCase::parse)
                .unwrap_or(KeywordCase::Upper),
        };

        Ok(Config {
            dialect,
            preset,
            rules,
            format,
        })
    }

    pub fn effective_severity(&self, rule_id: &str, default: Severity) -> Severity {
        // exact match wins
        if let Some(rc) = self.rules.get(rule_id) {
            if let Some(s) = rc.severity {
                return s;
            }
        }
        // wildcard match: drift.<category>.*
        if let Some(dot) = rule_id.rfind('.') {
            let prefix = &rule_id[..dot];
            let wildcard = format!("{prefix}.*");
            if let Some(rc) = self.rules.get(&wildcard) {
                if let Some(s) = rc.severity {
                    return s;
                }
            }
        }
        // preset
        match self.preset {
            Preset::Strict => Severity::Error,
            Preset::Warn => Severity::Warning,
            Preset::Compat => {
                if rule_id.starts_with("drift.correctness.")
                    || rule_id.starts_with("drift.security.")
                {
                    default
                } else {
                    Severity::Off
                }
            }
            Preset::None => default,
        }
    }

    pub fn rule_option(&self, rule_id: &str, key: &str) -> Option<&toml::Value> {
        self.rules.get(rule_id)?.extra.get(key)
    }

    pub fn rule_case(&self, rule_id: &str) -> Option<KeywordCase> {
        self.rules.get(rule_id)?.case
    }
}

fn decode_rule(raw: RawRule) -> anyhow::Result<RuleConfig> {
    match raw {
        RawRule::Short(s) => Ok(RuleConfig {
            severity: Some(parse_severity(&s)?),
            case: None,
            extra: BTreeMap::new(),
        }),
        RawRule::Full {
            severity,
            case,
            extra,
        } => {
            let severity = severity.as_deref().map(parse_severity).transpose()?;
            Ok(RuleConfig {
                severity,
                case: case.as_deref().and_then(KeywordCase::parse),
                extra,
            })
        }
    }
}

fn parse_severity(s: &str) -> anyhow::Result<Severity> {
    match s.to_ascii_lowercase().as_str() {
        "error" => Ok(Severity::Error),
        "warning" | "warn" => Ok(Severity::Warning),
        "info" => Ok(Severity::Info),
        "off" | "none" => Ok(Severity::Off),
        other => anyhow::bail!("unknown severity: {other}"),
    }
}
