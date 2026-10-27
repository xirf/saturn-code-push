use crate::platform::{
    database::Connection,
    error::{Result, internal},
};
use serde_json::{Value, json};
pub(in crate::modules::identity) fn list(
    c: &mut Connection<'_>,
    before: i64,
) -> Result<Vec<Value>> {
    Ok(c.query("SELECT id,actor_id,actor_name,actor_kind,action,resource,app_id,occurred_at FROM identity_audit WHERE id<$1 ORDER BY id DESC LIMIT 100", &[&before]).map_err(internal)?.iter().map(|r| json!({"id":r.get::<_,i64>(0),"actor_id":r.get::<_,String>(1),"actor_name":r.get::<_,String>(2),"actor_kind":r.get::<_,String>(3),"action":r.get::<_,String>(4),"resource":r.get::<_,String>(5),"app_id":r.get::<_,Option<String>>(6),"occurred_at":r.get::<_,String>(7)})).collect())
}

pub(in crate::modules::identity) fn append(
    c: &mut Connection<'_>,
    context: &crate::platform::database::AuditContext,
) -> Result<()> {
    c.execute(
        "INSERT INTO identity_audit(actor_id, actor_name, actor_kind, action, resource, app_id, occurred_at)
         VALUES($1, $2, $3, $4, $5, $6, $7)",
        &[&context.actor.id, &context.actor.name, &context.actor.kind, &context.action,
          &context.resource, &context.app_id, &crate::platform::values::now()],
    ).map_err(internal)?;
    Ok(())
}
