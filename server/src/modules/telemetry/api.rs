use super::{models::Event, repositories::event_repository};
use crate::platform::error::{Result, internal};
use rusqlite::Connection;
use serde_json::Value;

pub(crate) fn initialize(connection: &Connection) -> Result<()> {
    event_repository::initialize(connection)
}

pub(crate) fn events(connection: &Connection) -> Result<Vec<Value>> {
    event_repository::list(connection)?
        .into_iter()
        .map(|event| serde_json::to_value(event).map_err(internal))
        .collect()
}

/// The migration and fixtures import retained events through their owning repository.
pub(crate) fn import(connection: &Connection, events: Vec<Value>) -> Result<()> {
    connection
        .execute("DELETE FROM telemetry_events", [])
        .map_err(internal)?;
    for event in events {
        let event: Event = serde_json::from_value(event).map_err(internal)?;
        event_repository::prune(connection)?;
        event_repository::append(connection, &event)?;
    }
    Ok(())
}
