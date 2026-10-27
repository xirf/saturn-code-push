pub(crate) use super::{
    models::{Actor, Credentials},
    service::hash_password,
};
use crate::{
    bootstrap::Runtime,
    platform::error::{Result, internal},
};
use axum::http::{HeaderMap, header};
use chrono::Utc;
pub(crate) fn bootstrap(
    c: &mut crate::platform::database::Connection<'_>,
    credentials: &Credentials,
) -> Result<()> {
    super::repositories::account_repository::bootstrap(c, credentials)
}

pub(crate) fn same_origin(runtime: &Runtime, headers: &HeaderMap) -> bool {
    headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) == Some(runtime.base_url.as_str())
}
pub(crate) async fn session_actor(runtime: &Runtime, headers: &HeaderMap) -> Result<Option<Actor>> {
    let Some(token) = super::http::cookie_token(headers) else {
        return Ok(None);
    };
    let hash = super::service::token_hash(token);
    runtime
        .database
        .call(move |c| {
            super::repositories::session_repository::actor(c, &hash, Utc::now().timestamp())
        })
        .await
}
pub(crate) async fn key_actor(runtime: &Runtime, token: &str) -> Result<Option<Actor>> {
    if token.len() > 256 {
        return Ok(None);
    }
    let hash = super::service::token_hash(token);
    runtime
        .database
        .call(move |c| {
            super::repositories::key_repository::authenticate(c, &hash, Utc::now().timestamp())
        })
        .await
}

/// Local maintenance command: no HTTP route exposes this operation.
pub(crate) fn reset_password(
    c: &mut crate::platform::database::Connection<'_>,
    username: &str,
    password: &str,
) -> Result<()> {
    use crate::platform::{
        error::{bad, internal, missing},
        values::now,
    };
    if !(12..=1024).contains(&password.len()) {
        return Err(bad("Password must contain 12 to 1024 bytes"));
    }
    let account = super::repositories::account_repository::find_login(c, username)?
        .ok_or_else(missing)?
        .0;
    let hash = hash_password(password)?;
    c.execute(
        "UPDATE identity_accounts SET password_hash=$1,updated_at=$2 WHERE id=$3",
        &[&hash, &now(), &account.id],
    )
    .map_err(internal)?;
    c.execute(
        "DELETE FROM dashboard_sessions WHERE identity=$1",
        &[&account.id],
    )
    .map_err(internal)?;
    c.execute(
        "UPDATE identity_api_keys SET revoked_at=$1 WHERE created_by=$2 AND revoked_at IS NULL",
        &[&now(), &account.id],
    )
    .map_err(internal)?;
    c.execute("INSERT INTO identity_audit(actor_id,actor_name,actor_kind,action,resource,occurred_at) VALUES('local-maintenance','Local maintenance command','system','RESET_PASSWORD',$1,$2)", &[&format!("account:{}",account.id),&now()]).map_err(internal)?;
    Ok(())
}

pub(crate) fn validate_actor(
    c: &mut crate::platform::database::Connection<'_>,
    context: &crate::platform::database::AuditContext,
) -> Result<()> {
    let a = &context.actor;
    let valid=match a.kind.as_str() {
        "account" => c.query_one("SELECT EXISTS(SELECT 1 FROM identity_accounts WHERE id=$1 AND NOT disabled AND role=$2)", &[&a.id,&a.role]).map_err(internal)?.get::<_,bool>(0),
        "api_key" => c.query_one("SELECT EXISTS(SELECT 1 FROM identity_api_keys k JOIN identity_accounts a ON a.id=k.created_by WHERE k.id=$1 AND k.revoked_at IS NULL AND k.expires_at>$2 AND NOT a.disabled AND a.role IN ('admin','publisher'))", &[&a.id,&chrono::Utc::now().timestamp()]).map_err(internal)?.get::<_,bool>(0),
        "legacy_token"=>true,
        _=>false,
    };
    if !valid {
        return Err(crate::platform::error::ApiError(
            axum::http::StatusCode::UNAUTHORIZED,
            "Access changed. Authenticate again".into(),
        ));
    }
    Ok(())
}

pub(crate) fn append_audit(
    c: &mut crate::platform::database::Connection<'_>,
    context: &crate::platform::database::AuditContext,
) -> Result<()> {
    super::repositories::audit_repository::append(c, context)
}

pub(crate) fn needs_setup(c: &mut crate::platform::database::Connection<'_>) -> Result<bool> {
    super::repositories::account_repository::has_accounts(c).map(|exists| !exists)
}
