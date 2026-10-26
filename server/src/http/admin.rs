use crate::{
    bootstrap::Runtime,
    modules::{applications, artifacts, distribution, telemetry},
    platform::error::Result,
};
use axum::{Json, extract::State};
use serde_json::{Value, json};

pub(super) fn routes(runtime: &Runtime) -> axum::Router<Runtime> {
    super::policy::management(
        runtime,
        axum::Router::new().route("/admin/state", axum::routing::get(snapshot)),
    )
}

pub(super) async fn snapshot(State(runtime): State<Runtime>) -> Result<Json<Value>> {
    runtime
        .database
        .call(|connection| {
            let apps = applications::api::apps(connection)?;
            let channels = applications::api::channels(connection)?;
            let releases = distribution::api::releases(connection)?;
            let patches = distribution::api::patches(connection)?;
            let artifacts = artifacts::api::artifacts(connection)?;
            let bundles = artifacts::api::asset_bundles(connection)?;
            let events = telemetry::api::events(connection)?;
            // Capability tokens and signatures are intentionally omitted from the dashboard snapshot.
            let artifacts: Vec<Value> = artifacts
                .iter()
                .map(|artifact| {
                    json!({
                        "id": artifact.id,
                        "app_id": artifact.app_id,
                        "release_id": artifact.release_id,
                        "patch_id": artifact.patch_id,
                        "arch": artifact.arch,
                        "platform": artifact.platform,
                        "uploaded": artifact.uploaded,
                        "size": artifact.size,
                    })
                })
                .collect();
            let bundles: Vec<Value> = bundles
                .iter()
                .map(|bundle| {
                    json!({
                        "id": bundle.id,
                        "app_id": bundle.app_id,
                        "patch_id": bundle.patch_id,
                        "platform": bundle.platform,
                        "sha256": bundle.sha256,
                        "uploaded": bundle.uploaded,
                        "size": bundle.size,
                    })
                })
                .collect();
            Ok(json!({
                "apps": apps,
                "channels": channels,
                "releases": releases,
                "patches": patches,
                "artifacts": artifacts,
                "asset_bundles": bundles,
                "events": events,
            }))
        })
        .await
        .map(Json)
}
