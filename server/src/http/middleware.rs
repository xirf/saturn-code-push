use crate::{bootstrap::Runtime, modules::identity, platform::error::ApiError};
use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
pub(super) async fn authorize(
    State(runtime): State<Runtime>,
    request: Request,
    next: Next,
) -> Response {
    let supplied = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let bearer = supplied.is_some_and(|token| {
        let digest = Sha256::digest(token.as_bytes());
        bool::from(digest.as_slice().ct_eq(&runtime.token_digest))
    });
    if !bearer {
        match identity::api::session_username(&runtime, request.headers()).await {
            Ok(Some(_)) => {
                if !matches!(
                    *request.method(),
                    axum::http::Method::GET | axum::http::Method::HEAD
                ) && !identity::api::same_origin(&runtime, request.headers())
                {
                    return ApiError(
                        StatusCode::FORBIDDEN,
                        "Request origin is not allowed".into(),
                    )
                    .into_response();
                }
            }
            Ok(None) => {
                return ApiError(StatusCode::UNAUTHORIZED, "Sign in to continue".into())
                    .into_response();
            }
            Err(e) => return e.into_response(),
        }
    }
    next.run(request).await
}
// Shorebird's Dart client sends encoded JSON strings as text/plain. Match the
// upstream wire behavior without changing multipart registrations or uploads.
pub(super) async fn adapt_dart_json(mut request: Request, next: Next) -> Response {
    if request.uri().path().starts_with("/api/v1/")
        && request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|input| input.to_str().ok())
            .is_some_and(|input| input.starts_with("text/plain"))
    {
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }
    next.run(request).await
}

pub(super) async fn throttle(
    State(runtime): State<Runtime>,
    request: Request,
    next: Next,
) -> Response {
    let allowed = match runtime.rate.lock() {
        Ok(mut rate) => {
            if rate.0.elapsed().as_secs() >= 1 {
                *rate = (std::time::Instant::now(), 0);
            }
            rate.1 = rate.1.saturating_add(1);
            rate.1 <= 100
        }
        Err(_) => false,
    };
    if !allowed {
        return ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Device request limit reached; retry later".into(),
        )
        .into_response();
    }
    next.run(request).await
}
