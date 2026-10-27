use super::super::models::Artifact;
use crate::platform::error::{Result, internal};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::artifacts) fn list(
    connection: &mut Connection<'_>,
) -> Result<Vec<Artifact>> {
    connection
        .query(
            "SELECT
                id,
                app_id,
                release_id,
                patch_id,
                arch,
                platform,
                hash,
                hash_signature,
                size,
                can_sideload,
                signing,
                upload_key,
                download_key,
                uploaded
         FROM artifacts_binaries ORDER BY id",
            &[],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(Artifact {
                id: row.try_get(0).map_err(internal)?,
                app_id: row.try_get(1).map_err(internal)?,
                release_id: row.try_get(2).map_err(internal)?,
                patch_id: row.try_get(3).map_err(internal)?,
                arch: row.try_get(4).map_err(internal)?,
                platform: row.try_get(5).map_err(internal)?,
                hash: row.try_get(6).map_err(internal)?,
                hash_signature: row.try_get(7).map_err(internal)?,
                size: row.try_get::<_, i64>(8).map_err(internal)? as u64,
                can_sideload: row.try_get(9).map_err(internal)?,
                signing: serde_json::from_str(&row.try_get::<_, String>(10).map_err(internal)?)
                    .map_err(internal)?,
                upload_key: row.try_get(11).map_err(internal)?,
                download_key: row.try_get(12).map_err(internal)?,
                uploaded: row.try_get(13).map_err(internal)?,
            })
        })
        .collect()
}

pub(in crate::modules::artifacts) fn save(
    connection: &mut Connection<'_>,
    entity: &Artifact,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO artifacts_binaries (
                id,
                app_id,
                release_id,
                patch_id,
                arch,
                platform,
                hash,
                hash_signature,
                size,
                can_sideload,
                signing,
                upload_key,
                download_key,
                uploaded
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT(id) DO UPDATE SET
                app_id = excluded.app_id,
                release_id = excluded.release_id,
                patch_id = excluded.patch_id,
                arch = excluded.arch,
                platform = excluded.platform,
                hash = excluded.hash,
                hash_signature = excluded.hash_signature,
                size = excluded.size,
                can_sideload = excluded.can_sideload,
                signing = excluded.signing,
                upload_key = excluded.upload_key,
                download_key = excluded.download_key,
                uploaded = excluded.uploaded",
            params![
                entity.id,
                entity.app_id,
                entity.release_id,
                entity.patch_id,
                entity.arch,
                entity.platform,
                entity.hash,
                entity.hash_signature,
                i64::try_from(entity.size).map_err(internal)?,
                entity.can_sideload,
                serde_json::to_string(&entity.signing).map_err(internal)?,
                entity.upload_key,
                entity.download_key,
                entity.uploaded,
            ],
        )
        .map_err(internal)?;
    Ok(())
}
