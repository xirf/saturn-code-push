use super::super::models::Release;
use crate::platform::error::{Result, internal};
use rusqlite::{Connection, params};

pub(in crate::modules::distribution) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS distribution_releases (
                id INTEGER PRIMARY KEY,
                app_id TEXT NOT NULL,
                version TEXT NOT NULL,
                flutter_revision TEXT NOT NULL,
                flutter_version TEXT,
                platforms TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                UNIQUE(app_id,version),
                UNIQUE(id,app_id)
            );",
        )
        .map_err(internal)
}

pub(in crate::modules::distribution) fn list(connection: &Connection) -> Result<Vec<Release>> {
    let mut statement = connection
        .prepare(
            "SELECT
                id,
                app_id,
                version,
                flutter_revision,
                flutter_version,
                platforms,
                created_at,
                updated_at
         FROM distribution_releases ORDER BY rowid",
        )
        .map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Release {
                id: row.get(0)?,
                app_id: row.get(1)?,
                version: row.get(2)?,
                flutter_revision: row.get(3)?,
                flutter_version: row.get(4)?,
                platforms: serde_json::from_str(&row.get::<_, String>(5)?).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::distribution) fn save(
    connection: &Connection,
    entity: &Release,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO distribution_releases (
                id,
                app_id,
                version,
                flutter_revision,
                flutter_version,
                platforms,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                app_id = excluded.app_id,
                version = excluded.version,
                flutter_revision = excluded.flutter_revision,
                flutter_version = excluded.flutter_version,
                platforms = excluded.platforms,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at",
            params![
                entity.id,
                entity.app_id,
                entity.version,
                entity.flutter_revision,
                entity.flutter_version,
                serde_json::to_string(&entity.platforms).map_err(internal)?,
                entity.created_at,
                entity.updated_at,
            ],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::distribution) fn find(connection: &Connection, id: i64) -> Result<Release> {
    list(connection)?
        .into_iter()
        .find(|entity| entity.id == id)
        .ok_or_else(crate::platform::error::missing)
}
