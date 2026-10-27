use crate::platform::{
    database::Connection,
    error::{Result, internal},
    values::now,
};
use serde_json::{Value, json};

#[derive(Clone)]
pub(super) struct Candidate {
    pub id: i64,
    pub app: String,
    pub repository: String,
    pub installation: i64,
    pub run: i64,
    pub attempt: Option<i64>,
    pub commit: String,
    pub refresh_version: i64,
}
pub(super) fn connections(c: &mut Connection<'_>) -> Result<Vec<Value>> {
    Ok(c.query("SELECT app_id,repository,installation_id,connected_at FROM github_integrations ORDER BY app_id",&[]).map_err(internal)?.iter().map(|r|json!({"app_id":r.get::<_,String>(0),"repository":r.get::<_,String>(1),"installation_id":r.get::<_,i64>(2),"connected_at":r.get::<_,String>(3)})).collect())
}
pub(super) fn disconnect(c: &mut Connection<'_>, app: &str) -> Result<()> {
    c.execute("DELETE FROM github_workflow_runs WHERE deployment_id IN(SELECT id FROM deployment_reports WHERE app_id=$1)",&[&app]).map_err(internal)?;
    c.execute("DELETE FROM github_integrations WHERE app_id=$1", &[&app])
        .map_err(internal)?;
    Ok(())
}
pub(super) fn connect(
    c: &mut Connection<'_>,
    app: &str,
    repo: &str,
    installation: i64,
) -> Result<()> {
    crate::modules::applications::api::require(c, app)?;
    disconnect(c, app)?;
    c.execute("INSERT INTO github_integrations(app_id,repository,installation_id,connected_at) VALUES($1,$2,$3,$4)",&[&app,&repo,&installation,&now()]).map_err(internal)?;
    Ok(())
}
pub(super) fn candidates(c: &mut Connection<'_>) -> Result<Vec<Candidate>> {
    let rows=c.query("SELECT d.id,d.app_id,i.repository,i.installation_id,d.body FROM deployment_reports d JOIN github_integrations i ON i.app_id=d.app_id AND lower(i.repository)=lower(d.body::jsonb->>'repository') LEFT JOIN github_workflow_runs w ON w.deployment_id=d.id WHERE d.body::jsonb->>'workflow_run' IS NOT NULL AND (w.checked_at IS NULL OR w.checked_at < now() - CASE WHEN w.status='completed' AND w.sync_error IS NULL THEN interval '1 hour' ELSE interval '60 seconds' END) ORDER BY w.checked_at ASC NULLS FIRST,d.id ASC LIMIT 100",&[]).map_err(internal)?;
    let mut result = vec![];
    for row in rows {
        let body: Value = serde_json::from_str(&row.get::<_, String>(4)).map_err(internal)?;
        if let (Some(run), Some(commit)) = (
            body["workflow_run"].as_i64().filter(|v| *v > 0),
            body["commit"].as_str(),
        ) {
            c.execute("INSERT INTO github_workflow_runs(deployment_id,run_id,checked_at) VALUES($1,$2,to_timestamp(0)) ON CONFLICT DO NOTHING",&[&row.get::<_,i64>(0),&run]).map_err(internal)?;
            let refresh_version = c
                .query_one(
                    "SELECT refresh_version FROM github_workflow_runs WHERE deployment_id=$1",
                    &[&row.get::<_, i64>(0)],
                )
                .map_err(internal)?
                .get(0);
            result.push(Candidate {
                id: row.get(0),
                app: row.get(1),
                repository: row.get(2),
                installation: row.get(3),
                run,
                attempt: body["workflow_attempt"].as_i64(),
                commit: commit.into(),
                refresh_version,
            });
        }
    }
    Ok(result)
}
fn still_linked(c: &mut Connection<'_>, item: &Candidate) -> Result<bool> {
    Ok(c.query_one("SELECT EXISTS(SELECT 1 FROM deployment_reports d JOIN github_integrations i ON i.app_id=d.app_id WHERE d.id=$1 AND i.app_id=$2 AND i.repository=$3 AND i.installation_id=$4 AND d.body::jsonb->>'commit'=$5 AND d.body::jsonb->>'workflow_run'=$6 AND lower(d.body::jsonb->>'repository')=lower($3) AND COALESCE(d.body::jsonb->>'workflow_attempt','')=$7)",&[&item.id,&item.app,&item.repository,&item.installation,&item.commit,&item.run.to_string(),&item.attempt.map(|n|n.to_string()).unwrap_or_default()]).map_err(internal)?.get(0))
}
pub(super) fn save(c: &mut Connection<'_>, item: &Candidate, run: &Value) -> Result<()> {
    if !still_linked(c, item)? {
        return Ok(());
    }
    let Some(attempt) = run["run_attempt"].as_i64().filter(|v| *v > 0) else {
        return error(c, item, "GitHub returned an invalid run attempt");
    };
    if item.attempt.is_some_and(|expected| expected != attempt) {
        return error(c, item, "Workflow attempt does not match this deployment");
    }
    let Some(status) = run["status"].as_str().filter(|v| {
        [
            "queued",
            "in_progress",
            "completed",
            "waiting",
            "requested",
            "pending",
        ]
        .contains(v)
    }) else {
        return error(c, item, "GitHub returned an unknown workflow status");
    };
    let updated = run["updated_at"]
        .as_str()
        .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
        .map(|t| t.to_rfc3339());
    let Some(updated) = updated else {
        return error(c, item, "GitHub returned an invalid workflow timestamp");
    };
    if run["id"].as_i64() != Some(item.run)
        || !run["head_sha"]
            .as_str()
            .is_some_and(|v| v.eq_ignore_ascii_case(&item.commit))
        || !run["repository"]["full_name"]
            .as_str()
            .is_some_and(|v| v.eq_ignore_ascii_case(&item.repository))
    {
        return error(
            c,
            item,
            "Workflow repository or commit does not match this deployment",
        );
    }
    let conclusion = run["conclusion"].as_str().filter(|v| v.len() <= 64);
    let changed=c.execute("INSERT INTO github_workflow_runs(deployment_id,run_id,run_attempt,status,conclusion,github_updated_at,checked_at) VALUES($1,$2,$3,$4,$5,$6::text::timestamptz,now()) ON CONFLICT(deployment_id) DO UPDATE SET run_id=excluded.run_id,run_attempt=excluded.run_attempt,status=excluded.status,conclusion=excluded.conclusion,github_updated_at=excluded.github_updated_at,checked_at=CASE WHEN github_workflow_runs.refresh_version=$7 THEN now() ELSE github_workflow_runs.checked_at END,sync_error=NULL WHERE github_workflow_runs.run_attempt<excluded.run_attempt OR (github_workflow_runs.run_attempt=excluded.run_attempt AND (github_workflow_runs.github_updated_at IS NULL OR github_workflow_runs.github_updated_at<=excluded.github_updated_at) AND (github_workflow_runs.status IS DISTINCT FROM 'completed' OR excluded.status='completed'))",&[&item.id,&item.run,&attempt,&status,&conclusion,&updated,&item.refresh_version]).map_err(internal)?;
    if changed == 0 {
        c.execute(
            "UPDATE github_workflow_runs SET checked_at=CASE WHEN refresh_version=$2 THEN now() ELSE checked_at END WHERE deployment_id=$1",
            &[&item.id,&item.refresh_version],
        )
        .map_err(internal)?;
    }
    Ok(())
}
pub(super) fn error(c: &mut Connection<'_>, item: &Candidate, message: &str) -> Result<()> {
    if still_linked(c, item)? {
        c.execute("INSERT INTO github_workflow_runs(deployment_id,run_id,checked_at,sync_error) VALUES($1,$2,now(),$3) ON CONFLICT(deployment_id) DO UPDATE SET checked_at=CASE WHEN github_workflow_runs.refresh_version=$4 THEN now() ELSE github_workflow_runs.checked_at END,sync_error=excluded.sync_error",&[&item.id,&item.run,&message,&item.refresh_version]).map_err(internal)?;
    }
    Ok(())
}
