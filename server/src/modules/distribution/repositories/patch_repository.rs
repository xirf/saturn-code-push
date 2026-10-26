use super::super::models::Patch;
use crate::platform::error::{Result, internal};
use rusqlite::{Connection, params};

pub(in crate::modules::distribution) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS distribution_patches (
                id INTEGER PRIMARY KEY,
                release_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                channel_ids TEXT NOT NULL,
                is_rolled_back INTEGER NOT NULL CHECK (is_rolled_back IN (0, 1)),
                created_at TEXT NOT NULL,
                FOREIGN KEY(release_id) REFERENCES distribution_releases(id),
                UNIQUE(release_id,number),
                UNIQUE(id,release_id)
            );",
        )
        .map_err(internal)
}

pub(in crate::modules::distribution) fn list(connection: &Connection) -> Result<Vec<Patch>> {
    let mut statement = connection
        .prepare(
            "SELECT
                id,
                release_id,
                number,
                channel_ids,
                is_rolled_back,
                created_at
         FROM distribution_patches ORDER BY rowid",
        )
        .map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Patch {
                id: row.get(0)?,
                release_id: row.get(1)?,
                number: row.get(2)?,
                channel_ids: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?,
                is_rolled_back: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::distribution) fn save(
    connection: &Connection,
    entity: &Patch,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO distribution_patches (
                id,
                release_id,
                number,
                channel_ids,
                is_rolled_back,
                created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                release_id = excluded.release_id,
                number = excluded.number,
                channel_ids = excluded.channel_ids,
                is_rolled_back = excluded.is_rolled_back,
                created_at = excluded.created_at",
            params![
                entity.id,
                entity.release_id,
                entity.number,
                serde_json::to_string(&entity.channel_ids).map_err(internal)?,
                entity.is_rolled_back,
                entity.created_at,
            ],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::distribution) fn find(connection: &Connection, id: i64) -> Result<Patch> {
    list(connection)?
        .into_iter()
        .find(|entity| entity.id == id)
        .ok_or_else(crate::platform::error::missing)
}
