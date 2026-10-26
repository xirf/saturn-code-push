use super::{api, models::Session, repositories::session_repository, service};
use crate::{
    bootstrap::Runtime,
    platform::{
        error::{ApiError, Result, internal},
        values::uid,
    },
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

const COOKIE: &str = "saturn_session";
const SESSION_SECONDS: i64 = 8 * 60 * 60;

pub(super) fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == COOKIE
                && value.len() == 72
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() || byte == b'-'))
            .then_some(value)
        })
}

fn response(runtime: &Runtime, token: &str, age: i64, value: Value) -> Result<Response> {
    let secure = if runtime.base_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let cookie =
        format!("{COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={age}{secure}");
    let mut response = Json(value).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(internal)?,
    );
    Ok(response)
}

#[derive(Deserialize)]
pub(crate) struct Login {
    username: String,
    password: String,
}

pub(crate) async fn login(
    State(runtime): State<Runtime>,
    headers: HeaderMap,
    Json(input): Json<Login>,
) -> Result<Response> {
    if !api::same_origin(&runtime, &headers) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Request origin is not allowed".into(),
        ));
    }
    let credentials = runtime.dashboard.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Dashboard login has not been configured".into(),
        )
    })?;
    let allowed = {
        let mut rate = runtime.login_rate.lock().map_err(internal)?;
        if rate.0.elapsed().as_secs() >= 60 {
            *rate = (std::time::Instant::now(), 0);
        }
        rate.1 = rate.1.saturating_add(1);
        rate.1 <= 10
    };
    if !allowed {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many sign-in attempts. Try again in a minute.".into(),
        ));
    }
    if input.username.len() > 128 || input.password.is_empty() || input.password.len() > 1024 {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Incorrect username or password".into(),
        ));
    }
    let slot = runtime
        .login_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "Sign-in is busy. Try again shortly.".into(),
            )
        })?;
    let check = credentials.clone();
    let valid = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        service::verify(&check, &input.username, &input.password)
    })
    .await
    .map_err(internal)?;
    if !valid {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Incorrect username or password".into(),
        ));
    }
    let token = format!("{}{}", uid(), uid());
    let now = Utc::now().timestamp();
    let session = Session {
        token_hash: service::token_hash(&token),
        identity: credentials.identity.clone(),
        expires_at: now + SESSION_SECONDS,
    };
    let old_hash = cookie_token(&headers).map(service::token_hash);
    runtime
        .database
        .call(move |connection| {
            session_repository::rotate(connection, &session, old_hash.as_deref(), now)
        })
        .await?;
    response(
        &runtime,
        &token,
        SESSION_SECONDS,
        json!({"username": credentials.username}),
    )
}

pub(crate) async fn session(
    State(runtime): State<Runtime>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let username = api::session_username(&runtime, &headers)
        .await?
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Sign in to continue".into()))?;
    Ok(Json(json!({"username": username})))
}

pub(crate) async fn logout(State(runtime): State<Runtime>, headers: HeaderMap) -> Result<Response> {
    if !api::same_origin(&runtime, &headers) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Request origin is not allowed".into(),
        ));
    }
    if let Some(token) = cookie_token(&headers) {
        let hash = service::token_hash(token);
        runtime
            .database
            .call(move |connection| session_repository::remove(connection, &hash))
            .await?;
    }
    response(&runtime, "", 0, json!({}))
}

pub(crate) fn routes() -> axum::Router<Runtime> {
    use axum::{
        Router,
        extract::DefaultBodyLimit,
        routing::{get, post},
    };
    Router::new().nest(
        "/auth",
        Router::new()
            .route("/login", post(login).layer(DefaultBodyLimit::max(4096)))
            .route("/logout", post(logout))
            .route("/session", get(session)),
    )
}
