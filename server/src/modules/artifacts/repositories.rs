pub(super) mod artifact_repository;
pub(super) mod asset_bundle_repository;

use crate::platform::error::Result;
use rusqlite::Connection;

pub(super) fn initialize(connection: &Connection) -> Result<()> {
    artifact_repository::initialize(connection)?;
    asset_bundle_repository::initialize(connection)?;
    Ok(())
}
