use crate::{bootstrap::Runtime, platform::error::Result};
use axum::http::{HeaderMap, header};
use chrono::Utc;

pub(crate) use super::repositories::session_repository::initialize;
pub(crate) use super::{models::Credentials, service::hash_password};

pub(crate) fn same_origin(runtime: &Runtime, headers: &HeaderMap) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        == Some(runtime.base_url.as_str())
}

pub(crate) async fn session_username(
    runtime: &Runtime,
    headers: &HeaderMap,
) -> Result<Option<String>> {
    let Some(credentials) = runtime.dashboard.clone() else {
        return Ok(None);
    };
    let Some(token) = super::http::cookie_token(headers) else {
        return Ok(None);
    };
    let token_hash = super::service::token_hash(token);
    runtime
        .database
        .call(move |connection| {
            let valid = super::repositories::session_repository::exists(
                connection,
                &token_hash,
                &credentials.identity,
                Utc::now().timestamp(),
            )?;
            Ok(valid.then(|| credentials.username.clone()))
        })
        .await
}
