use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deployment_reports_are_authenticated_persistent_and_app_scoped() {
    let (dir, s) = service();
    fixture(&s);
    let path = "/api/v1/apps/app/deployments";
    let report = json!({"release_id":2,"key":"run-1","repository":"owner/repo","commit":"a".repeat(40),"source_kind":"release","source_ref":"v1","destination":"app-store","status":"uploaded","workflow_run":42});
    assert_eq!(
        call(&s, "POST", path, report.clone(), false).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (status, first) = call(&s, "POST", path, report.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    let id = first[0]["id"].as_i64().unwrap();
    let mut updated = report.clone();
    updated["status"] = json!("published");
    let (_, second) = call(&s, "POST", path, updated, true).await;
    assert_eq!(second.as_array().unwrap().len(), 1);
    assert_eq!(second[0]["id"], id);
    assert_eq!(second[0]["status"], "published");
    let mut invalid = report.clone();
    invalid["repository"] = json!("https://evil.test/repo");
    assert_eq!(
        call(&s, "POST", path, invalid, true).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&s, "POST", "/api/v1/apps/missing/deployments", report, true)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &s,
            "PATCH",
            &format!("/api/v1/apps/missing/deployments/{id}"),
            json!({"status":"published"}),
            true
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &s,
            "PATCH",
            &format!("{path}/{id}"),
            json!({"status":"in_review"}),
            true
        )
        .await
        .0,
        StatusCode::OK
    );
    drop(s);
    let reopened = Service::open(
        dir.path().into(),
        "http://localhost:3000".into(),
        TOKEN.into(),
    )
    .unwrap();
    assert_eq!(
        call(&reopened, "GET", path, json!({}), true).await.1[0]["status"],
        "in_review"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deployment_target_is_explicit_and_patch_ownership_is_immutable() {
    let (_dir, s) = service();
    fixture(&s);
    let patch_id = patch(&s, 1);
    let path = "/api/v1/apps/app/deployments";
    let report = json!({"release_id":2,"patch_id":patch_id,"key":"patch-run","repository":"owner/repo","commit":"a".repeat(40),"source_kind":"commit","source_ref":"main","destination":"ota/stable","status":"running","workflow_run":42});
    let (status, result) = call(&s, "POST", path, report.clone(), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result[0]["patch_id"], patch_id);
    let mut invalid_attempt = report.clone();
    invalid_attempt["workflow_attempt"] = json!(0);
    assert_eq!(
        call(&s, "POST", path, invalid_attempt, true).await.0,
        StatusCode::BAD_REQUEST
    );
    let mut missing_run = report.clone();
    missing_run["workflow_attempt"] = json!(1);
    missing_run["workflow_run"] = Value::Null;
    assert_eq!(
        call(&s, "POST", path, missing_run, true).await.0,
        StatusCode::BAD_REQUEST
    );
    let mut changed = report.clone();
    changed["patch_id"] = Value::Null;
    assert_eq!(
        call(&s, "POST", path, changed, true).await.0,
        StatusCode::BAD_REQUEST
    );
    let mut invalid = report.clone();
    invalid["patch_id"] = json!(999);
    assert_eq!(
        call(&s, "POST", path, invalid, true).await.0,
        StatusCode::NOT_FOUND
    );
    s.write(|d| {
        let mut release = d.releases[0].clone();
        release.id = d.next();
        release.version = "2.0.0+2".into();
        d.releases.push(release);
        Ok(())
    })
    .unwrap();
    let other_release = s.read(|d| Ok(d.releases.last().unwrap().id)).unwrap();
    let mut wrong = report;
    wrong["release_id"] = json!(other_release);
    wrong["key"] = json!("wrong-parent");
    assert_eq!(
        call(&s, "POST", path, wrong, true).await.0,
        StatusCode::BAD_REQUEST
    );
}
