use super::super::models::Event;
use crate::platform::error::{Result, bad, internal};
use rusqlite::{Connection, params};

pub(in crate::modules::telemetry) const RETENTION_LIMIT: usize = 10_000;

pub(in crate::modules::telemetry) fn initialize(connection: &Connection) -> Result<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS telemetry_events (
            id INTEGER PRIMARY KEY,
            app_id TEXT NOT NULL REFERENCES applications_apps(id),
            received_at TEXT NOT NULL,
            event TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS telemetry_events_app ON telemetry_events(app_id);",
        )
        .map_err(internal)
}

pub(in crate::modules::telemetry) fn list(connection: &Connection) -> Result<Vec<Event>> {
    let mut statement = connection
        .prepare("SELECT received_at, event FROM telemetry_events ORDER BY id")
        .map_err(internal)?;
    let rows = statement
        .query_map([], |row| {
            let event: String = row.get(1)?;
            let event = serde_json::from_str(&event).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            Ok(Event {
                received_at: row.get(0)?,
                event,
            })
        })
        .map_err(internal)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)
}

pub(in crate::modules::telemetry) fn append(connection: &Connection, event: &Event) -> Result<()> {
    let app_id = event.event["app_id"]
        .as_str()
        .ok_or_else(|| bad("Event requires app_id"))?;
    connection
        .execute(
            "INSERT INTO telemetry_events(app_id, received_at, event) VALUES (?1, ?2, ?3)",
            params![
                app_id,
                event.received_at,
                serde_json::to_string(&event.event).map_err(internal)?
            ],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::telemetry) fn prune(connection: &Connection) -> Result<()> {
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM telemetry_events", [], |row| {
            row.get(0)
        })
        .map_err(internal)?;
    if count >= RETENTION_LIMIT as i64 {
        connection
            .execute(
                "DELETE FROM telemetry_events WHERE id IN (
                SELECT id FROM telemetry_events ORDER BY id LIMIT ?1
            )",
                [count - 9_000],
            )
            .map_err(internal)?;
    }
    Ok(())
}
