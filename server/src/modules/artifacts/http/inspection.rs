use super::super::{dto, inspection, repositories::asset_bundle_repository};
use crate::platform::objects::{Kind, LocalObject};
use crate::{
    bootstrap::Runtime,
    platform::error::{Result, conflict, internal},
};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderValue, header},
    response::Response,
    routing::get,
};
use tokio::sync::Semaphore;
use tower::ServiceExt;
use tower_http::services::ServeFile;

// Bound archive and image work independently of the HTTP worker pool.
static INSPECTION_SLOTS: Semaphore = Semaphore::const_new(2);

pub(super) fn routes() -> Router<Runtime> {
    Router::new()
        .route("/files", get(list))
        .route("/files/{file}/preview", get(preview))
        .route("/download", get(download))
}

async fn bundle_path(
    runtime: &Runtime,
    app: String,
    patch: i64,
    bundle: i64,
) -> Result<LocalObject> {
    let entity = runtime
        .database
        .call(move |connection| asset_bundle_repository::find(connection, &app, patch, bundle))
        .await?;
    if !entity.uploaded {
        return Err(conflict("The asset bundle is awaiting upload"));
    }
    runtime
        .objects
        .materialize(&runtime.database, Kind::Bundle, entity.id)
        .await
}

async fn list(
    State(runtime): State<Runtime>,
    Path((app, patch, bundle)): Path<(String, i64, i64)>,
) -> Result<Json<dto::AssetInspection>> {
    let permit = INSPECTION_SLOTS.acquire().await.map_err(internal)?;
    let path = bundle_path(&runtime, app, patch, bundle).await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        inspection::list(&path.path)
    })
    .await
    .map_err(internal)?
    .map(Json)
}

async fn preview(
    State(runtime): State<Runtime>,
    Path((app, patch, bundle, file)): Path<(String, i64, i64, usize)>,
) -> Result<Json<dto::AssetPreview>> {
    let permit = INSPECTION_SLOTS.acquire().await.map_err(internal)?;
    let path = bundle_path(&runtime, app, patch, bundle).await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        inspection::preview(&path.path, file)
    })
    .await
    .map_err(internal)?
    .map(Json)
}

async fn download(
    State(runtime): State<Runtime>,
    Path((app, patch, bundle)): Path<(String, i64, i64)>,
    request: Request,
) -> Result<Response> {
    let _permit = INSPECTION_SLOTS.acquire().await.map_err(internal)?;
    let path = bundle_path(&runtime, app, patch, bundle).await?;
    let mut response = ServeFile::new(&path.path)
        .oneshot(request)
        .await
        .map_err(internal)?
        .map(Body::new);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"asset-bundle-{bundle}.zip\""
        ))
        .map_err(internal)?,
    );
    Ok(response)
}
