pub(crate) use super::models::{Artifact, AssetBundle};
use super::repositories::{artifact_repository, asset_bundle_repository};
use crate::platform::error::Result;
use postgres::Transaction as Connection;

pub(crate) fn artifacts(connection: &mut Connection<'_>) -> Result<Vec<Artifact>> {
    artifact_repository::list(connection)
}

pub(crate) fn asset_bundles(connection: &mut Connection<'_>) -> Result<Vec<AssetBundle>> {
    asset_bundle_repository::list(connection)
}

/// Used by the schema migration and test fixtures, never by request handlers.
pub(crate) fn import(
    connection: &mut Connection<'_>,
    artifacts: Vec<Artifact>,
    asset_bundles: Vec<AssetBundle>,
) -> Result<()> {
    for entity in &artifacts {
        artifact_repository::save(connection, entity)?;
    }
    for entity in &asset_bundles {
        asset_bundle_repository::save(connection, entity)?;
    }
    Ok(())
}

use crate::platform::values::now;
use serde_json::{Value, json};
pub(crate) fn to_wire(base_url: &str, artifact: &Artifact, upload: bool) -> Value {
    let (path, key) = if upload {
        ("uploads", &artifact.upload_key)
    } else {
        ("objects", &artifact.download_key)
    };
    json!({
        "id": artifact.id,
        "release_id": artifact.release_id,
        "patch_id": artifact.patch_id,
        "arch": artifact.arch,
        "platform": artifact.platform,
        "hash": artifact.hash,
        "size": artifact.size,
        "can_sideload": artifact.can_sideload,
        "signing": artifact.signing,
        "created_at": now(),
        "url": format!("{base_url}/{path}/{key}"),
        "upload_method": "multipart",
    })
}

pub(crate) fn has_asset_signature(connection: &mut Connection<'_>, bundle_id: i64) -> Result<bool> {
    connection
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM artifact_asset_signatures WHERE bundle_id=$1)",
            crate::sql_params![bundle_id],
        )
        .map_err(crate::platform::error::internal)?
        .try_get(0)
        .map_err(crate::platform::error::internal)
}
