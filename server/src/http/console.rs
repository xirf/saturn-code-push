use crate::bootstrap::Runtime;
use axum::{
    extract::State,
    http::{header, Method, Request, StatusCode},
    response::{Html, IntoResponse, Response},
};

pub(super) fn routes(runtime: &Runtime) -> axum::Router<Runtime> {
    use axum::{Router, routing::get};
    use tower_http::services::{ServeDir, ServeFile};

    let directory = &runtime.console_dir;
    Router::new()
        .route("/", get(index))
        .nest_service("/_nuxt", ServeDir::new(directory.join("_nuxt")))
        .nest_service("/licenses", ServeDir::new(directory.join("licenses")))
        .route_service(
            "/favicon.svg",
            ServeFile::new(directory.join("favicon.svg")),
        )
}

pub(super) async fn fallback(
    State(runtime): State<Runtime>,
    request: Request<axum::body::Body>,
) -> Response {
    let path = request.uri().path();
    let is_server_route = ["/api", "/auth", "/admin"]
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")));
    let accepts_html = request
        .headers()
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|part| part.trim().starts_with("text/html")));

    if request.method() != Method::GET || is_server_route || !accepts_html {
        return StatusCode::NOT_FOUND.into_response();
    }

    index(State(runtime)).await
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
