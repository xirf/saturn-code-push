use super::super::models::{Actor, Session};
use crate::platform::{
    database::Connection,
    error::{Result, internal},
};

pub(in crate::modules::identity) fn actor(
    c: &mut Connection<'_>,
    hash: &str,
    now: i64,
) -> Result<Option<Actor>> {
    Ok(c.query_opt("SELECT a.* FROM dashboard_sessions s JOIN identity_accounts a ON a.id=s.identity WHERE s.token_hash=$1 AND s.expires_at>$2 AND NOT a.disabled", &[&hash,&now]).map_err(internal)?.map(|r| Actor::account(&super::account_repository::decode(&r))))
}
pub(in crate::modules::identity) fn remove(c: &mut Connection<'_>, hash: &str) -> Result<()> {
    c.execute(
        "DELETE FROM dashboard_sessions WHERE token_hash=$1",
        &[&hash],
    )
    .map_err(internal)?;
    Ok(())
}
pub(in crate::modules::identity) fn rotate(
    c: &mut Connection<'_>,
    session: &Session,
    old: Option<&str>,
    now: i64,
) -> Result<()> {
    c.execute(
        "DELETE FROM dashboard_sessions WHERE expires_at<=$1",
        &[&now],
    )
    .map_err(internal)?;
    if let Some(old) = old {
        remove(c, old)?;
    }
    c.execute(
        "INSERT INTO dashboard_sessions(token_hash,identity,expires_at) VALUES($1,$2,$3)",
        &[&session.token_hash, &session.identity, &session.expires_at],
    )
    .map_err(internal)?;
    Ok(())
}
