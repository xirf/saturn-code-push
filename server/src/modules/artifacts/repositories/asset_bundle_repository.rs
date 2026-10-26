use super::super::models::AssetBundle;
use crate::platform::error::{Result, internal, missing};
use rusqlite::{Connection, OptionalExtension, params};

pub(in crate::modules::artifacts) fn find(
    connection: &Connection,
    app: &str,
    patch: i64,
    id: i64,
) -> Result<AssetBundle> {
    connection
        .query_row(
            "SELECT id, app_id, release_id, patch_id, platform, sha256, size,
                    upload_key, download_key, uploaded
             FROM artifacts_bundles WHERE id = ?1 AND app_id = ?2 AND patch_id = ?3",
            params![id, app, patch],
            |row| {
                Ok(AssetBundle {
                    id: row.get(0)?,
                    app_id: row.get(1)?,
                    release_id: row.get(2)?,
                    patch_id: row.get(3)?,
                    platform: row.get(4)?,
                    sha256: row.get(5)?,
                    size: row.get::<_, u32>(6)? as u64,
                    upload_key: row.get(7)?,
                    download_key: row.get(8)?,
                    uploaded: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(internal)?
        .ok_or_else(missing)
}

pub(in crate::modules::artifacts) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS artifacts_bundles (
                id INTEGER PRIMARY KEY,
                app_id TEXT NOT NULL,
                release_id INTEGER NOT NULL,
                patch_id INTEGER NOT NULL,
                platform TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                size INTEGER NOT NULL CHECK (size > 0),
                upload_key TEXT NOT NULL,
                download_key TEXT NOT NULL,
                uploaded INTEGER NOT NULL CHECK (uploaded IN (0, 1)),
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                FOREIGN KEY(release_id,app_id) REFERENCES distribution_releases(id,app_id),
                FOREIGN KEY(patch_id,release_id) REFERENCES distribution_patches(id,release_id),
                UNIQUE(patch_id,platform),
                UNIQUE(upload_key),
                UNIQUE(download_key)
            );",
        )
        .map_err(internal)
}

pub(in crate::modules::artifacts) fn list(connection: &Connection) -> Result<Vec<AssetBundle>> {
    let mut statement = connection.prepare(
        "SELECT id, app_id, release_id, patch_id, platform, sha256, size, upload_key, download_key, uploaded
         FROM artifacts_bundles ORDER BY rowid",
    ).map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            Ok(AssetBundle {
                id: row.get(0)?,
                app_id: row.get(1)?,
                release_id: row.get(2)?,
                patch_id: row.get(3)?,
                platform: row.get(4)?,
                sha256: row.get(5)?,
                size: row.get::<_, u32>(6)? as u64,
                upload_key: row.get(7)?,
                download_key: row.get(8)?,
                uploaded: row.get(9)?,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::artifacts) fn save(
    connection: &Connection,
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
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
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
