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
    let mut request = request;
    let supplied = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let legacy = supplied.is_some_and(|token| {
        !token.is_empty()
            && runtime.legacy_token_enabled
            && bool::from(
                Sha256::digest(token.as_bytes())
                    .as_slice()
                    .ct_eq(&runtime.token_digest),
            )
    });
    let actor = if legacy {
        Ok(Some(identity::api::Actor {
            id: "legacy-management-token".into(),
            name: "Legacy management token".into(),
            kind: "legacy_token".into(),
            role: "admin".into(),
            app_id: None,
            permission: None,
        }))
    } else if let Some(token) = supplied {
        identity::api::key_actor(&runtime, token).await
    } else {
        identity::api::session_actor(&runtime, request.headers()).await
    };
    let actor = match actor {
        Ok(Some(actor)) => actor,
        Ok(None) => {
            return ApiError(StatusCode::UNAUTHORIZED, "Sign in to continue".into())
                .into_response();
        }
        Err(error) => return error.into_response(),
    };
    let read = matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    );
    if actor.kind == "account" && !read && !identity::api::same_origin(&runtime, request.headers())
    {
        return ApiError(
            StatusCode::FORBIDDEN,
            "Request origin is not allowed".into(),
        )
        .into_response();
    }
    let path = request.uri().path().to_owned();
    let parts: Vec<_> = path.split('/').collect();
    let app = if parts.get(1) == Some(&"api")
        && parts.get(2) == Some(&"v1")
        && parts.get(3) == Some(&"apps")
    {
        parts
            .get(4)
            .and_then(|v| percent_encoding::percent_decode_str(v).decode_utf8().ok())
            .map(|v| v.into_owned())
    } else {
        None
    };
    let allowed = if actor.kind == "api_key" {
        (read
            && ["/api/v1/users/me", "/api/v1/plan", "/api/v1/organizations"]
                .contains(&path.as_str()))
            || (read && path == "/api/v1/apps" && actor.permission.as_deref() == Some("publish"))
            || (app.is_some()
                && app == actor.app_id
                && (actor.permission.as_deref() == Some("publish")
                    || (actor.permission.as_deref() == Some("deployments:write")
                        && parts.get(5) == Some(&"deployments"))))
    } else if path == "/auth/password" {
        actor.kind == "account"
    } else if path.starts_with("/admin/")
        && !["/admin/state", "/admin/analytics"].contains(&path.as_str())
    {
        actor.role == "admin" && actor.kind == "account"
    } else {
        read || actor.role == "admin" || (actor.role == "publisher" && app.is_some())
    };
    if !allowed {
        return ApiError(
            StatusCode::FORBIDDEN,
            "Your role or API key does not permit this action".into(),
        )
        .into_response();
    }
    request.extensions_mut().insert(actor.clone());
    let context = crate::platform::database::AuditContext {
        actor,
        action: request.method().to_string(),
        resource: path,
        app_id: app,
        write: !read,
    };
    crate::platform::database::AUDIT_CONTEXT
        .scope(context, next.run(request))
        .await
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
