pub(super) mod patch_repository;
pub(super) mod release_repository;

use crate::platform::error::Result;
use rusqlite::Connection;

pub(super) fn initialize(connection: &Connection) -> Result<()> {
    release_repository::initialize(connection)?;
    patch_repository::initialize(connection)?;
    Ok(())
}
