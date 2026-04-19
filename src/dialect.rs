//! dialect detection and mapping to sqlparser dialect impls.

use serde::{Deserialize, Serialize};
use sqlparser::dialect::{
    AnsiDialect, BigQueryDialect, Dialect as SqlDialect, GenericDialect, MySqlDialect,
    PostgreSqlDialect, SQLiteDialect,
};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    Postgres,
    MySql,
    Sqlite,
    BigQuery,
    #[default]
    Ansi,
}

impl Dialect {
    pub fn as_parser(&self) -> Box<dyn SqlDialect> {
        match self {
            Dialect::Postgres => Box::new(PostgreSqlDialect {}),
            Dialect::MySql => Box::new(MySqlDialect {}),
            Dialect::Sqlite => Box::new(SQLiteDialect {}),
            Dialect::BigQuery => Box::new(BigQueryDialect {}),
            Dialect::Ansi => Box::new(AnsiDialect {}),
        }
    }

    /// fallback dialect when we just need to tokenize anything.
    pub fn generic() -> Box<dyn SqlDialect> {
        Box::new(GenericDialect {})
    }

    pub fn detect_from_path(path: &Path) -> Option<Dialect> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "pgsql" | "psql" => Some(Dialect::Postgres),
            "mysql" | "mariadb" => Some(Dialect::MySql),
            "sqlite" | "sqlite3" => Some(Dialect::Sqlite),
            "bq" | "bigquery" => Some(Dialect::BigQuery),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Dialect::Postgres => "postgres",
            Dialect::MySql => "mysql",
            Dialect::Sqlite => "sqlite",
            Dialect::BigQuery => "bigquery",
            Dialect::Ansi => "ansi",
        }
    }
}

impl FromStr for Dialect {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Ok(Dialect::Postgres),
            "mysql" | "mariadb" => Ok(Dialect::MySql),
            "sqlite" | "sqlite3" => Ok(Dialect::Sqlite),
            "bigquery" | "bq" => Ok(Dialect::BigQuery),
            "ansi" | "standard" => Ok(Dialect::Ansi),
            other => Err(format!("unknown dialect: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_extensions() {
        assert_eq!(
            Dialect::detect_from_path(Path::new("q.pgsql")),
            Some(Dialect::Postgres)
        );
        assert_eq!(
            Dialect::detect_from_path(Path::new("q.mysql")),
            Some(Dialect::MySql)
        );
        assert_eq!(Dialect::detect_from_path(Path::new("q.sql")), None);
    }

    #[test]
    fn parses_from_string() {
        assert_eq!("postgres".parse::<Dialect>().unwrap(), Dialect::Postgres);
        assert_eq!("bq".parse::<Dialect>().unwrap(), Dialect::BigQuery);
        assert!("oracle".parse::<Dialect>().is_err());
    }
}
