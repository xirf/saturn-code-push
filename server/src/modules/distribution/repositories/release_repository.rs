use super::super::models::Release;
use crate::platform::error::{Result, internal};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::distribution) fn list(
    connection: &mut Connection<'_>,
) -> Result<Vec<Release>> {
    connection
        .query(
            "SELECT
                id,
                app_id,
                version,
                flutter_revision,
                flutter_version,
                platforms,
                created_at,
                updated_at
         FROM distribution_releases ORDER BY id",
            &[],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(Release {
                id: row.try_get(0).map_err(internal)?,
                app_id: row.try_get(1).map_err(internal)?,
                version: row.try_get(2).map_err(internal)?,
                flutter_revision: row.try_get(3).map_err(internal)?,
                flutter_version: row.try_get(4).map_err(internal)?,
                platforms: serde_json::from_str(&row.try_get::<_, String>(5).map_err(internal)?)
                    .map_err(internal)?,
                created_at: row.try_get(6).map_err(internal)?,
                updated_at: row.try_get(7).map_err(internal)?,
            })
        })
        .collect()
}

pub(in crate::modules::distribution) fn save(
    connection: &mut Connection<'_>,
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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

pub(in crate::modules::distribution) fn find(
    connection: &mut Connection<'_>,
    id: i64,
) -> Result<Release> {
    list(connection)?
        .into_iter()
        .find(|entity| entity.id == id)
        .ok_or_else(crate::platform::error::missing)
}
