use super::super::dto::{AssetCheck, RegisterBundle};
use super::super::{service, upload::Upload};
use crate::platform::objects::Kind;
use crate::{
    bootstrap::Runtime,
    platform::error::{Result, bad, internal},
};
use axum::response::{IntoResponse, Redirect};
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
    let location = runtime.objects.destination(
        Kind::Bundle,
        &bundle.app_id,
        bundle.release_id,
        Some(bundle.patch_id),
        bundle.id,
    );
    super::super::upload::publish(
        runtime.database,
        temporary,
        runtime.objects,
        location,
        Kind::Bundle,
        bundle.id,
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
    let location = runtime
        .objects
        .location(&runtime.database, Kind::Bundle, id)
        .await?;
    if let Some(s3) = &location.remote {
        let mut response =
            Redirect::temporary(&s3.signed_url(&location.key).await?).into_response();
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
        return Ok(response);
    }
    let response = ServeFile::new(runtime.objects.local_path(&location))
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
