use super::super::models::Actor;
use crate::platform::{
    database::Connection,
    error::{Result, internal},
    values::now,
};
use serde_json::{Value, json};

pub(in crate::modules::identity) fn list(c: &mut Connection<'_>) -> Result<Vec<Value>> {
    Ok(c.query("SELECT k.id,k.name,k.prefix,k.app_id,k.permission,k.created_by,a.username,k.created_at,k.expires_at,k.revoked_at,k.last_used_at FROM identity_api_keys k JOIN identity_accounts a ON a.id=k.created_by ORDER BY k.created_at DESC", &[]).map_err(internal)?.iter().map(|r| json!({"id":r.get::<_,String>(0),"name":r.get::<_,String>(1),"prefix":r.get::<_,String>(2),"app_id":r.get::<_,String>(3),"permission":r.get::<_,String>(4),"created_by":r.get::<_,String>(5),"creator":r.get::<_,String>(6),"created_at":r.get::<_,String>(7),"expires_at":r.get::<_,i64>(8),"revoked_at":r.get::<_,Option<String>>(9),"last_used_at":r.get::<_,Option<String>>(10)})).collect())
}
pub(in crate::modules::identity) fn authenticate(
    c: &mut Connection<'_>,
    hash: &str,
    timestamp: i64,
) -> Result<Option<Actor>> {
    let row=c.query_opt("SELECT k.id,k.name,k.app_id,k.permission,a.role FROM identity_api_keys k JOIN identity_accounts a ON a.id=k.created_by WHERE k.token_hash=$1 AND k.revoked_at IS NULL AND k.expires_at>$2 AND NOT a.disabled AND a.role IN ('admin','publisher')", &[&hash,&timestamp]).map_err(internal)?;
    let Some(r) = row else { return Ok(None) };
    let id: String = r.get(0);
    c.execute(
        "UPDATE identity_api_keys SET last_used_at=$1 WHERE id=$2",
        &[&now(), &id],
    )
    .map_err(internal)?;
    Ok(Some(Actor {
        id,
        name: r.get(1),
        app_id: Some(r.get(2)),
        permission: Some(r.get(3)),
        role: r.get(4),
        kind: "api_key".into(),
    }))
}
