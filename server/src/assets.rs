use axum::{
    Json,
    body::{Body, Bytes},
    extract::{Path, Query, Request, State},
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::{Catalog, Result, Service, bad, conflict, internal, missing, uid};

const MAX_BUNDLE_SIZE: u64 = 32 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AssetBundleRow {
    pub(crate) id: i64,
    pub(crate) app_id: String,
    pub(crate) release_id: i64,
    pub(crate) patch_id: i64,
    pub(crate) platform: String,
    pub(crate) sha256: String,
    pub(crate) size: u64,
    pub(crate) upload_key: String,
    pub(crate) download_key: String,
    pub(crate) uploaded: bool,
}

#[derive(Deserialize)]
pub(crate) struct RegisterBundle {
    sha256: String,
    size: u64,
    platform: String,
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit())
}

pub(crate) async fn register(
    State(s): State<Service>,
    Path((app, patch)): Path<(String, i64)>,
    Json(request): Json<RegisterBundle>,
) -> Result<Json<Value>> {
    if !valid_sha256(&request.sha256) || !(1..=MAX_BUNDLE_SIZE).contains(&request.size) {
        return Err(bad("Invalid asset bundle hash or size"));
    }
    if !["android", "ios", "linux", "macos", "windows"].contains(&request.platform.as_str()) {
        return Err(bad("Unsupported asset bundle platform"));
    }
    s.write(|d: &mut Catalog| {
        let release_id = d.patch(&app, patch)?.release_id;
        let release = d.release(&app, release_id)?;
        if !release.platforms.contains_key(&request.platform) {
            return Err(bad("Release does not include this platform"));
        }
        if d.asset_bundles
            .iter()
            .any(|b| b.patch_id == patch && b.platform == request.platform)
        {
            return Err(conflict(
                "Asset bundle already registered for this patch and platform",
            ));
        }
        let row = AssetBundleRow {
            id: d.next(),
            app_id: app,
            release_id,
            patch_id: patch,
            platform: request.platform,
            sha256: request.sha256.to_ascii_lowercase(),
            size: request.size,
            upload_key: uid(),
            download_key: uid(),
            uploaded: false,
        };
        let upload_url = format!("{}/asset-uploads/{}", s.base_url, row.upload_key);
        d.asset_bundles.push(row.clone());
        Ok(Json(json!({"id":row.id,"upload_url":upload_url})))
    })
}

pub(crate) async fn upload(
    State(s): State<Service>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<Json<Value>> {
    let row = s.read(|d| {
        d.asset_bundles
            .iter()
            .find(|b| b.upload_key == key && !b.uploaded)
            .cloned()
            .ok_or_else(missing)
    })?;
    if body.len() as u64 != row.size || format!("{:x}", Sha256::digest(&body)) != row.sha256 {
        return Err(bad("Asset bundle size or SHA-256 mismatch"));
    }
    let tmp = s.root.join("objects").join(format!("asset-{}.tmp", uid()));
    tokio::fs::write(&tmp, &body).await.map_err(internal)?;
    let outcome = s.write(|d| {
        let bundle = d
            .asset_bundles
            .iter_mut()
            .find(|b| b.id == row.id && !b.uploaded)
            .ok_or_else(|| conflict("Asset bundle already uploaded"))?;
        std::fs::rename(
            &tmp,
            s.root.join("objects").join(format!("asset-{}", row.id)),
        )
        .map_err(internal)?;
        bundle.uploaded = true;
        Ok(Json(json!({"uploaded":true})))
    });
    if outcome.is_err() {
        let _ = tokio::fs::remove_file(tmp).await;
    }
    outcome
}

pub(crate) async fn download(
    State(s): State<Service>,
    Path(key): Path<String>,
    request: Request,
) -> Result<Response> {
    let id = s.read(|d| {
        d.asset_bundles
            .iter()
            .find(|b| b.download_key == key && b.uploaded)
            .map(|b| b.id)
            .ok_or_else(missing)
    })?;
    let response = ServeFile::new(s.root.join("objects").join(format!("asset-{id}")))
        .oneshot(request)
        .await
        .map_err(internal)?;
    Ok(response.map(Body::new))
}

#[derive(Deserialize)]
pub(crate) struct AssetCheck {
    app_id: String,
    release_version: String,
    channel: String,
    platform: String,
    patch_number: i64,
}

pub(crate) async fn check(
    State(s): State<Service>,
    Query(q): Query<AssetCheck>,
) -> Result<Json<Value>> {
    if q.patch_number < 1 || q.patch_number > 1000000 {
        return Ok(Json(json!({"asset_bundle_available":false})));
    }
    s.read(|d| {
        let Some(release) = d
            .releases
            .iter()
            .find(|r| r.app_id == q.app_id && r.version == q.release_version)
        else {
            return Ok(Json(json!({"asset_bundle_available":false})));
        };
        if release
            .platforms
            .get(&q.platform)
            .and_then(|v| v["status"].as_str())
            != Some("active")
        {
            return Ok(Json(json!({"asset_bundle_available":false})));
        }
        let Some(channel) = d
            .channels
            .iter()
            .find(|c| c.app_id == q.app_id && c.name == q.channel)
        else {
            return Ok(Json(json!({"asset_bundle_available":false})));
        };
        let Some(patch) = d.patches.iter().find(|p| {
            p.release_id == release.id
                && p.number == q.patch_number
                && !p.is_rolled_back
                && p.channel_ids.contains(&channel.id)
        }) else {
            return Ok(Json(json!({"asset_bundle_available":false})));
        };
        let Some(bundle) = d
            .asset_bundles
            .iter()
            .find(|b| b.patch_id == patch.id && b.platform == q.platform && b.uploaded)
        else {
            return Ok(Json(json!({"asset_bundle_available":false})));
        };
        Ok(Json(
            json!({"asset_bundle_available":true,"patch_number":patch.number,
            "sha256":bundle.sha256,"size":bundle.size,
            "download_url":format!("{}/asset-objects/{}",s.base_url,bundle.download_key)}),
        ))
    })
}
