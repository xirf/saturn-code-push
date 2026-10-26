use crate::{bootstrap::Runtime, platform::error::Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::Value;
pub(crate) async fn list_channels(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::list_channels(connection, app))
        .await
        .map(Json)
}

pub(crate) async fn create_channel(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::create_channel(connection, app, input))
        .await
        .map(Json)
}
pub(super) fn routes() -> Router<Runtime> {
    Router::new().route("/channels", get(list_channels).post(create_channel))
}
