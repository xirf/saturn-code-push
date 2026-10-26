use super::super::models::App;
use crate::platform::error::{Result, internal};
use rusqlite::{Connection, params};

pub(in crate::modules::applications) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS applications_apps (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )
        .map_err(internal)
}

pub(in crate::modules::applications) fn list(connection: &Connection) -> Result<Vec<App>> {
    let mut statement = connection
        .prepare(
            "SELECT
                id,
                display_name,
                created_at
         FROM applications_apps ORDER BY rowid",
        )
        .map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            Ok(App {
                id: row.get(0)?,
                display_name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::applications) fn save(connection: &Connection, entity: &App) -> Result<()> {
    connection
        .execute(
            "INSERT INTO applications_apps (
                id,
                display_name,
                created_at
            ) VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                created_at = excluded.created_at",
            params![entity.id, entity.display_name, entity.created_at,],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::applications) fn find(connection: &Connection, id: &str) -> Result<App> {
    list(connection)?
        .into_iter()
        .find(|entity| entity.id == id)
        .ok_or_else(crate::platform::error::missing)
}
