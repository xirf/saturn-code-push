use super::super::models::AssetBundle;
use crate::platform::error::{Result, internal, missing};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::artifacts) fn find(
    connection: &mut Connection<'_>,
    app: &str,
    patch: i64,
    id: i64,
) -> Result<AssetBundle> {
    list(connection)?
        .into_iter()
        .find(|b| b.id == id && b.app_id == app && b.patch_id == patch)
        .ok_or_else(missing)
}

pub(in crate::modules::artifacts) fn list(
    connection: &mut Connection<'_>,
) -> Result<Vec<AssetBundle>> {
    connection.query("SELECT id, app_id, release_id, patch_id, platform, sha256, size, upload_key, download_key, uploaded
         FROM artifacts_bundles ORDER BY id", &[]).map_err(internal)?
        .iter().map(|row| { Ok(AssetBundle {
                id: row.try_get(0).map_err(internal)?,
                app_id: row.try_get(1).map_err(internal)?,
                release_id: row.try_get(2).map_err(internal)?,
                patch_id: row.try_get(3).map_err(internal)?,
                platform: row.try_get(4).map_err(internal)?,
                sha256: row.try_get(5).map_err(internal)?,
                size: row.try_get::<_, i64>(6).map_err(internal)? as u64,
                upload_key: row.try_get(7).map_err(internal)?,
                download_key: row.try_get(8).map_err(internal)?,
                uploaded: row.try_get(9).map_err(internal)?,
            }) }).collect()
}

pub(in crate::modules::artifacts) fn save(
    connection: &mut Connection<'_>,
    entity: &AssetBundle,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO artifacts_bundles (
                id,
                app_id,
                release_id,
                patch_id,
                platform,
                sha256,
                size,
                upload_key,
                download_key,
                uploaded
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT(id) DO UPDATE SET
                app_id = excluded.app_id,
                release_id = excluded.release_id,
                patch_id = excluded.patch_id,
                platform = excluded.platform,
                sha256 = excluded.sha256,
                size = excluded.size,
                upload_key = excluded.upload_key,
                download_key = excluded.download_key,
                uploaded = excluded.uploaded",
            params![
                entity.id,
                entity.app_id,
                entity.release_id,
                entity.patch_id,
                entity.platform,
                entity.sha256,
                i64::try_from(entity.size).map_err(internal)?,
                entity.upload_key,
                entity.download_key,
                entity.uploaded,
            ],
        )
        .map_err(internal)?;
    Ok(())
}
