use drift::dialect::Dialect;
use std::path::Path;

#[test]
fn detects_postgres_extensions() {
    assert_eq!(
        Dialect::detect_from_path(Path::new("q.pgsql")),
        Some(Dialect::Postgres)
    );
    assert_eq!(
        Dialect::detect_from_path(Path::new("q.psql")),
        Some(Dialect::Postgres)
    );
}

#[test]
fn detects_mysql() {
    assert_eq!(
        Dialect::detect_from_path(Path::new("q.mysql")),
        Some(Dialect::MySql)
    );
}

#[test]
fn detects_sqlite() {
    assert_eq!(
        Dialect::detect_from_path(Path::new("x.sqlite")),
        Some(Dialect::Sqlite)
    );
}

#[test]
fn detects_bigquery() {
    assert_eq!(
        Dialect::detect_from_path(Path::new("q.bq")),
        Some(Dialect::BigQuery)
    );
}

#[test]
fn plain_sql_has_no_detected_dialect() {
    assert_eq!(Dialect::detect_from_path(Path::new("x.sql")), None);
}

#[test]
fn parse_round_trip_string() {
    for name in ["postgres", "mysql", "sqlite", "bigquery", "ansi"] {
        let d: Dialect = name.parse().unwrap();
        assert_eq!(d.name(), name);
    }
}
