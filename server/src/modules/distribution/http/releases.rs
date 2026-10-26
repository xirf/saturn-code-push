use crate::{bootstrap::Runtime, platform::error::Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, patch},
};
use serde_json::Value;
pub(crate) async fn list_releases(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::list_releases(connection, app))
        .await
        .map(Json)
}

pub(crate) async fn create_release(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::create_release(connection, app, input))
        .await
        .map(Json)
}

pub(crate) async fn update_release(
    State(runtime): State<Runtime>,
    Path((app, id)): Path<(String, i64)>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::update_release(connection, app, id, input))
        .await
        .map(Json)
}

pub(crate) async fn list_patches(
    State(runtime): State<Runtime>,
    Path((app, id)): Path<(String, i64)>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::list_patches(&runtime, connection, app, id))
        .await
        .map(Json)
}
pub(super) fn routes() -> Router<Runtime> {
    Router::new()
        .route("/releases", get(list_releases).post(create_release))
        .route("/releases/{release}", patch(update_release))
        .nest("/releases/{release}", detail_routes())
}

fn detail_routes() -> Router<Runtime> {
    Router::new()
        .route("/patches", get(list_patches))
        .nest("/patches/{patch}", super::patches::release_patch_routes())
        .merge(crate::modules::artifacts::http::release_routes())
}
