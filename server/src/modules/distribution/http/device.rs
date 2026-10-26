use crate::{bootstrap::Runtime, platform::error::Result};
use axum::{Json, Router, extract::State, routing::post};
use serde_json::Value;
pub(crate) async fn check_patch(
    State(runtime): State<Runtime>,
    Json(query): Json<super::super::dto::Check>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::super::service::check_patch(&runtime, connection, query))
        .await
        .map(Json)
}
pub(super) fn routes(runtime: &Runtime) -> Router<Runtime> {
    crate::http::policy::device(
        runtime,
        Router::new().route("/api/v1/patches/check", post(check_patch)),
    )
}
