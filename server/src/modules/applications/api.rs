pub(crate) use super::models::{App, Channel};
use super::repositories::{app_repository, channel_repository};
use crate::platform::error::Result;
use rusqlite::Connection;

pub(crate) fn initialize(connection: &Connection) -> Result<()> {
    super::repositories::initialize(connection)
}

pub(crate) fn apps(connection: &Connection) -> Result<Vec<App>> {
    app_repository::list(connection)
}

pub(crate) fn channels(connection: &Connection) -> Result<Vec<Channel>> {
    channel_repository::list(connection)
}

/// Used by the schema migration and test fixtures, never by request handlers.
pub(crate) fn import(
    connection: &Connection,
    apps: Vec<App>,
    channels: Vec<Channel>,
) -> Result<()> {
    for entity in &apps {
        app_repository::save(connection, entity)?;
    }
    for entity in &channels {
        channel_repository::save(connection, entity)?;
    }
    Ok(())
}

pub(crate) fn require(connection: &Connection, id: &str) -> Result<()> {
    app_repository::find(connection, id).map(|_| ())
}
