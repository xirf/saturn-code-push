//! First-account setup requires deployment ownership and closes after any account exists.
use super::{
    api, http,
    models::{Actor, Session},
    repositories::{account_repository, session_repository},
    service,
};
use crate::{
    bootstrap::Runtime,
    platform::{
        error::{ApiError, Result, bad, conflict, internal},
        values::{now, uid},
    },
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub(super) fn routes() -> Router<Runtime> {
    Router::new().route(
        "/setup",
        get(status)
            .post(complete)
            .layer(DefaultBodyLimit::max(4096)),
    )
}
async fn status(State(runtime): State<Runtime>) -> Result<Json<Value>> {
    let required = runtime.database.call(api::needs_setup).await?;
    Ok(Json(
        json!({"required":required,"enabled":required && runtime.setup_token_digest.is_some()}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Setup {
    setup_token: String,
    username: String,
    password: String,
}
async fn complete(
    State(runtime): State<Runtime>,
    headers: HeaderMap,
    Json(input): Json<Setup>,
) -> Result<Response> {
    if !api::same_origin(&runtime, &headers) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Request origin is not allowed".into(),
        ));
    }
    if !runtime.database.call(api::needs_setup).await? {
        return Err(conflict(
            "Setup is complete. Sign in with an existing account.",
        ));
    }
    if let Some(response) = http::limit_attempt(&runtime)? {
        return Ok(response);
    }
    let digest = runtime.setup_token_digest.ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Set SETUP_TOKEN in deployment settings to enable first-run setup.".into(),
        )
    })?;
    if input.setup_token.len() > 256
        || !bool::from(
            Sha256::digest(input.setup_token.as_bytes())
                .as_slice()
                .ct_eq(&digest),
        )
    {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Incorrect setup token".into(),
        ));
    }
    if input.username.is_empty()
        || input.username.len() > 128
        || input.username.trim() != input.username
        || input.username.chars().any(char::is_control)
    {
        return Err(bad(
            "Use a username of 1 to 128 characters without surrounding spaces or control characters",
        ));
    }
    if !(12..=1024).contains(&input.password.len()) {
        return Err(bad("Passwords must contain 12 to 1024 bytes"));
    }
    let slot = runtime
        .login_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "Setup is busy. Try again shortly.".into(),
            )
        })?;
    let hash = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        service::hash_password(&input.password)
    })
    .await
    .map_err(internal)??;
    let token = format!("{}{}", uid(), uid());
    let token_hash = service::token_hash(&token);
    let actor=runtime.database.call(move |c| {
        // The shared PostgreSQL lock makes simultaneous setup requests single-winner.
        if !api::needs_setup(c)? {return Err(conflict("Setup is complete. Sign in with an existing account."))}
        let account=account_repository::insert(c,&input.username,&hash,"admin")?;
        let session=Session{token_hash,identity:account.id.clone(),expires_at:chrono::Utc::now().timestamp()+http::SESSION_SECONDS};
        session_repository::rotate(c,&session,None,chrono::Utc::now().timestamp())?;
        c.execute("INSERT INTO identity_audit(actor_id,actor_name,actor_kind,action,resource,occurred_at) VALUES($1,$2,'account','SETUP','/auth/setup',$3)",&[&account.id,&account.username,&now()]).map_err(internal)?;
        Ok(Actor::account(&account))
    }).await?;
    http::response(
        &runtime,
        &token,
        http::SESSION_SECONDS,
        json!({"id":actor.id,"username":actor.name,"role":actor.role}),
    )
}
