use super::super::models::Session;
use crate::platform::error::{Result, internal};
use rusqlite::{Connection, params};

pub(crate) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS dashboard_sessions (
            token_hash TEXT PRIMARY KEY,
            identity TEXT NOT NULL,
            expires_at INTEGER NOT NULL
        );",
        )
        .map_err(internal)
}

pub(in crate::modules::identity) fn exists(
    connection: &Connection,
    token_hash: &str,
    identity: &str,
    now: i64,
) -> Result<bool> {
    connection
        .query_row(
            "SELECT EXISTS (
            SELECT 1 FROM dashboard_sessions
            WHERE token_hash = ?1 AND identity = ?2 AND expires_at > ?3
        )",
            params![token_hash, identity, now],
            |row| row.get(0),
        )
        .map_err(internal)
}

pub(in crate::modules::identity) fn remove(
    connection: &Connection,
    token_hash: &str,
) -> Result<()> {
    connection
        .execute(
            "DELETE FROM dashboard_sessions WHERE token_hash = ?1",
            [token_hash],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::identity) fn rotate(
    connection: &Connection,
    session: &Session,
    old_hash: Option<&str>,
    now: i64,
) -> Result<()> {
    connection
        .execute(
            "DELETE FROM dashboard_sessions WHERE expires_at <= ?1 OR identity != ?2",
            params![now, session.identity],
        )
        .map_err(internal)?;
    if let Some(old_hash) = old_hash {
        remove(connection, old_hash)?;
    }
    connection
        .execute(
            "INSERT INTO dashboard_sessions (token_hash, identity, expires_at)
         VALUES (?1, ?2, ?3)",
            params![session.token_hash, session.identity, session.expires_at],
        )
        .map_err(internal)?;
    Ok(())
}
