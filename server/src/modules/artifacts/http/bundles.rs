use super::super::dto::{AssetCheck, RegisterBundle};
use super::super::{service, upload::Upload};
use crate::{
    bootstrap::Runtime,
    platform::error::{Result, bad, internal},
};
use axum::{
    Json,
    body::Body,
    extract::{Path, Query, Request, State},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use tower_http::services::ServeFile;

pub(crate) async fn register(
    State(runtime): State<Runtime>,
    Path((app, patch)): Path<(String, i64)>,
    Json(input): Json<RegisterBundle>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| service::register_bundle(&runtime, connection, app, patch, input))
        .await
        .map(Json)
}

pub(crate) async fn upload(
    State(runtime): State<Runtime>,
    Path(key): Path<String>,
    request: Request,
) -> Result<Json<Value>> {
    let bundle = runtime
        .database
        .call(move |connection| service::pending_bundle(connection, &key))
        .await?;
    let mut upload = Upload::new(runtime.objects.directory(), bundle.size)?;
    let mut body = request.into_body();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|error| bad(error.to_string()))?;
        if let Ok(bytes) = frame.into_data() {
            upload.chunk(&bytes).await?;
        }
    }
    let temporary = upload.finish(Some(&bundle.sha256)).await?;
    let destination = runtime.objects.bundle(bundle.id);
    super::super::upload::publish(
        runtime.database,
        temporary,
        destination,
        move |connection| service::complete_bundle(connection, bundle.id),
    )
    .await?;
    Ok(Json(serde_json::json!({"uploaded": true})))
}

pub(crate) async fn download(
    State(runtime): State<Runtime>,
    Path(key): Path<String>,
    request: Request,
) -> Result<Response> {
    let id = runtime
        .database
        .call(move |connection| service::download_bundle(connection, &key))
        .await?;
    let response = ServeFile::new(runtime.objects.bundle(id))
        .oneshot(request)
        .await
        .map_err(internal)?;
    Ok(response.map(Body::new))
}

pub(crate) async fn check(
    State(runtime): State<Runtime>,
    Query(input): Query<AssetCheck>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| service::check_bundle(&runtime, connection, input))
        .await
        .map(Json)
}
