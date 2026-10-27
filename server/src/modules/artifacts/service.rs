use super::{
    models::{Artifact, AssetBundle},
    repositories::{artifact_repository, asset_bundle_repository},
};
use crate::modules::distribution;
use crate::{
    bootstrap::Runtime,
    platform::{
        database::next_id,
        error::{Result, bad, conflict, missing},
        values::{text, uid},
    },
};
use postgres::Transaction as Connection;
use serde_json::{Value, json};
use std::collections::HashMap;

pub(super) fn list_artifacts(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    app_id: String,
    release_id: i64,
    filters: HashMap<String, String>,
) -> Result<Value> {
    distribution::api::release(connection, &app_id, release_id)?;
    let artifacts = artifact_repository::list(connection)?;
    let artifacts: Vec<Value> = artifacts
        .iter()
        .filter(|artifact| {
            artifact.release_id == release_id
                && artifact.patch_id.is_none()
                && artifact.uploaded
                && filters
                    .get("arch")
                    .is_none_or(|arch| *arch == artifact.arch)
                && filters
                    .get("platform")
                    .is_none_or(|platform| *platform == artifact.platform)
        })
        .map(|artifact| super::api::to_wire(&runtime.base_url, artifact, false))
        .collect();
    Ok(json!({"artifacts": artifacts}))
}

pub(super) fn register(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    app: String,
    release: i64,
    patch: Option<i64>,
    metadata: HashMap<String, String>,
) -> Result<Value> {
    let input = json!(metadata);
    let hash = text(&input, "hash")?.to_ascii_lowercase();
    if hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(bad("Expected SHA-256 of the reconstructed artifact"));
    }
    let arch = text(&input, "arch")?;
    let platform = text(&input, "platform")?;
    let size = text(&input, "size")?
        .parse::<u64>()
        .map_err(|_| bad("Invalid size"))?;
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err(bad("Artifact size must be between 1 and 256 MiB"));
    }
    let artifacts = artifact_repository::list(connection)?;
    distribution::api::release(connection, &app, release)?;
    if let Some(p) = patch {
        let patch = distribution::api::patch(connection, &app, p)?;
        if patch.release_id != release {
            return Err(bad("Patch does not belong to release"));
        }
    }
    if let Some(a) = artifacts.iter().find(|a| {
        a.release_id == release && a.patch_id == patch && a.arch == arch && a.platform == platform
    }) {
        if a.hash != hash || a.size != size {
            return Err(bad("An immutable artifact already has different content"));
        }
        if a.uploaded {
            return Err(conflict("Artifact already registered"));
        }
        return Ok(super::api::to_wire(&runtime.base_url, a, true));
    }
    let a = Artifact {
        id: next_id(connection)?,
        app_id: app,
        release_id: release,
        patch_id: patch,
        arch,
        platform,
        hash: hash.to_lowercase(),
        hash_signature: input["hash_signature"].as_str().map(str::to_owned),
        size,
        can_sideload: input["can_sideload"] == "true",
        signing: input["signing"]
            .as_str()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|_| bad("Invalid signing metadata"))?,
        upload_key: uid(),
        download_key: uid(),
        uploaded: false,
    };
    let response = super::api::to_wire(&runtime.base_url, &a, true);
    artifact_repository::save(connection, &a)?;
    Ok(response)
}

pub(super) fn register_release_artifact(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    app: String,
    id: i64,
    metadata: HashMap<String, String>,
) -> Result<Value> {
    register(runtime, connection, app, id, None, metadata)
}

pub(super) fn register_patch_artifact(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    app: String,
    id: i64,
    metadata: HashMap<String, String>,
) -> Result<Value> {
    let release = distribution::api::patch(connection, &app, id)?.release_id;
    register(runtime, connection, app, release, Some(id), metadata)
}

pub(super) fn pending_binary(connection: &mut Connection<'_>, key: &str) -> Result<Artifact> {
    artifact_repository::list(connection)?
        .into_iter()
        .find(|entity| entity.upload_key == key && !entity.uploaded)
        .ok_or_else(missing)
}

pub(super) fn complete_binary(connection: &mut Connection<'_>, id: i64) -> Result<()> {
    let mut entity = artifact_repository::list(connection)?
        .into_iter()
        .find(|entity| entity.id == id && !entity.uploaded)
        .ok_or_else(|| conflict("Upload already completed"))?;
    entity.uploaded = true;
    artifact_repository::save(connection, &entity)
}

pub(super) fn download_binary(connection: &mut Connection<'_>, key: &str) -> Result<i64> {
    artifact_repository::list(connection)?
        .into_iter()
        .find(|entity| entity.download_key == key && entity.uploaded)
        .map(|entity| entity.id)
        .ok_or_else(missing)
}

pub(super) fn register_bundle(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    app: String,
    patch_id: i64,
    input: super::dto::RegisterBundle,
) -> Result<Value> {
    if input.sha256.len() != 64 || !input.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(bad("Invalid asset bundle hash"));
    }
    if input.size == 0 || input.size > 32 * 1024 * 1024 {
        return Err(bad("Asset bundle size must be between 1 and 32 MiB"));
    }
    if !["android", "ios", "linux", "macos", "windows"].contains(&input.platform.as_str()) {
        return Err(bad("Unsupported asset bundle platform"));
    }
    let patch = distribution::api::patch(connection, &app, patch_id)?;
    if !patch.channel_ids.is_empty() {
        return Err(conflict(
            "Cannot add asset bundles after patch promotion; create a new patch and declare required_asset_platforms",
        ));
    }
    let release = distribution::api::release(connection, &app, patch.release_id)?;
    if !release.platforms.contains_key(&input.platform) {
        return Err(bad("Release does not include this platform"));
    }
    let asset_bundles = asset_bundle_repository::list(connection)?;
    if asset_bundles
        .iter()
        .any(|bundle| bundle.patch_id == patch_id && bundle.platform == input.platform)
    {
        return Err(conflict(
            "Asset bundle already registered for this patch and platform",
        ));
    }
    if let Some(signature) = &input.signature {
        use base64::Engine;
        if base64::engine::general_purpose::STANDARD
            .decode(signature)
            .map_or(true, |bytes| bytes.len() != 64)
        {
            return Err(bad("Asset signature must be a base64 Ed25519 signature"));
        }
    }
    let signature = input.signature;
    let bundle = AssetBundle {
        id: next_id(connection)?,
        app_id: app,
        release_id: patch.release_id,
        patch_id,
        platform: input.platform,
        sha256: input.sha256.to_ascii_lowercase(),
        size: input.size,
        upload_key: uid(),
        download_key: uid(),
        uploaded: false,
    };
    let response = json!({
        "id": bundle.id,
        "upload_url": format!("{}/asset-uploads/{}", runtime.base_url, bundle.upload_key),
    });
    asset_bundle_repository::save(connection, &bundle)?;
    if let Some(signature) = signature {
        connection
            .execute(
                "INSERT INTO artifact_asset_signatures(bundle_id, signature) VALUES ($1, $2)",
                crate::sql_params![bundle.id, signature],
            )
            .map_err(crate::platform::error::internal)?;
    }
    Ok(response)
}

pub(super) fn pending_bundle(connection: &mut Connection<'_>, key: &str) -> Result<AssetBundle> {
    asset_bundle_repository::list(connection)?
        .into_iter()
        .find(|entity| entity.upload_key == key && !entity.uploaded)
        .ok_or_else(missing)
}

pub(super) fn complete_bundle(connection: &mut Connection<'_>, id: i64) -> Result<()> {
    let mut entity = asset_bundle_repository::list(connection)?
        .into_iter()
        .find(|entity| entity.id == id && !entity.uploaded)
        .ok_or_else(|| conflict("Asset bundle already completed"))?;
    entity.uploaded = true;
    asset_bundle_repository::save(connection, &entity)
}

pub(super) fn download_bundle(connection: &mut Connection<'_>, key: &str) -> Result<i64> {
    asset_bundle_repository::list(connection)?
        .into_iter()
        .find(|entity| entity.download_key == key && entity.uploaded)
        .map(|entity| entity.id)
        .ok_or_else(missing)
}

pub(super) fn check_bundle(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    input: super::dto::AssetCheck,
) -> Result<Value> {
    let unavailable = json!({"asset_bundle_available": false});
    if !(1..=1_000_000).contains(&input.patch_number) {
        return Ok(unavailable);
    }
    let releases = distribution::api::releases(connection)?;
    let Some(release) = releases
        .iter()
        .find(|release| release.app_id == input.app_id && release.version == input.release_version)
    else {
        return Ok(unavailable);
    };
    if release
        .platforms
        .get(&input.platform)
        .and_then(|platform| platform["status"].as_str())
        != Some("active")
    {
        return Ok(unavailable);
    }
    let channels = crate::modules::applications::api::channels(connection)?;
    let Some(channel) = channels
        .iter()
        .find(|channel| channel.app_id == input.app_id && channel.name == input.channel)
    else {
        return Ok(unavailable);
    };
    let patches = distribution::api::patches(connection)?;
    let Some(patch) = patches.iter().find(|patch| {
        patch.release_id == release.id
            && patch.number == input.patch_number
            && !patch.is_rolled_back
            && patch.channel_ids.contains(&channel.id)
    }) else {
        return Ok(unavailable);
    };
    let bundles = super::api::asset_bundles(connection)?;
    let Some(bundle) = bundles.iter().find(|bundle| {
        bundle.patch_id == patch.id && bundle.platform == input.platform && bundle.uploaded
    }) else {
        return Ok(unavailable);
    };

    let signature: Option<String> = connection
        .query_opt(
            "SELECT signature FROM artifact_asset_signatures WHERE bundle_id = $1",
            &[&bundle.id],
        )
        .map_err(crate::platform::error::internal)?
        .map(|row| row.get(0));
    let manifest = serde_json::to_string(&json!([
        "saturn-assets-v1",
        input.app_id,
        input.release_version,
        patch.number,
        input.platform,
        bundle.sha256,
        bundle.size
    ]))
    .map_err(crate::platform::error::internal)?;
    Ok(json!({
        "manifest": manifest,
        "signature": signature,
        "asset_bundle_available": true,
        "patch_number": patch.number,
        "sha256": bundle.sha256,
        "size": bundle.size,
        "download_url": format!("{}/asset-objects/{}", runtime.base_url, bundle.download_key),
    }))
}
