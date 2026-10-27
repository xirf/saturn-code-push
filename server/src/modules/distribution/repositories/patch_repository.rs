use super::super::models::Patch;
use crate::platform::error::{Result, internal};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::distribution) fn list(
    connection: &mut Connection<'_>,
) -> Result<Vec<Patch>> {
    connection
        .query(
            "SELECT
                id,
                release_id,
                number,
                channel_ids,
                is_rolled_back,
                created_at
         FROM distribution_patches ORDER BY id",
            &[],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(Patch {
                id: row.try_get(0).map_err(internal)?,
                release_id: row.try_get(1).map_err(internal)?,
                number: row.try_get(2).map_err(internal)?,
                channel_ids: serde_json::from_str(&row.try_get::<_, String>(3).map_err(internal)?)
                    .map_err(internal)?,
                is_rolled_back: row.try_get(4).map_err(internal)?,
                created_at: row.try_get(5).map_err(internal)?,
            })
        })
        .collect()
}

pub(in crate::modules::distribution) fn save(
    connection: &mut Connection<'_>,
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
            ) VALUES ($1, $2, $3, $4, $5, $6)
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

pub(in crate::modules::distribution) fn find(
    connection: &mut Connection<'_>,
    id: i64,
) -> Result<Patch> {
    list(connection)?
        .into_iter()
        .find(|entity| entity.id == id)
        .ok_or_else(crate::platform::error::missing)
}

/// Requirements are declared before any code artifact can be made available.
pub(crate) fn require_assets(
    connection: &mut Connection<'_>,
    patch_id: i64,
    platforms: &[String],
) -> Result<()> {
    for platform in platforms {
        connection
            .execute(
                "INSERT INTO distribution_patch_assets(patch_id, platform) VALUES ($1, $2)",
                params![patch_id, platform],
            )
            .map_err(internal)?;
    }
    Ok(())
}

pub(crate) fn required_assets(
    connection: &mut Connection<'_>,
    patch_id: i64,
) -> Result<Vec<String>> {
    connection
        .query(
            "SELECT platform FROM distribution_patch_assets WHERE patch_id=$1",
            &[&patch_id],
        )
        .map_err(internal)?
        .iter()
        .map(|row| row.try_get(0).map_err(internal))
        .collect()
}
