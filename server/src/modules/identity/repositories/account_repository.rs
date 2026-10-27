use super::super::models::{Account, Credentials};
use crate::platform::{
    database::Connection,
    error::{Result, internal},
    values::{now, uid},
};
use postgres::Row;

pub(super) fn decode(row: &Row) -> Account {
    Account {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        disabled: row.get("disabled"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
pub(in crate::modules::identity) fn list(c: &mut Connection<'_>) -> Result<Vec<Account>> {
    Ok(c.query("SELECT id, username, role, disabled, created_at, updated_at FROM identity_accounts ORDER BY created_at,id", &[]).map_err(internal)?.iter().map(decode).collect())
}
pub(in crate::modules::identity) fn find_login(
    c: &mut Connection<'_>,
    username: &str,
) -> Result<Option<(Account, String)>> {
    Ok(c.query_opt(
        "SELECT * FROM identity_accounts WHERE username=$1",
        &[&username],
    )
    .map_err(internal)?
    .map(|r| (decode(&r), r.get("password_hash"))))
}
pub(in crate::modules::identity) fn insert(
    c: &mut Connection<'_>,
    username: &str,
    hash: &str,
    role: &str,
) -> Result<Account> {
    let id = uid();
    let timestamp = now();
    let r=c.query_one("INSERT INTO identity_accounts(id,username,password_hash,role,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$5) RETURNING *", &[&id,&username,&hash,&role,&timestamp]).map_err(internal)?;
    Ok(decode(&r))
}
pub(in crate::modules::identity) fn bootstrap(
    c: &mut Connection<'_>,
    credentials: &Credentials,
) -> Result<()> {
    let count: i64 = c
        .query_one("SELECT COUNT(*) FROM identity_accounts", &[])
        .map_err(internal)?
        .get(0);
    if count == 0 {
        insert(c, &credentials.username, &credentials.hash, "admin")?;
    }
    Ok(())
}

pub(in crate::modules::identity) fn has_accounts(c: &mut Connection<'_>) -> Result<bool> {
    Ok(
        c.query_one("SELECT EXISTS(SELECT 1 FROM identity_accounts)", &[])
            .map_err(internal)?
            .get(0),
    )
}
