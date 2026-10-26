mod bundles;
mod inspection;
mod registration;
mod transfers;

use crate::{bootstrap::Runtime, http::policy};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    Router::new()
        .route(
            "/uploads/{key}",
            post(transfers::upload).layer(DefaultBodyLimit::max(256 * 1024 * 1024 + 64 * 1024)),
        )
        .route("/objects/{key}", get(transfers::download))
        .route(
            "/asset-uploads/{key}",
            post(bundles::upload).layer(DefaultBodyLimit::max(32 * 1024 * 1024)),
        )
        .route("/asset-objects/{key}", get(bundles::download))
        .merge(policy::device(
            runtime,
            Router::new().route("/api/v1/assets/check", get(bundles::check)),
        ))
}

pub(crate) fn release_routes() -> Router<Runtime> {
    Router::new().route(
        "/artifacts",
        get(registration::list_artifacts).post(registration::register_release_artifact),
    )
}

pub(crate) fn patch_routes() -> Router<Runtime> {
    Router::new()
        .route("/artifacts", post(registration::register_patch_artifact))
        .route("/asset-bundle", post(bundles::register))
        .nest("/asset-bundles/{bundle}", inspection::routes())
}
