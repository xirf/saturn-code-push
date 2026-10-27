pub(crate) use super::models::{Patch, Release};
use super::repositories::{patch_repository, release_repository};
use crate::platform::error::Result;
use postgres::Transaction as Connection;

pub(crate) fn releases(connection: &mut Connection<'_>) -> Result<Vec<Release>> {
    release_repository::list(connection)
}

pub(crate) fn patches(connection: &mut Connection<'_>) -> Result<Vec<Patch>> {
    patch_repository::list(connection)
}

/// Used by the schema migration and test fixtures, never by request handlers.
pub(crate) fn import(
    connection: &mut Connection<'_>,
    releases: Vec<Release>,
    patches: Vec<Patch>,
) -> Result<()> {
    for entity in &releases {
        release_repository::save(connection, entity)?;
    }
    for entity in &patches {
        patch_repository::save(connection, entity)?;
    }
    Ok(())
}

pub(crate) fn release(connection: &mut Connection<'_>, app: &str, id: i64) -> Result<Release> {
    let release = release_repository::find(connection, id)?;
    if release.app_id != app {
        return Err(crate::platform::error::missing());
    }
    Ok(release)
}

pub(crate) fn patch(connection: &mut Connection<'_>, app: &str, id: i64) -> Result<Patch> {
    let patch = patch_repository::find(connection, id)?;
    release(connection, app, patch.release_id)?;
    Ok(patch)
}
#[cfg(test)]
pub(crate) use super::dto::Check;

#[cfg(test)]
pub(crate) fn select_for_test(
    runtime: &crate::bootstrap::Runtime,
    connection: &mut Connection<'_>,
    input: &Check,
) -> Result<serde_json::Value> {
    super::service::select_patch(runtime, connection, input)
}

#[cfg(test)]
pub(crate) fn rollback_for_test(
    connection: &mut Connection<'_>,
    app: String,
    release: i64,
    patch: i64,
    value: bool,
) -> Result<serde_json::Value> {
    super::service::set_rollback(connection, app, release, patch, value)
}
