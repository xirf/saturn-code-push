use super::dto::Check;
use postgres::Transaction as Connection;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::{
    api,
    models::{Patch, Release},
    repositories::{patch_repository, release_repository},
};
use crate::{
    bootstrap::Runtime,
    modules::{applications, artifacts},
    platform::{
        database::next_id,
        error::{Result, bad, conflict, missing},
        values::{now, text},
    },
};

pub(super) fn list_releases(connection: &mut Connection<'_>, app_id: String) -> Result<Value> {
    applications::api::require(connection, &app_id)?;
    let releases: Vec<Release> = release_repository::list(connection)?
        .into_iter()
        .rev()
        .filter(|release| release.app_id == app_id)
        .collect();
    Ok(json!({"releases": releases}))
}

pub(super) fn create_release(
    connection: &mut Connection<'_>,
    app_id: String,
    input: Value,
) -> Result<Value> {
    let version = text(&input, "version")?;
    let flutter_revision = text(&input, "flutter_revision")?;
    applications::api::require(connection, &app_id)?;
    let existing = release_repository::list(connection)?
        .into_iter()
        .find(|release| release.app_id == app_id && release.version == version);
    if let Some(release) = existing {
        if release
            .platforms
            .values()
            .any(|platform| platform["status"] == "active")
        {
            return Err(conflict(
                "An active release with this version already exists",
            ));
        }
        return Ok(json!({"release": release}));
    }
    let release = Release {
        id: next_id(connection)?,
        app_id,
        version,
        flutter_revision,
        flutter_version: input["flutter_version"].as_str().map(str::to_owned),
        platforms: HashMap::new(),
        created_at: now(),
        updated_at: now(),
    };
    release_repository::save(connection, &release)?;
    Ok(json!({"release": release}))
}

pub(super) fn update_release(
    connection: &mut Connection<'_>,
    app_id: String,
    release_id: i64,
    input: Value,
) -> Result<Value> {
    let platform = text(&input, "platform")?;
    let status = text(&input, "status")?;
    if !["draft", "active"].contains(&status.as_str()) {
        return Err(bad("Invalid release status"));
    }
    let mut release = api::release(connection, &app_id, release_id)?;
    let binaries = artifacts::api::artifacts(connection)?;
    let has_baseline = binaries.iter().any(|artifact| {
        artifact.release_id == release_id
            && artifact.patch_id.is_none()
            && artifact.platform == platform
            && artifact.uploaded
    });
    if status == "active" && !has_baseline {
        return Err(bad("No uploaded release artifacts"));
    }
    let signing_mode = binaries
        .iter()
        .find(|artifact| {
            artifact.release_id == release_id
                && artifact.platform == platform
                && artifact.patch_id.is_none()
        })
        .and_then(|artifact| artifact.signing.as_ref())
        .and_then(|signing| signing["mode"].as_str())
        .unwrap_or("unsigned");
    release.platforms.insert(
        platform,
        json!({
            "status": status,
            "signing_mode": signing_mode,
        }),
    );
    release.updated_at = now();
    release_repository::save(connection, &release)?;
    Ok(json!({}))
}

pub(super) fn create_patch(
    connection: &mut Connection<'_>,
    app_id: String,
    input: Value,
) -> Result<Value> {
    let required_assets: Vec<String> = match input.get("required_asset_platforms") {
        None => Vec::new(),
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|_| bad("required_asset_platforms must be a string array"))?,
    };
    let mut unique = std::collections::HashSet::new();
    for platform in &required_assets {
        if !["android", "ios", "linux", "macos", "windows"].contains(&platform.as_str())
            || !unique.insert(platform)
        {
            return Err(bad("Invalid or duplicate required asset platform"));
        }
    }
    let release_id = input["release_id"]
        .as_i64()
        .ok_or_else(|| bad("Invalid release_id"))?;
    api::release(connection, &app_id, release_id)?;
    let previous_number = patch_repository::list(connection)?
        .iter()
        .filter(|patch| patch.release_id == release_id)
        .map(|patch| patch.number)
        .max()
        .unwrap_or(0);
    let patch = Patch {
        id: next_id(connection)?,
        release_id,
        number: previous_number
            .checked_add(1)
            .ok_or_else(|| conflict("Patch number limit reached"))?,
        channel_ids: Vec::new(),
        rollout_percentages: HashMap::new(),
        is_rolled_back: false,
        created_at: now(),
    };
    patch_repository::save(connection, &patch)?;
    patch_repository::require_assets(connection, patch.id, &required_assets)?;
    Ok(json!(patch))
}

pub(super) fn list_patches(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    app_id: String,
    release_id: i64,
) -> Result<Value> {
    api::release(connection, &app_id, release_id)?;
    let binaries = artifacts::api::artifacts(connection)?;
    let channels = applications::api::channels(connection)?;
    let mut responses = Vec::new();
    for patch in patch_repository::list(connection)? {
        if patch.release_id != release_id {
            continue;
        }
        let artifacts: Vec<Value> = binaries
            .iter()
            .filter(|artifact| artifact.patch_id == Some(patch.id) && artifact.uploaded)
            .map(|artifact| artifacts::api::to_wire(&runtime.base_url, artifact, false))
            .collect();
        let channel = patch
            .channel_ids
            .first()
            .and_then(|id| channels.iter().find(|channel| channel.id == *id))
            .map(|channel| &channel.name);
        let mut response = json!(patch);
        response["required_asset_platforms"] =
            json!(patch_repository::required_assets(connection, patch.id)?);
        response["assets_ready"] = json!(assets_ready(connection, patch.id)?);
        response["artifacts"] = json!(artifacts);
        response["channel"] = json!(channel);
        responses.push(response);
    }
    Ok(json!({"patches": responses}))
}

pub(super) fn promote(
    connection: &mut Connection<'_>,
    app_id: String,
    input: Value,
) -> Result<Value> {
    let patch_id = input["patch_id"]
        .as_i64()
        .ok_or_else(|| bad("Invalid patch_id"))?;
    let channel_id = input["channel_id"]
        .as_i64()
        .ok_or_else(|| bad("Invalid channel_id"))?;
    let mut patch = api::patch(connection, &app_id, patch_id)?;
    if patch.is_rolled_back {
        return Err(bad("Cannot promote a rolled-back patch"));
    }
    let channel_exists = applications::api::channels(connection)?
        .iter()
        .any(|channel| channel.id == channel_id && channel.app_id == app_id);
    if !channel_exists {
        return Err(missing());
    }
    let binaries = artifacts::api::artifacts(connection)?;
    let patch_binaries: Vec<_> = binaries
        .iter()
        .filter(|artifact| artifact.patch_id == Some(patch_id))
        .collect();
    if patch_binaries.is_empty() || patch_binaries.iter().any(|artifact| !artifact.uploaded) {
        return Err(bad("All registered patch artifacts must be uploaded"));
    }
    if !assets_ready(connection, patch.id)? {
        return Err(bad(
            "All required and registered asset bundles must be uploaded before promotion",
        ));
    }
    let release = api::release(connection, &app_id, patch.release_id)?;
    for artifact in patch_binaries {
        let has_baseline = binaries.iter().any(|baseline| {
            baseline.release_id == artifact.release_id
                && baseline.patch_id.is_none()
                && baseline.uploaded
                && baseline.arch == artifact.arch
                && baseline.platform == artifact.platform
        });
        if !has_baseline {
            return Err(bad("No matching release baseline for patch"));
        }
        if !release.is_active(&artifact.platform) {
            return Err(bad("Release platform is not active"));
        }
    }
    if let Some(value) = input.get("rollout_percentage") {
        let percentage = value
            .as_u64()
            .filter(|p| *p <= 100)
            .ok_or_else(|| bad("rollout_percentage must be an integer from 0 to 100"))?;
        patch
            .rollout_percentages
            .insert(channel_id.to_string(), percentage as u8);
    }
    if !patch.channel_ids.contains(&channel_id) {
        patch.channel_ids.push(channel_id);
    }
    patch_repository::save(connection, &patch)?;
    Ok(json!({}))
}

pub(super) fn set_rollback(
    connection: &mut Connection<'_>,
    app_id: String,
    release_id: i64,
    patch_id: i64,
    rolled_back: bool,
) -> Result<Value> {
    let mut patch = api::patch(connection, &app_id, patch_id)?;
    if patch.release_id != release_id {
        return Err(missing());
    }
    patch.is_rolled_back = rolled_back;
    patch_repository::save(connection, &patch)?;
    Ok(json!({}))
}

pub(super) fn rollback(
    connection: &mut Connection<'_>,
    app_id: String,
    release_id: i64,
    patch_id: i64,
) -> Result<Value> {
    set_rollback(connection, app_id, release_id, patch_id, true)
}

pub(super) fn rollforward(
    connection: &mut Connection<'_>,
    app_id: String,
    release_id: i64,
    patch_id: i64,
) -> Result<Value> {
    set_rollback(connection, app_id, release_id, patch_id, false)
}

pub(super) fn select_patch(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    input: &Check,
) -> Result<Value> {
    let mut response = json!({
        "patch_available": false,
        "rolled_back_patch_numbers": [],
    });
    let releases = release_repository::list(connection)?;
    let Some(release) = releases
        .iter()
        .find(|release| release.app_id == input.app_id && release.version == input.release_version)
    else {
        return Ok(response);
    };
    let patches = patch_repository::list(connection)?;
    let rolled_back: Vec<i64> = patches
        .iter()
        .filter(|patch| patch.release_id == release.id && patch.is_rolled_back)
        .map(|patch| patch.number)
        .collect();
    response["rolled_back_patch_numbers"] = json!(rolled_back);
    if !release.is_active(&input.platform) {
        return Ok(response);
    }
    let channels = applications::api::channels(connection)?;
    let Some(channel) = channels
        .iter()
        .find(|channel| channel.app_id == input.app_id && channel.name == input.channel)
    else {
        return Ok(response);
    };
    let binaries = artifacts::api::artifacts(connection)?;
    let current = input
        .current_patch_number
        .or(input.patch_number)
        .unwrap_or(0);
    let mut ready = std::collections::HashSet::new();
    for patch in &patches {
        if assets_ready(connection, patch.id)? {
            ready.insert(patch.id);
        }
    }
    let selected = patches
        .iter()
        .filter(|patch| {
            patch.release_id == release.id
                && ready.contains(&patch.id)
                && !patch.is_rolled_back
                && patch.channel_ids.contains(&channel.id)
                && rollout_eligible(patch, channel.id, input.client_id.as_deref())
        })
        .filter_map(|patch| {
            binaries
                .iter()
                .find(|artifact| {
                    artifact.patch_id == Some(patch.id)
                        && artifact.platform == input.platform
                        && artifact.arch == input.arch
                        && artifact.uploaded
                })
                .map(|artifact| (patch, artifact))
        })
        .max_by_key(|(patch, _)| patch.number);
    if let Some((patch, artifact)) = selected
        && (patch.number > current || (rolled_back.contains(&current) && patch.number != current))
    {
        response["patch_available"] = json!(true);
        response["patch"] = json!({
            "number": patch.number,
            "hash": artifact.hash,
            "hash_signature": artifact.hash_signature,
            "download_url": format!("{}/objects/{}", runtime.base_url, artifact.download_key),
        });
    }
    Ok(response)
}

pub(super) fn check_patch(
    runtime: &Runtime,
    connection: &mut Connection<'_>,
    input: Check,
) -> Result<Value> {
    select_patch(runtime, connection, &input)
}

fn assets_ready(connection: &mut Connection<'_>, patch_id: i64) -> Result<bool> {
    let required = patch_repository::required_assets(connection, patch_id)?;
    let bundles = artifacts::api::asset_bundles(connection)?;
    for platform in &required {
        let Some(bundle) = bundles
            .iter()
            .find(|b| b.patch_id == patch_id && b.platform == *platform && b.uploaded)
        else {
            return Ok(false);
        };
        if !artifacts::api::has_asset_signature(connection, bundle.id)? {
            return Ok(false);
        }
    }
    Ok(required.iter().all(|platform| {
        bundles
            .iter()
            .any(|b| b.patch_id == patch_id && b.platform == *platform && b.uploaded)
    }) && bundles
        .iter()
        .filter(|b| b.patch_id == patch_id)
        .all(|b| b.uploaded))
}

/// Hash a length-delimited identity so cohorts remain stable as the percentage grows.
fn rollout_eligible(patch: &Patch, channel: i64, client: Option<&str>) -> bool {
    use sha2::{Digest, Sha256};
    let percentage = patch
        .rollout_percentages
        .get(&channel.to_string())
        .copied()
        .unwrap_or(100);
    if percentage == 100 {
        return true;
    }
    if percentage == 0 {
        return false;
    }
    let Some(client) = client.filter(|id| !id.trim().is_empty() && id.len() <= 256) else {
        return false;
    };
    let mut hash = Sha256::new();
    hash.update(b"saturn-rollout-v1");
    hash.update(patch.id.to_be_bytes());
    hash.update(channel.to_be_bytes());
    hash.update((client.len() as u64).to_be_bytes());
    hash.update(client.as_bytes());
    let bytes = hash.finalize();
    let bucket = u64::from_be_bytes(bytes[..8].try_into().expect("eight hash bytes")) % 10_000;
    bucket < u64::from(percentage) * 100
}

#[cfg(test)]
mod rollout_tests {
    use super::*;

    #[test]
    fn cohorts_are_nested_and_isolated_by_channel() {
        let mut patch = Patch {
            id: 42,
            release_id: 1,
            number: 1,
            channel_ids: vec![1, 2],
            rollout_percentages: HashMap::new(),
            is_rolled_back: false,
            created_at: String::new(),
        };
        assert!(rollout_eligible(&patch, 1, None));
        patch.rollout_percentages.insert("1".into(), 25);
        assert!(!rollout_eligible(&patch, 1, None));
        assert!(!rollout_eligible(&patch, 1, Some(" ")));
        assert!(!rollout_eligible(&patch, 1, Some(&"a".repeat(257))));
        assert!(rollout_eligible(&patch, 2, None));
        let cohort: Vec<_> = (0..1000)
            .map(|n| rollout_eligible(&patch, 1, Some(&format!("device-{n}"))))
            .collect();
        let count = cohort.iter().filter(|v| **v).count();
        assert!((180..320).contains(&count));
        patch.rollout_percentages.insert("1".into(), 75);
        for (n, included) in cohort.iter().enumerate() {
            if *included {
                assert!(rollout_eligible(&patch, 1, Some(&format!("device-{n}"))));
            }
        }
        patch.rollout_percentages.insert("1".into(), 0);
        assert!(!rollout_eligible(&patch, 1, Some("device-1")));
        patch.rollout_percentages.insert("1".into(), 100);
        assert!(rollout_eligible(&patch, 1, None));
    }
}
