use super::super::models::Channel;
use crate::platform::error::{Result, internal};
use rusqlite::{Connection, params};

pub(in crate::modules::applications) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS applications_channels (
                id INTEGER PRIMARY KEY,
                app_id TEXT NOT NULL,
                name TEXT NOT NULL,
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                UNIQUE(app_id,name)
            );",
        )
        .map_err(internal)
}

pub(in crate::modules::applications) fn list(connection: &Connection) -> Result<Vec<Channel>> {
    let mut statement = connection
        .prepare(
            "SELECT
                id,
                app_id,
                name
         FROM applications_channels ORDER BY rowid",
        )
        .map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Channel {
                id: row.get(0)?,
                app_id: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::applications) fn save(
    connection: &Connection,
    entity: &Channel,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO applications_channels (
                id,
                app_id,
                name
            ) VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                app_id = excluded.app_id,
                name = excluded.name",
            params![entity.id, entity.app_id, entity.name,],
        )
        .map_err(internal)?;
    Ok(())
}
