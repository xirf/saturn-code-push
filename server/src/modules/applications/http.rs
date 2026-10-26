mod channels;
use crate::{bootstrap::Runtime, platform::error::Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::Value;
pub(crate) async fn list_apps(State(runtime): State<Runtime>) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database.call(super::service::list_apps).await.map(Json)
}

pub(crate) async fn get_app(
    State(runtime): State<Runtime>,
    Path(app): Path<String>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::service::get_app(connection, app))
        .await
        .map(Json)
}

pub(crate) async fn create_app(
    State(runtime): State<Runtime>,
    Json(input): Json<Value>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| super::service::create_app(connection, input))
        .await
        .map(Json)
}
pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    let routes = Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/apps", get(list_apps).post(create_app))
            .route("/organizations/{org}/apps", get(list_apps))
            .route("/apps/{app}", get(get_app))
            .nest("/apps/{app}", channels::routes()),
    );
    crate::http::policy::management(runtime, routes)
}
