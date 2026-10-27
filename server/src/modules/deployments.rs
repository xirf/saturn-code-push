//! Reported deployment provenance. Store status is explicitly supplied, never inferred from CI.
use crate::sql_params as params;
use crate::{
    bootstrap::Runtime,
    http::policy,
    modules::distribution,
    platform::{
        error::{Result, bad, internal},
        values::now,
    },
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    routing::{get, patch},
};
use postgres::Transaction as Connection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Report {
    key: String,
    repository: String,
    commit: String,
    source_kind: String,
    source_ref: String,
    destination: String,
    status: String,
    #[serde(default)]
    build_number: String,
    #[serde(default)]
    workflow_run: Option<u64>,
}
#[derive(Serialize)]
struct Deployment {
    id: i64,
    release_id: i64,
    #[serde(flatten)]
    report: Report,
    created_at: String,
    updated_at: String,
    reported_by: Option<String>,
    updated_by: Option<String>,
}

pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    policy::management(
        runtime,
        Router::new()
            .route("/api/v1/apps/{app}/deployments", get(list).post(report))
            .route("/api/v1/apps/{app}/deployments/{id}", patch(update)),
    )
}
fn validate(r: &Report) -> Result<()> {
    let parts: Vec<_> = r.repository.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|p| {
            p.is_empty()
                || *p == "."
                || *p == ".."
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
    {
        return Err(bad("Repository must be owner/repository"));
    }
    if !(r.commit.len() == 40 || r.commit.len() == 64)
        || !r.commit.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(bad("Use a full commit SHA"));
    }
    if !["commit", "tag", "release"].contains(&r.source_kind.as_str())
        || ![
            "pending",
            "running",
            "failed",
            "uploaded",
            "in_review",
            "published",
        ]
        .contains(&r.status.as_str())
    {
        return Err(bad("Invalid source kind or deployment status"));
    }
    if [&r.key, &r.source_ref, &r.destination]
        .iter()
        .any(|s| s.trim().is_empty() || s.len() > 256)
        || r.build_number.len() > 256
        || r.repository.len() > 256
        || r.workflow_run == Some(0)
    {
        return Err(bad("Invalid deployment fields"));
    }
    Ok(())
}
fn records(c: &mut Connection<'_>, app: &str) -> Result<Vec<Deployment>> {
    c.query("SELECT id,release_id,body,created_at,updated_at,reported_by,updated_by FROM deployment_reports WHERE app_id=$1 ORDER BY id DESC", &[&app]).map_err(internal)?.iter().map(|r| Ok(Deployment {
        id:r.get(0),release_id:r.get(1),report:serde_json::from_str(&r.get::<_,String>(2)).map_err(internal)?,created_at:r.get(3),updated_at:r.get(4),reported_by:r.get(5),updated_by:r.get(6)
    })).collect()
}
async fn list(State(rt): State<Runtime>, Path(app): Path<String>) -> Result<Json<Vec<Deployment>>> {
    rt.database
        .call(move |c| {
            crate::modules::applications::api::require(c, &app)?;
            records(c, &app)
        })
        .await
        .map(Json)
}
#[derive(Deserialize)]
struct Input {
    release_id: i64,
    #[serde(flatten)]
    report: Report,
}
async fn report(
    State(rt): State<Runtime>,
    Path(app): Path<String>,
    Extension(actor): Extension<crate::modules::identity::api::Actor>,
    Json(input): Json<Input>,
) -> Result<Json<Vec<Deployment>>> {
    validate(&input.report)?;
    rt.database.call(move |c| {
        distribution::api::release(c, &app, input.release_id)?;
        let existing: Option<i64> = c.query_opt("SELECT release_id FROM deployment_reports WHERE app_id=$1 AND report_key=$2", params![app, input.report.key]).map_err(internal)?.map(|r|r.get(0));
        if existing.is_some_and(|id| id != input.release_id) { return Err(bad("Deployment key belongs to another release")); }
        let timestamp = now();
        c.execute("INSERT INTO deployment_reports(app_id, release_id, report_key, body, created_at, updated_at,reported_by,updated_by) VALUES($1,$2,$3,$4,$5,$5,$6,$6) ON CONFLICT(app_id,report_key) DO UPDATE SET body=excluded.body, updated_at=excluded.updated_at,updated_by=excluded.updated_by", params![app, input.release_id, input.report.key, serde_json::to_string(&input.report).map_err(internal)?, timestamp, format!("{}: {}",actor.kind,actor.name)]).map_err(internal)?;
        records(c, &app)
    }).await.map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusUpdate {
    status: String,
}
async fn update(
    State(rt): State<Runtime>,
    Path((app, id)): Path<(String, i64)>,
    Extension(actor): Extension<crate::modules::identity::api::Actor>,
    Json(input): Json<StatusUpdate>,
) -> Result<Json<Vec<Deployment>>> {
    rt.database
        .call(move |c| {
            let mut record = records(c, &app)?
                .into_iter()
                .find(|r| r.id == id)
                .ok_or_else(crate::platform::error::missing)?;
            record.report.status = input.status;
            validate(&record.report)?;
            c.execute(
                "UPDATE deployment_reports SET body=$1, updated_at=$2,updated_by=$5 WHERE id=$3 AND app_id=$4",
                params![
                    serde_json::to_string(&record.report).map_err(internal)?,
                    now(),
                    id,
                    app,
                    format!("{}: {}",actor.kind,actor.name)
                ],
            )
            .map_err(internal)?;
            records(c, &app)
        })
        .await
        .map(Json)
}
