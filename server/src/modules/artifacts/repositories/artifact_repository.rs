use super::super::models::Artifact;
use crate::platform::error::{Result, internal};
use rusqlite::{Connection, params};

pub(in crate::modules::artifacts) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS artifacts_binaries (
                id INTEGER PRIMARY KEY,
                app_id TEXT NOT NULL,
                release_id INTEGER NOT NULL,
                patch_id INTEGER,
                arch TEXT NOT NULL,
                platform TEXT NOT NULL,
                hash TEXT NOT NULL,
                hash_signature TEXT,
                size INTEGER NOT NULL CHECK (size > 0),
                can_sideload INTEGER NOT NULL CHECK (can_sideload IN (0, 1)),
                signing TEXT,
                upload_key TEXT NOT NULL,
                download_key TEXT NOT NULL,
                uploaded INTEGER NOT NULL CHECK (uploaded IN (0, 1)),
                FOREIGN KEY(app_id) REFERENCES applications_apps(id),
                FOREIGN KEY(release_id,app_id) REFERENCES distribution_releases(id,app_id),
                FOREIGN KEY(patch_id,release_id) REFERENCES distribution_patches(id,release_id),
                UNIQUE(upload_key),
                UNIQUE(download_key)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS artifacts_release_target
                ON artifacts_binaries(release_id,arch,platform) WHERE patch_id IS NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS artifacts_patch_target
                ON artifacts_binaries(patch_id,arch,platform) WHERE patch_id IS NOT NULL;",
        )
        .map_err(internal)
}

pub(in crate::modules::artifacts) fn list(connection: &Connection) -> Result<Vec<Artifact>> {
    let mut statement = connection
        .prepare(
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
         FROM artifacts_binaries ORDER BY rowid",
        )
        .map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Artifact {
                id: row.get(0)?,
                app_id: row.get(1)?,
                release_id: row.get(2)?,
                patch_id: row.get(3)?,
                arch: row.get(4)?,
                platform: row.get(5)?,
                hash: row.get(6)?,
                hash_signature: row.get(7)?,
                size: row.get::<_, u32>(8)? as u64,
                can_sideload: row.get(9)?,
                signing: serde_json::from_str(&row.get::<_, String>(10)?).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        10,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?,
                upload_key: row.get(11)?,
                download_key: row.get(12)?,
                uploaded: row.get(13)?,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::artifacts) fn save(
    connection: &Connection,
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
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
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
