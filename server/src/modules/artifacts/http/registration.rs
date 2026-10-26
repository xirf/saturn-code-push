use crate::{
    bootstrap::Runtime,
    platform::error::{Result, bad},
};
use axum::extract::Query;
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use serde_json::Value;
use std::collections::HashMap;
pub(crate) async fn list_artifacts(
    State(runtime): State<Runtime>,
    Path((app, id)): Path<(String, i64)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>> {
    let database = runtime.database.clone();
    database
        .call(move |connection| {
            super::super::service::list_artifacts(&runtime, connection, app, id, query)
        })
        .await
        .map(Json)
}

pub(crate) async fn register_release_artifact(
    State(runtime): State<Runtime>,
    Path((app, id)): Path<(String, i64)>,
    multipart: Multipart,
) -> Result<Json<Value>> {
    let metadata = metadata(multipart).await?;
    let database = runtime.database.clone();
    database
        .call(move |connection| {
            super::super::service::register_release_artifact(
                &runtime, connection, app, id, metadata,
            )
        })
        .await
        .map(Json)
}

pub(crate) async fn register_patch_artifact(
    State(runtime): State<Runtime>,
    Path((app, id)): Path<(String, i64)>,
    multipart: Multipart,
) -> Result<Json<Value>> {
    let metadata = metadata(multipart).await?;
    let database = runtime.database.clone();
    database
        .call(move |connection| {
            super::super::service::register_patch_artifact(&runtime, connection, app, id, metadata)
        })
        .await
        .map(Json)
}
async fn metadata(mut multipart: Multipart) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| bad(error.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "file" {
            return Err(bad("Register metadata first; upload to the returned URL"));
        }
        let value = field.text().await.map_err(|error| bad(error.to_string()))?;
        if value.len() > 8192 {
            return Err(bad("Metadata too large"));
        }
        values.insert(name, value);
    }
    Ok(values)
}
