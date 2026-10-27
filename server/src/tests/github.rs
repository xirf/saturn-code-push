use super::*;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Mock {
    run: Arc<Mutex<Value>>,
    tokens: Arc<Mutex<usize>>,
    block: Arc<std::sync::atomic::AtomicBool>,
    started: Arc<tokio::sync::Notify>,
    proceed: Arc<tokio::sync::Notify>,
}
async fn installation(headers: HeaderMap) -> Json<Value> {
    let auth = headers["authorization"].to_str().unwrap();
    assert!(auth.starts_with("Bearer eyJ"));
    Json(json!({"app_id":42,"suspended_at":null}))
}
async fn token(
    State(mock): State<Mock>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    assert!(
        headers["authorization"]
            .to_str()
            .unwrap()
            .starts_with("Bearer eyJ")
    );
    assert_eq!(body["permissions"]["actions"], "read");
    assert_eq!(body["repositories"], json!(["repo"]));
    *mock.tokens.lock().unwrap() += 1;
    Json(json!({"token":"installation-test-token"}))
}
async fn repo(headers: HeaderMap) -> Json<Value> {
    assert_eq!(headers["authorization"], "Bearer installation-test-token");
    Json(json!({"full_name":"owner/repo"}))
}
async fn run(State(mock): State<Mock>, Path(id): Path<i64>, headers: HeaderMap) -> Json<Value> {
    assert_eq!(id, 42);
    assert_eq!(headers["authorization"], "Bearer installation-test-token");
    if mock.block.swap(false, std::sync::atomic::Ordering::SeqCst) {
        mock.started.notify_one();
        mock.proceed.notified().await;
    }
    Json(mock.run.lock().unwrap().clone())
}
async fn run_attempt(Path((id, attempt)): Path<(i64, i64)>, headers: HeaderMap) -> Json<Value> {
    assert_eq!(id, 42);
    assert_eq!(attempt, 1);
    assert_eq!(headers["authorization"], "Bearer installation-test-token");
    Json(
        json!({"id":42,"run_attempt":1,"status":"completed","conclusion":"success","head_sha":"a".repeat(40),"repository":{"full_name":"owner/repo"},"updated_at":"2026-10-27T00:00:00Z"}),
    )
}
async fn mock_server() -> (String, Mock, tokio::task::JoinHandle<()>) {
    let mock = Mock {
        run: Arc::new(Mutex::new(
            json!({"id":42,"run_attempt":1,"status":"completed","conclusion":"success","head_sha":"a".repeat(40),"repository":{"full_name":"owner/repo"},"updated_at":"2026-10-27T00:00:00Z"}),
        )),
        tokens: Arc::new(Mutex::new(0)),
        block: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        started: Arc::new(tokio::sync::Notify::new()),
        proceed: Arc::new(tokio::sync::Notify::new()),
    };
    let router = Router::new()
        .route("/app/installations/7", get(installation))
        .route("/app/installations/7/access_tokens", post(token))
        .route("/repos/owner/repo", get(repo))
        .route("/repos/owner/repo/actions/runs/{id}", get(run))
        .route(
            "/repos/owner/repo/actions/runs/{id}/attempts/{attempt}",
            get(run_attempt),
        )
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    (format!("http://{address}"), mock, handle)
}
async fn report(s: &Service) {
    assert_eq!(call(s,"POST","/api/v1/apps/app/deployments",json!({"release_id":2,"key":"run","repository":"owner/repo","commit":"a".repeat(40),"source_kind":"commit","source_ref":"main","destination":"app-store/production","status":"uploaded","workflow_run":42}),true).await.0,StatusCode::OK);
}
async fn payload(response: Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}
async fn connect(s: &Service, cookie: &str) -> StatusCode {
    browser(
        s,
        "POST",
        "/admin/integrations/github",
        json!({"app_id":"app","repository":"owner/repo","installation_id":7}),
        Some(cookie),
        Some("https://saturn.test"),
    )
    .await
    .status()
}
fn due(s: &Service) {
    s.database
        .transaction(|c| {
            c.execute(
                "UPDATE github_workflow_runs SET checked_at=to_timestamp(0)",
                &[],
            )
            .map_err(internal)?;
            Ok(())
        })
        .unwrap()
}
fn signature(app: &modules::github::App, body: &[u8]) -> String {
    let key = openssl::pkey::PKey::hmac(&app.secret).unwrap();
    let mut signer =
        openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), &key).unwrap();
    signer.update(body).unwrap();
    format!("sha256={}", hex::encode(signer.sign_to_vec().unwrap()))
}
async fn webhook(
    s: &Service,
    body: Value,
    signature_override: Option<&str>,
    delivery: &str,
) -> StatusCode {
    let bytes = serde_json::to_vec(&body).unwrap();
    let signature = signature_override
        .map(str::to_owned)
        .unwrap_or_else(|| signature(s.github.as_ref().unwrap(), &bytes));
    router(s.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/integrations/github/webhook")
                .header("x-hub-signature-256", signature)
                .header("x-github-event", "workflow_run")
                .header("x-github-delivery", delivery)
                .body(Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_connection_and_reconciliation_keep_ci_separate_from_publication() {
    let (url, mock, handle) = mock_server().await;
    let (_dir, mut s) = dashboard_service();
    fixture(&s);
    s.github = Some(modules::github::App::for_test(url));
    let cookie = sign_in(&s).await;
    assert_eq!(
        call(&s, "GET", "/admin/integrations/github", json!({}), false)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &s,
            "POST",
            "/admin/integrations/github",
            json!({"app_id":"app","repository":"owner/repo","installation_id":7}),
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(connect(&s, &cookie).await, StatusCode::OK);
    report(&s).await;
    modules::github::start(s.database.clone(), s.github.as_ref().unwrap()).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
                .await
                .1;
            if records[0]["workflow"]["conclusion"] == "success" {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
        .await
        .1;
    assert_eq!(records[0]["workflow"]["conclusion"], "success");
    assert_eq!(records[0]["status"], "uploaded");
    // New attempts supersede completion, but late older attempts cannot undo them.
    {
        let mut value = mock.run.lock().unwrap();
        value["run_attempt"] = json!(2);
        value["status"] = json!("in_progress");
        value["conclusion"] = Value::Null;
    }
    due(&s);
    modules::github::reconcile_for_test(&s).await.unwrap();
    {
        let mut value = mock.run.lock().unwrap();
        value["run_attempt"] = json!(1);
        value["status"] = json!("completed");
        value["conclusion"] = json!("success");
    }
    due(&s);
    modules::github::reconcile_for_test(&s).await.unwrap();
    let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
        .await
        .1;
    assert_eq!(records[0]["workflow"]["run_attempt"], 2);
    assert_eq!(records[0]["workflow"]["status"], "in_progress");
    // A returned run from the wrong commit is never attached as verified status.
    {
        let mut value = mock.run.lock().unwrap();
        value["run_attempt"] = json!(3);
        value["head_sha"] = json!("b".repeat(40));
    }
    due(&s);
    modules::github::reconcile_for_test(&s).await.unwrap();
    let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
        .await
        .1;
    assert!(
        records[0]["workflow"]["sync_error"]
            .as_str()
            .unwrap()
            .contains("does not match")
    );
    let settings = payload(
        browser(
            &s,
            "GET",
            "/admin/integrations/github",
            json!({}),
            Some(&cookie),
            None,
        )
        .await,
    )
    .await;
    assert!(!settings.to_string().contains("installation-test-token"));
    assert!(!settings.to_string().contains("secret"));
    handle.abort();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn signed_webhooks_are_deduplicated_refresh_hints_and_disconnect_clears_status() {
    let (url, _mock, handle) = mock_server().await;
    let (_dir, mut s) = dashboard_service();
    fixture(&s);
    s.github = Some(modules::github::App::for_test(url));
    let cookie = sign_in(&s).await;
    assert_eq!(connect(&s, &cookie).await, StatusCode::OK);
    report(&s).await;
    modules::github::reconcile_for_test(&s).await.unwrap();
    let delivery = uid();
    let body = json!({"installation":{"id":7},"repository":{"full_name":"owner/repo"},"workflow_run":{"id":42,"status":"completed","conclusion":"failure"}});
    assert_eq!(
        webhook(&s, body.clone(), Some("sha256=bad"), &delivery).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        webhook(&s, body.clone(), None, &delivery).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        webhook(&s, body, None, &delivery).await,
        StatusCode::NO_CONTENT
    );
    s.database
        .transaction(|c| {
            assert_eq!(
                c.query_one("SELECT COUNT(*) FROM github_deliveries", &[])
                    .map_err(internal)?
                    .get::<_, i64>(0),
                1
            );
            Ok(())
        })
        .unwrap();
    let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
        .await
        .1;
    assert_eq!(records[0]["workflow"]["conclusion"], "success");
    assert_eq!(records[0]["status"], "uploaded");
    let response = browser(
        &s,
        "POST",
        "/admin/integrations/github/app/disconnect",
        json!({}),
        Some(&cookie),
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
        .await
        .1;
    assert!(records[0]["workflow"].is_null());
    modules::github::reconcile_for_test(&s).await.unwrap();
    assert!(
        call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
            .await
            .1[0]["workflow"]
            .is_null()
    );
    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webhook_hints_survive_in_flight_fetches_and_disconnect_blocks_late_results() {
    let (url, mock, handle) = mock_server().await;
    let (_dir, mut s) = dashboard_service();
    fixture(&s);
    s.github = Some(modules::github::App::for_test(url));
    let cookie = sign_in(&s).await;
    assert_eq!(connect(&s, &cookie).await, StatusCode::OK);
    report(&s).await;
    mock.block.store(true, std::sync::atomic::Ordering::SeqCst);
    let other = s.clone();
    let fetch = tokio::spawn(async move { modules::github::reconcile_for_test(&other).await });
    tokio::time::timeout(std::time::Duration::from_secs(5), mock.started.notified())
        .await
        .unwrap();
    assert_eq!(webhook(&s,json!({"installation":{"id":7},"repository":{"full_name":"owner/repo"},"workflow_run":{"id":42}}),None,&uid()).await,StatusCode::NO_CONTENT);
    mock.proceed.notify_one();
    fetch.await.unwrap().unwrap();
    s.database
        .transaction(|c| {
            assert!(
                c.query_one(
                    "SELECT checked_at=to_timestamp(0) FROM github_workflow_runs",
                    &[]
                )
                .map_err(internal)?
                .get::<_, bool>(0)
            );
            Ok(())
        })
        .unwrap();
    {
        let mut run = mock.run.lock().unwrap();
        run["run_attempt"] = json!(2);
        run["status"] = json!("in_progress");
        run["conclusion"] = Value::Null;
    }
    modules::github::reconcile_for_test(&s).await.unwrap();
    assert_eq!(
        call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
            .await
            .1[0]["workflow"]["run_attempt"],
        2
    );
    due(&s);
    mock.block.store(true, std::sync::atomic::Ordering::SeqCst);
    let other = s.clone();
    let fetch = tokio::spawn(async move { modules::github::reconcile_for_test(&other).await });
    tokio::time::timeout(std::time::Duration::from_secs(5), mock.started.notified())
        .await
        .unwrap();
    assert_eq!(
        browser(
            &s,
            "POST",
            "/admin/integrations/github/app/disconnect",
            json!({}),
            Some(&cookie),
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::OK
    );
    mock.proceed.notify_one();
    fetch.await.unwrap().unwrap();
    assert!(
        call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
            .await
            .1[0]["workflow"]
            .is_null()
    );
    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reported_attempt_is_pinned_even_when_the_workflow_is_rerun() {
    let (url, mock, handle) = mock_server().await;
    let (_dir, mut s) = dashboard_service();
    fixture(&s);
    s.github = Some(modules::github::App::for_test(url));
    let cookie = sign_in(&s).await;
    assert_eq!(connect(&s, &cookie).await, StatusCode::OK);
    assert_eq!(call(&s,"POST","/api/v1/apps/app/deployments",json!({"release_id":2,"key":"pinned-run","repository":"owner/repo","commit":"a".repeat(40),"source_kind":"commit","source_ref":"main","destination":"production","status":"published","workflow_run":42,"workflow_attempt":1}),true).await.0,StatusCode::OK);
    modules::github::reconcile_for_test(&s).await.unwrap();
    {
        let mut value = mock.run.lock().unwrap();
        value["run_attempt"] = json!(2);
        value["conclusion"] = json!("failure");
    }
    due(&s);
    modules::github::reconcile_for_test(&s).await.unwrap();
    let records = call(&s, "GET", "/api/v1/apps/app/deployments", json!({}), true)
        .await
        .1;
    assert_eq!(records[0]["workflow_attempt"], 1);
    assert_eq!(records[0]["workflow"]["run_attempt"], 1);
    assert_eq!(records[0]["workflow"]["conclusion"], "success");
    assert!(records[0]["workflow"]["sync_error"].is_null());
    handle.abort();
}
