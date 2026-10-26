use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
pub(crate) type Result<T> = std::result::Result<T, ApiError>;
#[derive(Debug)]
pub struct ApiError(pub(crate) StatusCode, pub(crate) String);

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.0, self.1)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"message":self.1}))).into_response()
    }
}

pub(crate) fn bad(message: impl Into<String>) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, message.into())
}

pub(crate) fn missing() -> ApiError {
    ApiError(StatusCode::NOT_FOUND, "Not found".into())
}

pub(crate) fn conflict(message: &str) -> ApiError {
    ApiError(StatusCode::CONFLICT, message.into())
}

pub(crate) fn internal(error: impl std::fmt::Display) -> ApiError {
    tracing::error!("storage error: {error}");
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Storage operation failed".into(),
    )
}
