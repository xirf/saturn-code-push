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
    #[serde(default)]
    workflow_attempt: Option<u64>,
}
#[derive(Serialize)]
struct Deployment {
    id: i64,
    release_id: i64,
    patch_id: Option<i64>,
    #[serde(flatten)]
    report: Report,
    created_at: String,
    updated_at: String,
    reported_by: Option<String>,
    updated_by: Option<String>,
    workflow: Option<serde_json::Value>,
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
        || r.workflow_attempt
            .is_some_and(|id| id == 0 || id > i64::MAX as u64 || r.workflow_run.is_none())
        || r.workflow_run
            .is_some_and(|id| id == 0 || id > i64::MAX as u64)
    {
        return Err(bad("Invalid deployment fields"));
    }
    Ok(())
}
fn records(c: &mut Connection<'_>, app: &str) -> Result<Vec<Deployment>> {
    c.query("SELECT d.id,d.release_id,d.body,d.created_at,d.updated_at,d.reported_by,d.updated_by,d.patch_id,
        CASE WHEN w.deployment_id IS NULL THEN NULL ELSE json_build_object('run_id',w.run_id,'run_attempt',w.run_attempt,'status',w.status,'conclusion',w.conclusion,'checked_at',w.checked_at::text,'sync_error',w.sync_error)::text END
        FROM deployment_reports d LEFT JOIN github_workflow_runs w ON w.deployment_id=d.id WHERE d.app_id=$1 ORDER BY d.id DESC", &[&app]).map_err(internal)?.iter().map(|r| Ok(Deployment {
        id:r.get(0), release_id:r.get(1), report:serde_json::from_str(&r.get::<_,String>(2)).map_err(internal)?,
        created_at:r.get(3),updated_at:r.get(4),reported_by:r.get(5),updated_by:r.get(6),patch_id:r.get(7),
        workflow:r.get::<_,Option<String>>(8).map(|s|serde_json::from_str(&s)).transpose().map_err(internal)?,
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
    #[serde(default)]
    patch_id: Option<i64>,
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
        if let Some(id)=input.patch_id
            && distribution::api::patch(c,&app,id)?.release_id != input.release_id {
            return Err(bad("Patch does not belong to this release"));
        }
        let existing=c.query_opt("SELECT release_id,patch_id,body FROM deployment_reports WHERE app_id=$1 AND report_key=$2", params![app,input.report.key]).map_err(internal)?;
        if let Some(row)=existing {
            if row.get::<_,i64>(0)!=input.release_id || row.get::<_,Option<i64>>(1)!=input.patch_id {return Err(bad("Deployment key belongs to another release or patch"));}
            let old:Report=serde_json::from_str(&row.get::<_,String>(2)).map_err(internal)?;
            if old.repository!=input.report.repository || old.commit!=input.report.commit || old.workflow_run!=input.report.workflow_run || old.workflow_attempt!=input.report.workflow_attempt {
                c.execute("DELETE FROM github_workflow_runs WHERE deployment_id=(SELECT id FROM deployment_reports WHERE app_id=$1 AND report_key=$2)",params![app,input.report.key]).map_err(internal)?;
            }
        }
        let timestamp = now();
        c.execute("INSERT INTO deployment_reports(app_id, release_id, report_key, body, created_at, updated_at,reported_by,updated_by,patch_id) VALUES($1,$2,$3,$4,$5,$5,$6,$6,$7) ON CONFLICT(app_id,report_key) DO UPDATE SET body=excluded.body, updated_at=excluded.updated_at,updated_by=excluded.updated_by", params![app, input.release_id, input.report.key, serde_json::to_string(&input.report).map_err(internal)?, timestamp, format!("{}: {}",actor.kind,actor.name),input.patch_id]).map_err(internal)?;
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
