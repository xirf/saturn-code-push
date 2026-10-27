use super::{App, repository};
use crate::{
    bootstrap::Runtime,
    http::policy,
    platform::error::{ApiError, Result, bad, internal},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    policy::management(
        runtime,
        Router::new()
            .route("/admin/integrations/github", get(settings).post(connect))
            .route(
                "/admin/integrations/github/{app}/disconnect",
                post(disconnect),
            )
            .route("/admin/integrations/github/sync", post(sync)),
    )
    .route(
        "/integrations/github/webhook",
        post(webhook).layer(DefaultBodyLimit::max(1024 * 1024)),
    )
}
fn app(runtime: &Runtime) -> Result<Arc<App>> {
    runtime.github.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "GitHub App is not configured in deployment settings".into(),
        )
    })
}
async fn settings(State(runtime): State<Runtime>) -> Result<Json<Value>> {
    let connections = runtime.database.call_read(repository::connections).await?;
    Ok(Json(
        json!({"configured":runtime.github.is_some(),"installation_url":runtime.github.as_ref().map(|app|format!("https://github.com/apps/{}/installations/new",app.slug)),"webhook_url":format!("{}/integrations/github/webhook",runtime.base_url),"connections":connections}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Connect {
    app_id: String,
    repository: String,
    installation_id: i64,
}
async fn connect(
    State(runtime): State<Runtime>,
    Json(input): Json<Connect>,
) -> Result<Json<Value>> {
    let app = app(&runtime)?;
    app.verify_installation(input.installation_id, &input.repository)
        .await?;
    runtime
        .database
        .call(move |c| {
            repository::connect(
                c,
                &input.app_id,
                &input.repository.to_ascii_lowercase(),
                input.installation_id,
            )
        })
        .await?;
    app.notify.notify_one();
    Ok(Json(json!({"connected":true})))
}
async fn disconnect(State(runtime): State<Runtime>, Path(id): Path<String>) -> Result<Json<Value>> {
    runtime
        .database
        .call(move |c| repository::disconnect(c, &id))
        .await?;
    Ok(Json(json!({"disconnected":true})))
}
async fn sync(State(runtime): State<Runtime>) -> Result<Json<Value>> {
    let app = app(&runtime)?;
    runtime
        .database
        .call(|c| {
            c.execute(
                "UPDATE github_workflow_runs SET checked_at=to_timestamp(0),refresh_version=refresh_version+1",
                &[],
            )
            .map_err(internal)?;
            Ok(())
        })
        .await?;
    app.notify.notify_one();
    Ok(Json(json!({"queued":true})))
}
async fn webhook(
    State(runtime): State<Runtime>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode> {
    let app = app(&runtime)?;
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !app.verify_signature(signature, &body) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Invalid GitHub webhook signature".into(),
        ));
    }
    let event = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let delivery = headers
        .get("x-github-delivery")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| uuid::Uuid::parse_str(v).ok())
        .ok_or_else(|| bad("Invalid GitHub delivery ID"))?
        .to_string();
    if ![
        "workflow_run",
        "installation",
        "installation_repositories",
        "ping",
    ]
    .contains(&event.as_str())
    {
        return Ok(StatusCode::NO_CONTENT);
    }
    let payload: Value = serde_json::from_slice(&body).map_err(|_| bad("Invalid webhook JSON"))?;
    let installation = payload["installation"]["id"].as_i64().unwrap_or(0);
    let repository = payload["repository"]["full_name"]
        .as_str()
        .unwrap_or("")
        .to_ascii_lowercase();
    let run = payload["workflow_run"]["id"]
        .as_i64()
        .unwrap_or(0)
        .to_string();
    runtime.database.call(move |c| {
        c.execute("DELETE FROM github_deliveries WHERE received_at<now()-interval '7 days'",&[]).map_err(internal)?;
        if c.execute("INSERT INTO github_deliveries(id) VALUES($1) ON CONFLICT DO NOTHING",&[&delivery]).map_err(internal)?==0 {return Ok(())}
        // Treat events as refresh hints. Fetch authoritative state from GitHub;
        // webhook replay and out-of-order delivery must never publish an app.
        c.execute("UPDATE github_workflow_runs w SET checked_at=to_timestamp(0),refresh_version=w.refresh_version+1 FROM deployment_reports d JOIN github_integrations i ON i.app_id=d.app_id WHERE w.deployment_id=d.id AND i.installation_id=$1 AND ($2<>'workflow_run' OR (lower(d.body::jsonb->>'repository')=$3 AND d.body::jsonb->>'workflow_run'=$4))",&[&installation,&event,&repository,&run]).map_err(internal)?;Ok(())
    }).await?;
    app.notify.notify_one();
    Ok(StatusCode::NO_CONTENT)
}
