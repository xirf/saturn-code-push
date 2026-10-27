//! Offline, read-only SQLite import. Object IDs and file paths stay unchanged.
use crate::{
    modules::{applications, artifacts, distribution, telemetry},
    platform::{
        database::{Connection, Database},
        error::{Result, bad, internal},
    },
};
use rusqlite::{Connection as Sqlite, OpenFlags, types::ValueRef};
use serde_json::{Map, Value};
use std::path::Path;

const TABLES: &[&str] = &[
    "applications_apps",
    "applications_channels",
    "distribution_releases",
    "distribution_patches",
    "distribution_patch_assets",
    "artifacts_binaries",
    "artifacts_bundles",
    "artifact_asset_signatures",
    "telemetry_events",
    "deployment_reports",
];
fn exists(c: &Sqlite, name: &str) -> Result<bool> {
    c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [name],
        |r| r.get(0),
    )
    .map_err(internal)
}
fn rows(c: &Sqlite, table: &str) -> Result<Vec<Map<String, Value>>> {
    if !exists(c, table)? {
        return Ok(vec![]);
    }
    let mut statement = c
        .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
        .map_err(internal)?;
    let names = statement
        .column_names()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let result = statement
        .query_map([], |r| {
            let mut map = Map::new();
            for (i, name) in names.iter().enumerate() {
                let value = match r.get_ref(i)? {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(n) => serde_json::json!(n),
                    ValueRef::Real(n) => serde_json::json!(n),
                    ValueRef::Text(v) => Value::String(String::from_utf8_lossy(v).into_owned()),
                    ValueRef::Blob(_) => return Err(rusqlite::Error::InvalidQuery),
                };
                map.insert(name.clone(), value);
            }
            Ok(map)
        })
        .map_err(internal)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(internal)?;
    Ok(result)
}
fn entity<T: serde::de::DeserializeOwned>(mut row: Map<String, Value>) -> Result<T> {
    for key in ["platforms", "channel_ids", "signing", "event"] {
        if let Some(Value::String(text)) = row.get(key) {
            let value = serde_json::from_str(text).map_err(internal)?;
            row.insert(key.into(), value);
        }
    }
    for key in ["is_rolled_back", "uploaded", "can_sideload"] {
        if let Some(value) = row.get(key).and_then(Value::as_i64) {
            row.insert(key.into(), Value::Bool(value != 0));
        }
    }
    serde_json::from_value(Value::Object(row)).map_err(internal)
}
fn text(row: &Map<String, Value>, key: &str) -> Result<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| bad(format!("SQLite import: missing {key}")))
}
fn number(row: &Map<String, Value>, key: &str) -> Result<i64> {
    row.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| bad(format!("SQLite import: missing {key}")))
}

pub(crate) fn import(database: &Database, path: &Path) -> Result<()> {
    let source =
        Sqlite::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(internal)?;
    source.execute_batch("BEGIN").map_err(internal)?;
    let version: i64 = source
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(internal)?;
    if !(1..=3).contains(&version) {
        return Err(bad(
            "Import requires a migrated SQLite schema version 1–3. Upgrade the SQLite server first.",
        ));
    }
    let data = TABLES
        .iter()
        .map(|table| Ok((*table, rows(&source, table)?)))
        .collect::<Result<std::collections::HashMap<_, _>>>()?;
    let sequence: i64 = source
        .query_row("SELECT value FROM saturn_sequence WHERE id=1", [], |r| {
            r.get(0)
        })
        .map_err(internal)?;
    database.transaction(|c| {
        for table in TABLES {
            let count:i64=c.query_one(&format!("SELECT COUNT(*) FROM {table}"),&[]).map_err(internal)?.get(0);
            if count>0 {return Err(bad("Import requires an empty PostgreSQL application catalog"))}
        }
        applications::api::import(c,data["applications_apps"].iter().cloned().map(entity).collect::<Result<_>>()?,data["applications_channels"].iter().cloned().map(entity).collect::<Result<_>>()?)?;
        distribution::api::import(c,data["distribution_releases"].iter().cloned().map(entity).collect::<Result<_>>()?,data["distribution_patches"].iter().cloned().map(entity).collect::<Result<_>>()?)?;
        artifacts::api::import(c,data["artifacts_binaries"].iter().cloned().map(entity).collect::<Result<_>>()?,data["artifacts_bundles"].iter().cloned().map(entity).collect::<Result<_>>()?)?;
        for r in &data["distribution_patch_assets"] {c.execute("INSERT INTO distribution_patch_assets VALUES($1,$2)",&[&number(r,"patch_id")?,&text(r,"platform")?]).map_err(internal)?;}
        for r in &data["artifact_asset_signatures"] {c.execute("INSERT INTO artifact_asset_signatures VALUES($1,$2)",&[&number(r,"bundle_id")?,&text(r,"signature")?]).map_err(internal)?;}
        telemetry::api::import(c,data["telemetry_events"].iter().cloned().map(entity::<Value>).collect::<Result<_>>()?)?;
        for r in &data["deployment_reports"] {
            c.execute("INSERT INTO deployment_reports(id,app_id,release_id,report_key,body,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7)",&[&number(r,"id")?,&text(r,"app_id")?,&number(r,"release_id")?,&text(r,"report_key")?,&text(r,"body")?,&text(r,"created_at")?,&text(r,"updated_at")?]).map_err(internal)?;
        }
        let max_id=data.iter().filter(|(t,_)| !["applications_apps","telemetry_events","deployment_reports"].contains(t)).flat_map(|(_,rows)|rows.iter()).filter_map(|r|r.get("id").and_then(Value::as_i64)).max().unwrap_or(0);
        c.execute("UPDATE saturn_sequence SET value=$1 WHERE id=1", &[&sequence.max(max_id)]).map_err(internal)?;
        reset_sequence(c,"deployment_reports")?;
        c.execute("INSERT INTO saturn_imports(source) VALUES($1)", &[&path.to_string_lossy().as_ref()]).map_err(internal)?;
        // Sessions deliberately expire: imported operator sessions cannot identify new accounts.
        Ok(())
    })
}
fn reset_sequence(c: &mut Connection<'_>, table: &str) -> Result<()> {
    c.query_one(&format!("SELECT setval(pg_get_serial_sequence('{table}','id'), GREATEST(COALESCE(MAX(id),0),1), COUNT(*)>0) FROM {table}"),&[]).map_err(internal)?;
    Ok(())
}
