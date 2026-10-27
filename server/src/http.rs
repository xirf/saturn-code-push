mod admin;
mod compatibility;
mod console;
mod middleware;
pub(crate) mod policy;

use crate::{
    bootstrap::Runtime,
    modules::{applications, artifacts, distribution, identity, telemetry},
};
use axum::{
    Json, Router,
    http::{HeaderValue, header},
    routing::get,
};
use serde_json::{Value, json};
use tower_http::set_header::SetResponseHeaderLayer;

pub(crate) fn router(runtime: Runtime) -> Router {
    Router::new()
        .merge(applications::http::routes(&runtime))
        .merge(distribution::http::routes(&runtime))
        .merge(artifacts::http::routes(&runtime))
        .merge(identity::http::routes())
        .merge(crate::modules::github::routes(&runtime))
        .merge(identity::management::routes(&runtime))
        .merge(telemetry::http::routes(&runtime))
        .merge(crate::modules::deployments::routes(&runtime))
        .merge(admin::routes(&runtime))
        .merge(compatibility::routes(&runtime))
        .merge(console::routes(&runtime))
        .route("/health", get(health))
        .layer(axum::middleware::from_fn(middleware::adapt_dart_json))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(concat!(
                "default-src 'self'; script-src 'self'; ",
                "style-src 'self' 'unsafe-inline'; font-src 'self'; ",
                "img-src 'self' data:; connect-src 'self'; ",
                "frame-ancestors 'none'; base-uri 'none'; ",
                "form-action 'self'; object-src 'none'",
            )),
        ))
        .with_state(runtime)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok", "service": "saturn-code-push"}))
}
