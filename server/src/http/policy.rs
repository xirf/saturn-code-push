use super::middleware;
use crate::bootstrap::Runtime;
use axum::{Router, extract::DefaultBodyLimit};

pub(crate) fn management(runtime: &Runtime, routes: Router<Runtime>) -> Router<Runtime> {
    routes.layer(DefaultBodyLimit::max(64 * 1024)).route_layer(
        axum::middleware::from_fn_with_state(runtime.clone(), middleware::authorize),
    )
}

pub(crate) fn device(runtime: &Runtime, routes: Router<Runtime>) -> Router<Runtime> {
    routes.layer(DefaultBodyLimit::max(64 * 1024)).route_layer(
        axum::middleware::from_fn_with_state(runtime.clone(), middleware::throttle),
    )
}
