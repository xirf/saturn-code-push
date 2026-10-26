pub(super) mod app_repository;
pub(super) mod channel_repository;

use crate::platform::error::Result;
use rusqlite::Connection;

pub(super) fn initialize(connection: &Connection) -> Result<()> {
    app_repository::initialize(connection)?;
    channel_repository::initialize(connection)?;
    Ok(())
}
