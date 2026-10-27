//! PostgreSQL schema migrations run under the database's cross-process transaction lock.
use crate::platform::{
    database::Connection,
    error::{Result, internal},
};

const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_postgresql.sql"),
    include_str!("../migrations/002_accounts.sql"),
    include_str!("../migrations/003_github_sync.sql"),
    include_str!("../migrations/004_object_storage.sql"),
];

pub(crate) fn run(c: &mut Connection<'_>) -> Result<()> {
    c.batch_execute("CREATE TABLE IF NOT EXISTS saturn_schema (version BIGINT PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT now());").map_err(internal)?;
    let version: i64 = c
        .query_one("SELECT COALESCE(MAX(version),0) FROM saturn_schema", &[])
        .map_err(internal)?
        .get(0);
    if version > MIGRATIONS.len() as i64 {
        return Err(internal("Database schema is newer than this server"));
    }
    for (index, migration) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        c.batch_execute(migration).map_err(internal)?;
        c.execute(
            "INSERT INTO saturn_schema(version) VALUES($1)",
            &[&((index + 1) as i64)],
        )
        .map_err(internal)?;
    }
    Ok(())
}
