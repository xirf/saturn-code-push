use super::super::models::Channel;
use crate::platform::error::{Result, internal};
use crate::sql_params as params;
use postgres::Transaction as Connection;

pub(in crate::modules::applications) fn list(
    connection: &mut Connection<'_>,
) -> Result<Vec<Channel>> {
    connection
        .query(
            "SELECT
                id,
                app_id,
                name
         FROM applications_channels ORDER BY id",
            &[],
        )
        .map_err(internal)?
        .iter()
        .map(|row| {
            Ok(Channel {
                id: row.try_get(0).map_err(internal)?,
                app_id: row.try_get(1).map_err(internal)?,
                name: row.try_get(2).map_err(internal)?,
            })
        })
        .collect()
}

pub(in crate::modules::applications) fn save(
    connection: &mut Connection<'_>,
    entity: &Channel,
) -> Result<()> {
    connection
        .execute(
            "INSERT INTO applications_channels (
                id,
                app_id,
                name
            ) VALUES ($1, $2, $3)
            ON CONFLICT(id) DO UPDATE SET
                app_id = excluded.app_id,
                name = excluded.name",
            params![entity.id, entity.app_id, entity.name,],
        )
        .map_err(internal)?;
    Ok(())
}
