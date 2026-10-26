use crate::{bootstrap::Runtime, platform::error::Result};
use axum::{
    Json,
    extract::{Query, State},
};
use axum::{Router, routing::get};
use serde_json::Value;
use std::collections::HashMap;
pub(crate) async fn event(
    State(runtime): State<Runtime>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::service::event(connection, input))
        .await
        .map(Json)
}

pub(crate) async fn analytics(
    State(runtime): State<Runtime>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::service::analytics(connection, query))
        .await
        .map(Json)
}

pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    let management = crate::http::policy::management(
        runtime,
        Router::new().route("/admin/analytics", get(analytics)),
    );
    let device = crate::http::policy::device(
        runtime,
        Router::new().route("/api/v1/patches/events", axum::routing::post(event)),
    );
    management.merge(device)
}
