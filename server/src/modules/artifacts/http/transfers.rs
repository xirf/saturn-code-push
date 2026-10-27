use crate::platform::{error::internal, objects::Kind};
use crate::{
    bootstrap::Runtime,
    platform::error::{Result, bad},
};
use axum::response::{IntoResponse, Redirect};
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use axum::{body::Body, extract::Request, response::Response};
use serde_json::Value;
use tower::ServiceExt;
use tower_http::services::ServeFile;
pub(crate) async fn upload(
    State(runtime): State<Runtime>,
    Path(key): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    let artifact = runtime
        .database
        .call(move |connection| super::super::service::pending_binary(connection, &key))
        .await?;
    let mut upload = super::super::upload::Upload::new(runtime.objects.directory(), artifact.size)?;
    let mut found = false;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| bad(error.to_string()))?
    {
        if field.name() != Some("file") || found {
            return Err(bad("Expected exactly one file"));
        }
        found = true;
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|error| bad(error.to_string()))?
        {
            upload.chunk(&chunk).await?;
        }
    }
    if !found {
        return Err(bad("Expected exactly one file"));
    }
    // A patch hash describes the reconstructed executable, not the uploaded diff.
    let expected_hash = artifact
        .patch_id
        .is_none()
        .then_some(artifact.hash.as_str());
    let temporary = upload.finish(expected_hash).await?;
    let location = runtime.objects.destination(
        Kind::Binary,
        &artifact.app_id,
        artifact.release_id,
        artifact.patch_id,
        artifact.id,
    );
    super::super::upload::publish(
        runtime.database,
        temporary,
        runtime.objects,
        location,
        Kind::Binary,
        artifact.id,
        move |connection| super::super::service::complete_binary(connection, artifact.id),
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
        .call(move |connection| super::super::service::download_binary(connection, &key))
        .await?;
    let location = runtime
        .objects
        .location(&runtime.database, Kind::Binary, id)
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
