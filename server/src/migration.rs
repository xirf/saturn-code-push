//! One-way, transactional import of the original catalog. Artifact paths and session rows are preserved.
use crate::{
    modules::{applications, artifacts, distribution, identity, telemetry},
    platform::error::{Result, internal},
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use serde::Deserialize;
use serde_json::Value;

#[derive(Default, Deserialize)]
struct LegacyCatalog {
    sequence: i64,
    apps: Vec<applications::api::App>,
    channels: Vec<applications::api::Channel>,
    releases: Vec<distribution::api::Release>,
    patches: Vec<distribution::api::Patch>,
    artifacts: Vec<artifacts::api::Artifact>,
    events: Vec<Value>,
    #[serde(default)]
    asset_bundles: Vec<artifacts::api::AssetBundle>,
}

pub(crate) fn run(connection: &mut Connection) -> Result<()> {
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(internal)?;
    let version: i64 = tx
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(internal)?;
    if version > 1 {
        return Err(internal("Database schema is newer than this server"));
    }
    if version == 1 {
        return Ok(());
    }
    applications::api::initialize(&tx)?;
    distribution::api::initialize(&tx)?;
    artifacts::api::initialize(&tx)?;
    identity::api::initialize(&tx)?;
    telemetry::api::initialize(&tx)?;
    tx.execute_batch(
        "CREATE TABLE saturn_sequence (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            value INTEGER NOT NULL
        );
        INSERT INTO saturn_sequence VALUES(1, 0);",
    )
    .map_err(internal)?;
    let legacy_exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='catalog')",
            [],
            |row| row.get(0),
        )
        .map_err(internal)?;
    if legacy_exists {
        let body: Option<String> = tx
            .query_row("SELECT body FROM catalog WHERE id=1", [], |row| row.get(0))
            .optional()
            .map_err(internal)?;
        if let Some(body) = body {
            let legacy: LegacyCatalog = serde_json::from_str(&body).map_err(internal)?;
            // IDs were allocated from one sequence. Keep it above every imported ID.
            let max_id = legacy
                .channels
                .iter()
                .map(|x| x.id)
                .chain(legacy.releases.iter().map(|x| x.id))
                .chain(legacy.patches.iter().map(|x| x.id))
                .chain(legacy.artifacts.iter().map(|x| x.id))
                .chain(legacy.asset_bundles.iter().map(|x| x.id))
                .max()
                .unwrap_or(0)
                .max(legacy.sequence);
            applications::api::import(&tx, legacy.apps, legacy.channels)?;
            distribution::api::import(&tx, legacy.releases, legacy.patches)?;
            artifacts::api::import(&tx, legacy.artifacts, legacy.asset_bundles)?;
            telemetry::api::import(&tx, legacy.events)?;
            tx.execute("UPDATE saturn_sequence SET value=?1 WHERE id=1", [max_id])
                .map_err(internal)?;
        }
        tx.execute_batch("ALTER TABLE catalog RENAME TO legacy_catalog_v0;")
            .map_err(internal)?;
    }
    // The original binary creates `catalog` if absent, then deserializes its body.
    // A view with an incompatible body makes that binary fail at startup, before
    // it can allocate conflicting IDs or overwrite objects in the migrated store.
    tx.execute_batch(
        "CREATE VIEW catalog AS
            SELECT 1 AS id,
                'Schema migrated; use the current Saturn server or restore a pre-migration backup' AS body;",
    ).map_err(internal)?;
    tx.pragma_update(None, "user_version", 1)
        .map_err(internal)?;
    tx.commit().map_err(internal)
}
