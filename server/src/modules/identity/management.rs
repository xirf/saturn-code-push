use super::{
    api::Actor,
    repositories::{account_repository, audit_repository, key_repository},
    service,
};
use crate::{
    bootstrap::Runtime,
    http::policy,
    platform::{
        database::Connection,
        error::{Result, bad, conflict, internal, missing},
        values::{now, uid},
    },
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::{get, patch, post},
};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    policy::management(
        runtime,
        Router::new()
            .route("/admin/accounts", get(accounts).post(create_account))
            .route("/admin/accounts/{id}", patch(update_account))
            .route("/admin/api-keys", get(keys).post(create_key))
            .route("/admin/api-keys/{id}/revoke", post(revoke_key))
            .route("/admin/audit", get(audit))
            .route("/auth/password", post(change_password)),
    )
}
fn role_valid(role: &str) -> Result<()> {
    if !["admin", "publisher", "viewer"].contains(&role) {
        return Err(bad("Invalid account role"));
    }
    Ok(())
}
async fn hash(password: String) -> Result<String> {
    if !(12..=1024).contains(&password.len()) {
        return Err(bad("Passwords must contain 12 to 1024 bytes"));
    }
    tokio::task::spawn_blocking(move || service::hash_password(&password))
        .await
        .map_err(internal)?
}
async fn accounts(State(rt): State<Runtime>) -> Result<Json<Value>> {
    rt.database
        .call(|c| Ok(json!({"accounts":account_repository::list(c)?})))
        .await
        .map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewAccount {
    username: String,
    password: String,
    role: String,
}
async fn create_account(
    State(rt): State<Runtime>,
    Json(input): Json<NewAccount>,
) -> Result<Json<Value>> {
    role_valid(&input.role)?;
    if input.username.trim() != input.username
        || input.username.is_empty()
        || input.username.len() > 128
        || input.username.chars().any(char::is_control)
    {
        return Err(bad("Invalid username"));
    }
    let hash = hash(input.password).await?;
    rt.database
        .call(move |c| {
            if account_repository::find_login(c, &input.username)?.is_some() {
                return Err(conflict("Username already exists"));
            }
            Ok(json!(account_repository::insert(
                c,
                &input.username,
                &hash,
                &input.role
            )?))
        })
        .await
        .map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateAccount {
    role: Option<String>,
    disabled: Option<bool>,
    password: Option<String>,
}
async fn update_account(
    State(rt): State<Runtime>,
    Path(id): Path<String>,
    Json(input): Json<UpdateAccount>,
) -> Result<Json<Value>> {
    if let Some(role) = &input.role {
        role_valid(role)?
    }
    let hash = match input.password {
        Some(password) => Some(hash(password).await?),
        None => None,
    };
    rt.database.call(move |c| {
        let accounts=account_repository::list(c)?;
        let old=accounts.iter().find(|a|a.id==id).ok_or_else(missing)?;
        let role=input.role.unwrap_or_else(||old.role.clone());
        let disabled=input.disabled.unwrap_or(old.disabled);
        if old.role=="admin" && !old.disabled && (role!="admin" || disabled) && !accounts.iter().any(|a|a.id!=id && a.role=="admin" && !a.disabled) {
            return Err(conflict("Keep at least one active administrator"));
        }
        c.execute("UPDATE identity_accounts SET role=$1,disabled=$2,password_hash=COALESCE($3,password_hash),updated_at=$4 WHERE id=$5", &[&role,&disabled,&hash,&now(),&id]).map_err(internal)?;
        c.execute("DELETE FROM dashboard_sessions WHERE identity=$1", &[&id]).map_err(internal)?;
        // Keys cannot outlive a password reset, disable, or role change.
        c.execute("UPDATE identity_api_keys SET revoked_at=$1 WHERE created_by=$2 AND revoked_at IS NULL", &[&now(),&id]).map_err(internal)?;
        Ok(json!({"updated":true}))
    }).await.map(Json)
}
async fn keys(State(rt): State<Runtime>) -> Result<Json<Value>> {
    rt.database
        .call(|c| Ok(json!({"keys":key_repository::list(c)?})))
        .await
        .map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewKey {
    name: String,
    app_id: String,
    permission: String,
    expires_in_days: i64,
}
async fn create_key(
    State(rt): State<Runtime>,
    Extension(actor): Extension<Actor>,
    Json(input): Json<NewKey>,
) -> Result<Json<Value>> {
    if input.name.trim().is_empty()
        || input.name.len() > 128
        || !["deployments:write", "publish"].contains(&input.permission.as_str())
        || !(1..=365).contains(&input.expires_in_days)
    {
        return Err(bad(
            "Use a name, app, supported permission and expiry between 1 and 365 days",
        ));
    }
    rt.database.call(move |c| {
        crate::modules::applications::api::require(c,&input.app_id)?;
        let token=format!("sb_api_{}{}",uid().replace('-',""),uid().replace('-',""));
        let id=uid(); let prefix=&token[..14]; let timestamp=now(); let expires=chrono::Utc::now().timestamp()+input.expires_in_days*86400;
        c.execute("INSERT INTO identity_api_keys(id,name,token_hash,prefix,app_id,permission,created_by,created_at,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)", &[&id,&input.name,&service::token_hash(&token),&prefix,&input.app_id,&input.permission,&actor.id,&timestamp,&expires]).map_err(internal)?;
        Ok(json!({"id":id,"token":token}))
    }).await.map(Json)
}
async fn revoke_key(State(rt): State<Runtime>, Path(id): Path<String>) -> Result<Json<Value>> {
    rt.database
        .call(move |c| {
            let affected = c
                .execute(
                    "UPDATE identity_api_keys SET revoked_at=COALESCE(revoked_at,$1) WHERE id=$2",
                    &[&now(), &id],
                )
                .map_err(internal)?;
            if affected == 0 {
                return Err(missing());
            }
            Ok(json!({"revoked":true}))
        })
        .await
        .map(Json)
}
#[derive(Deserialize)]
struct AuditQuery {
    before: Option<i64>,
}
async fn audit(State(rt): State<Runtime>, Query(query): Query<AuditQuery>) -> Result<Json<Value>> {
    rt.database
        .call(move |c| {
            Ok(json!({"events":audit_repository::list(c,query.before.unwrap_or(i64::MAX))?}))
        })
        .await
        .map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordChange {
    current_password: String,
    password: String,
}
async fn change_password(
    State(rt): State<Runtime>,
    Extension(actor): Extension<Actor>,
    Json(input): Json<PasswordChange>,
) -> Result<Json<Value>> {
    let username = actor.name;
    let found = rt
        .database
        .call_read(move |c| account_repository::find_login(c, &username))
        .await?
        .ok_or_else(missing)?;
    let checked_hash = found.1.clone();
    let valid = tokio::task::spawn_blocking(move || {
        service::verify_hash(&found.1, &input.current_password)
    })
    .await
    .map_err(internal)?;
    if !valid {
        return Err(bad("Current password is incorrect"));
    }
    let hash = hash(input.password).await?;
    rt.database.call(move |c| {
        let affected=c.execute("UPDATE identity_accounts SET password_hash=$1,updated_at=$2 WHERE id=$3 AND password_hash=$4 AND NOT disabled", &[&hash,&now(),&actor.id,&checked_hash]).map_err(internal)?;
        if affected==0 {return Err(conflict("Credentials changed. Sign in again"))}
        revoke_account_access(c,&actor.id)?;
        Ok(json!({"updated":true}))
    }).await.map(Json)
}
fn revoke_account_access(c: &mut Connection<'_>, id: &str) -> Result<()> {
    c.execute("DELETE FROM dashboard_sessions WHERE identity=$1", &[&id])
        .map_err(internal)?;
    c.execute(
        "UPDATE identity_api_keys SET revoked_at=$1 WHERE created_by=$2 AND revoked_at IS NULL",
        &[&now(), &id],
    )
    .map_err(internal)?;
    Ok(())
}
