use super::super::models::Event;
use crate::platform::error::{Result, bad, internal};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::telemetry) const RETENTION_LIMIT: usize = 10_000;

pub(in crate::modules::telemetry) fn list(connection: &mut Connection<'_>) -> Result<Vec<Event>> {
    connection
        .query(
            "SELECT received_at, event FROM telemetry_events ORDER BY id",
            &[],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(Event {
                received_at: row.try_get(0).map_err(internal)?,
                event: serde_json::from_str(&row.try_get::<_, String>(1).map_err(internal)?)
                    .map_err(internal)?,
            })
        })
        .collect()
}

pub(in crate::modules::telemetry) fn append(
    connection: &mut Connection<'_>,
    event: &Event,
) -> Result<()> {
    let app_id = event.event["app_id"]
        .as_str()
        .ok_or_else(|| bad("Event requires app_id"))?;
    connection
        .execute(
            "INSERT INTO telemetry_events(app_id, received_at, event) VALUES ($1, $2, $3)",
            params![
                app_id,
                event.received_at,
                serde_json::to_string(&event.event).map_err(internal)?
            ],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::telemetry) fn prune(connection: &mut Connection<'_>) -> Result<()> {
    let count: i64 = connection
        .query_one("SELECT COUNT(*) FROM telemetry_events", &[])
        .map_err(internal)?
        .try_get(0)
        .map_err(internal)?;
    if count >= RETENTION_LIMIT as i64 {
        connection
            .execute(
                "DELETE FROM telemetry_events WHERE id IN (
                SELECT id FROM telemetry_events ORDER BY id LIMIT $1
            )",
                &[&(count - 9_000)],
            )
            .map_err(internal)?;
    }
    Ok(())
}

pub(in crate::modules::telemetry) fn contains_id(
    connection: &mut Connection<'_>,
    app: &str,
    id: &str,
) -> Result<bool> {
    connection.query_one("SELECT EXISTS(SELECT 1 FROM telemetry_events WHERE app_id=$1 AND (event::jsonb ->> 'event_id')=$2)", params![app,id]).map_err(internal)?.try_get(0).map_err(internal)
}

/// The app index bounds reads before deserializing event bodies.
pub(in crate::modules::telemetry) fn list_app(
    connection: &mut Connection<'_>,
    app: &str,
) -> Result<Vec<Event>> {
    connection
        .query(
            "SELECT received_at, event FROM telemetry_events WHERE app_id=$1 ORDER BY id",
            &[&app],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(Event {
                received_at: row.try_get(0).map_err(internal)?,
                event: serde_json::from_str(&row.try_get::<_, String>(1).map_err(internal)?)
                    .map_err(internal)?,
            })
        })
        .collect()
}
