use super::super::models::App;
use crate::platform::error::{Result, internal};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::applications) fn list(connection: &mut Connection<'_>) -> Result<Vec<App>> {
    connection
        .query(
            "SELECT
                id,
                display_name,
                created_at
         FROM applications_apps ORDER BY id",
            &[],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(App {
                id: row.try_get(0).map_err(internal)?,
                display_name: row.try_get(1).map_err(internal)?,
                created_at: row.try_get(2).map_err(internal)?,
            })
        })
        .collect()
}

pub(in crate::modules::applications) fn save(
    connection: &mut Connection<'_>,
    entity: &App,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO applications_apps (
                id,
                display_name,
                created_at
            ) VALUES ($1, $2, $3)
            ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                created_at = excluded.created_at",
            params![entity.id, entity.display_name, entity.created_at,],
        )
        .map_err(internal)?;
    Ok(())
}

pub(in crate::modules::applications) fn find(
    connection: &mut Connection<'_>,
    id: &str,
) -> Result<App> {
    list(connection)?
        .into_iter()
        .find(|entity| entity.id == id)
        .ok_or_else(crate::platform::error::missing)
}
