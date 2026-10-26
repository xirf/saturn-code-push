use crate::bootstrap::Runtime;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub(super) fn routes(runtime: &Runtime) -> axum::Router<Runtime> {
    use axum::{Router, routing::get};
    use tower_http::services::{ServeDir, ServeFile};

    let directory = &runtime.console_dir;
    Router::new()
        .route("/", get(index))
        .route("/login", get(index))
        .route("/applications", get(index))
        .route("/releases", get(index))
        .route("/releases/{release}", get(index))
        .route("/patches", get(index))
        .route("/analytics", get(index))
        .route("/settings", get(index))
        .route("/docs", get(index))
        .route("/docs/{page}", get(index))
        .nest_service("/_nuxt", ServeDir::new(directory.join("_nuxt")))
        .nest_service("/licenses", ServeDir::new(directory.join("licenses")))
        .route_service(
            "/favicon.svg",
            ServeFile::new(directory.join("favicon.svg")),
        )
}

pub(super) async fn index(State(runtime): State<Runtime>) -> Response {
    let root = runtime.console_dir;
    match tokio::fs::read_to_string(root.join("index.html")).await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Html(concat!(
                "<h1>Saturn console needs a build</h1>",
                "<p>Run npm ci and npm run build in console/, ",
                "or start the Docker service.</p>",
            )),
        )
            .into_response(),
    }
}
