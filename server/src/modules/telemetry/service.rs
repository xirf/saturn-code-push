use super::{models::Event, repositories::event_repository};
use crate::modules::applications;
use crate::platform::{
    error::{Result, bad},
    values::{now, text},
};
use postgres::Transaction as Connection;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(super) fn event(connection: &mut Connection<'_>, input: Value) -> Result<Value> {
    let e = input
        .get("event")
        .filter(|e| e.is_object())
        .cloned()
        .ok_or_else(|| bad("Expected event"))?;
    let app = text(&e, "app_id")?;
    let kind = text(&e, "type")?;
    if ![
        "__patch_download__",
        "__patch_install__",
        "__patch_first_launch__",
        "__patch_health__",
        "__patch_install_failure__",
        "__patch_update_failure__",
    ]
    .contains(&kind.as_str())
    {
        return Err(bad("Unknown event type"));
    }
    applications::api::require(connection, &app)?;
    if let Some(id) = e.get("event_id") {
        let id = id
            .as_str()
            .filter(|id| !id.is_empty() && id.len() <= 128)
            .ok_or_else(|| bad("event_id must contain 1 to 128 bytes"))?;
        if event_repository::contains_id(connection, &app, id)? {
            return Ok(json!({}));
        }
    }
    let event = Event {
        received_at: now(),
        event: e,
    };
    event_repository::prune(connection)?;
    event_repository::append(connection, &event)?;
    Ok(json!({}))
}

pub(super) fn analytics(
    connection: &mut Connection<'_>,
    query: HashMap<String, String>,
) -> Result<Value> {
    let events = if let Some(app) = query.get("app_id") {
        event_repository::list_app(connection, app)?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(crate::platform::error::internal)?
    } else {
        super::api::events(connection)?
    };
    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut daily: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut clients = HashSet::new();
    let mut patches: std::collections::BTreeMap<String, HashMap<String, u64>> =
        std::collections::BTreeMap::new();
    let mut total = 0;
    for record in &events {
        let e = &record["event"];
        if query
            .get("app_id")
            .is_some_and(|id| e["app_id"].as_str() != Some(id))
        {
            continue;
        }
        total += 1;
        let kind = e["type"].as_str().unwrap_or("unknown").to_owned();
        *counts.entry(kind.clone()).or_default() += 1;
        if let Some(client) = e["client_id"].as_str().filter(|input| !input.is_empty()) {
            clients.insert((e["app_id"].as_str().unwrap_or(""), client));
        }
        let day = record["received_at"]
            .as_str()
            .unwrap_or("")
            .split('T')
            .next()
            .unwrap_or("")
            .to_owned();
        *daily.entry(day).or_default() += 1;
        let key = format!(
            "{} / {} / patch {}",
            e["app_id"].as_str().unwrap_or(""),
            e["release_version"].as_str().unwrap_or(""),
            e["patch_number"]
        );
        *patches.entry(key).or_default().entry(kind).or_default() += 1;
    }
    Ok(json!({
        "event_counts": counts,
        "unique_reporting_installs": clients.len(),
        "daily_events": daily,
        "patch_events": patches,
        "retained_events": total,
        "retention_limit": event_repository::RETENTION_LIMIT,
        "measurement": "Client-reported events in the retained window; event_id retries are deduplicated per app within this window. Legacy events without IDs count separately. Download, install, first launch and health are distinct reports, not verified installations.",
    }))
}
