use crate::{bootstrap::Runtime, platform::error::Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};
use serde_json::Value;
pub(crate) async fn create_patch(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::create_patch(connection, app, input))
        .await
        .map(Json)
}

pub(crate) async fn promote(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::promote(connection, app, input))
        .await
        .map(Json)
}

pub(crate) async fn rollback(
    State(runtime): State<Runtime>,
    Path((app, release, patch)): Path<(String, i64, i64)>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::rollback(connection, app, release, patch))
        .await
        .map(Json)
}

pub(crate) async fn rollforward(
    State(runtime): State<Runtime>,
    Path((app, release, patch)): Path<(String, i64, i64)>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::rollforward(connection, app, release, patch))
        .await
        .map(Json)
}
pub(super) fn routes() -> Router<Runtime> {
    Router::new()
        .route("/patches", post(create_patch))
        .route("/patches/promote", post(promote))
        .nest(
            "/patches/{patch}",
            crate::modules::artifacts::http::patch_routes(),
        )
}

pub(super) fn release_patch_routes() -> Router<Runtime> {
    Router::new()
        .route("/rollback", post(rollback))
        .route("/rollforward", post(rollforward))
}
