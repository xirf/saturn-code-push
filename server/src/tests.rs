use super::*;
use axum::http::Request;
use http_body_util::BodyExt;

const TOKEN: &str = "sb_api_1234567890123456789012345678901234567890";
fn service() -> (tempfile::TempDir, Service) {
    let dir = tempfile::tempdir().unwrap();
    let s = Service::open(
        dir.path().into(),
        "http://localhost:3000".into(),
        TOKEN.into(),
    )
    .unwrap();
    (dir, s)
}
async fn call(
    s: &Service,
    method: &str,
    path: &str,
    body: Value,
    auth: bool,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if auth {
        request = request.header("Authorization", format!("Bearer {TOKEN}"));
    }
    let r = router(s.clone())
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = r.status();
    let bytes = r.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
fn fixture(s: &Service) {
    s.write(|d| {
        d.apps.push(App {
            id: "app".into(),
            display_name: "Test".into(),
            created_at: now(),
        });
        d.channels.push(Channel {
            id: 1,
            app_id: "app".into(),
            name: "stable".into(),
        });
        d.releases.push(Release {
            id: 2,
            app_id: "app".into(),
            version: "1.0.0+1".into(),
            flutter_revision: "revision".into(),
            flutter_version: None,
            platforms: HashMap::from([("android".into(), json!({"status":"active"}))]),
            created_at: now(),
            updated_at: now(),
        });
        d.sequence = 2;
        Ok(())
    })
    .unwrap();
}
fn query() -> Check {
    Check {
        app_id: "app".into(),
        release_version: "1.0.0+1".into(),
        channel: "stable".into(),
        platform: "android".into(),
        arch: "x86_64".into(),
        current_patch_number: None,
        patch_number: None,
    }
}
fn artifact(s: &Service, patch: Option<i64>, hash: &str, size: u64) -> Artifact {
    s.write(|d| {
        let a = Artifact {
            id: d.next(),
            app_id: "app".into(),
            release_id: 2,
            patch_id: patch,
            arch: "x86_64".into(),
            platform: "android".into(),
            hash: hash.into(),
            hash_signature: None,
            size,
            can_sideload: false,
            signing: None,
            upload_key: uid(),
            download_key: uid(),
            uploaded: false,
        };
        d.artifacts.push(a.clone());
        Ok(a)
    })
    .unwrap()
}
fn patch(s: &Service, number: i64) -> i64 {
    s.write(|d| {
        let id = d.next();
        d.patches.push(Patch {
            id,
            release_id: 2,
            number,
            channel_ids: vec![1],
            is_rolled_back: false,
            created_at: now(),
        });
        Ok(id)
    })
    .unwrap()
}
async fn upload_bytes(s: &Service, a: &Artifact, bytes: &[u8]) -> StatusCode {
    let mut body=b"--boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"artifact\"\r\nContent-Type: application/octet-stream\r\n\r\n".to_vec();
    body.extend(bytes);
    body.extend(b"\r\n--boundary--\r\n");
    router(s.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/uploads/{}", a.upload_key))
                .header("Content-Type", "multipart/form-data; boundary=boundary")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}
#[tokio::test]
async fn management_requires_token_and_device_api_does_not() {
    let (_dir, s) = service();
    assert_eq!(
        call(
            &s,
            "POST",
            "/api/v1/apps",
            json!({"display_name":"Oops"}),
            false
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&s, "GET", "/admin/state", json!({}), false).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(call(&s,"POST","/api/v1/patches/check",json!({"app_id":"unknown","release_version":"1","platform":"android","arch":"x86_64","channel":"stable"}),false).await.1["patch_available"],false);
}
#[tokio::test]
async fn app_and_release_lifecycle_is_persistent() {
    let (dir, s) = service();
    let (code, a) = call(
        &s,
        "POST",
        "/api/v1/apps",
        json!({"display_name":"Demo"}),
        true,
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    let app = a["id"].as_str().unwrap();
    let path = format!("/api/v1/apps/{app}/releases");
    let (code, r) = call(
        &s,
        "POST",
        &path,
        json!({"version":"1.0.0+1","flutter_revision":"abc"}),
        true,
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    let id = r["release"]["id"].as_i64().unwrap();
    assert_eq!(
        call(
            &s,
            "PATCH",
            &format!("{path}/{id}"),
            json!({"status":"active","platform":"android"}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    drop(s);
    let s = Service::open(
        dir.path().into(),
        "http://localhost:3000".into(),
        TOKEN.into(),
    )
    .unwrap();
    assert_eq!(
        call(&s, "GET", &path, json!({}), true).await.1["releases"][0]["version"],
        "1.0.0+1"
    );
}
#[tokio::test]
async fn uploads_check_hash_size_and_are_immutable() {
    let (_dir, s) = service();
    fixture(&s);
    let a = artifact(&s, None, &format!("{:x}", Sha256::digest(b"abc")), 3);
    assert_eq!(upload_bytes(&s, &a, b"bad").await, StatusCode::BAD_REQUEST);
    assert_eq!(upload_bytes(&s, &a, b"abcd").await, StatusCode::BAD_REQUEST);
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::OK);
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::NOT_FOUND);
    assert_eq!(
        std::fs::read(s.root.join("objects").join(a.id.to_string())).unwrap(),
        b"abc"
    );
    assert!(
        std::fs::read_dir(s.root.join("objects"))
            .unwrap()
            .all(|p| !p.unwrap().path().to_string_lossy().ends_with(".tmp"))
    );
}
#[tokio::test]
async fn artifact_download_supports_byte_ranges() {
    let (_dir, s) = service();
    fixture(&s);
    let a = artifact(&s, None, &format!("{:x}", Sha256::digest(b"abcdef")), 6);
    assert_eq!(upload_bytes(&s, &a, b"abcdef").await, StatusCode::OK);
    let r = router(s.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/objects/{}", a.download_key))
                .header("Range", "bytes=2-4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(r.into_body().collect().await.unwrap().to_bytes(), "cde");
}
#[tokio::test]
async fn unpublished_or_incomplete_patch_is_never_offered() {
    let (_dir, s) = service();
    fixture(&s);
    let id = patch(&s, 1);
    let a = artifact(&s, Some(id), &"a".repeat(64), 3);
    assert_eq!(
        select_patch(&s, &query()).unwrap()["patch_available"],
        false
    );
    let promote = "/api/v1/apps/app/patches/promote";
    assert_eq!(
        call(
            &s,
            "POST",
            promote,
            json!({"patch_id":id,"channel_id":1}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::OK);
    // Upload completion alone does not satisfy the baseline requirement.
    assert_eq!(
        call(
            &s,
            "POST",
            promote,
            json!({"patch_id":id,"channel_id":1}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}
#[tokio::test]
async fn selection_is_scoped_and_rollback_can_return_to_previous_patch() {
    let (_dir, s) = service();
    fixture(&s);
    for n in [1, 2] {
        let id = patch(&s, n);
        let a = artifact(&s, Some(id), &"a".repeat(64), 3);
        upload_bytes(&s, &a, b"abc").await;
    }
    assert_eq!(select_patch(&s, &query()).unwrap()["patch"]["number"], 2);
    for field in ["app", "version", "platform", "arch", "channel"] {
        let mut q = query();
        match field {
            "app" => q.app_id = "other".into(),
            "version" => q.release_version = "2".into(),
            "platform" => q.platform = "ios".into(),
            "arch" => q.arch = "aarch64".into(),
            _ => q.channel = "beta".into(),
        };
        assert_eq!(
            select_patch(&s, &q).unwrap()["patch_available"],
            false,
            "{field}"
        );
    }
    let mut q = query();
    q.current_patch_number = Some(2);
    assert_eq!(select_patch(&s, &q).unwrap()["patch_available"], false);
    let id = s.read(|d| Ok(d.patches[1].id)).unwrap();
    let _ = set_rollback(&s, "app".into(), 2, id, true).unwrap();
    let result = select_patch(&s, &q).unwrap();
    assert_eq!(result["patch"]["number"], 1);
    assert_eq!(result["rolled_back_patch_numbers"], json!([2]));
    q.current_patch_number = None;
    q.patch_number = Some(1);
    assert_eq!(select_patch(&s, &q).unwrap()["patch_available"], false);
}
#[tokio::test]
async fn failed_mutations_do_not_commit() {
    let (_dir, s) = service();
    let result: Result<()> = s.write(|d| {
        d.sequence = 999;
        Err(bad("Fail"))
    });
    assert!(result.is_err());
    assert_eq!(s.read(|d| Ok(d.sequence)).unwrap(), 0);
}
#[tokio::test]
async fn anonymous_events_cannot_create_apps_or_accept_unknown_types() {
    let (_dir, s) = service();
    fixture(&s);
    let path = "/api/v1/patches/events";
    assert_eq!(
        call(
            &s,
            "POST",
            path,
            json!({"event":{"app_id":"app","type":"evil"}}),
            false
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &s,
            "POST",
            path,
            json!({"event":{"app_id":"missing","type":"__patch_download__"}}),
            false
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &s,
            "POST",
            path,
            json!({"event":{"app_id":"app","type":"__patch_install__","patch_number":1}}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(s.read(|d| Ok(d.events.len())).unwrap(), 1);
}

#[tokio::test]
async fn dart_text_plain_json_is_accepted() {
    let (_dir, s) = service();
    let response = router(s)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("Authorization", format!("Bearer {TOKEN}"))
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Body::from(r#"{"display_name":"Dart wire format"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn active_release_cannot_be_replaced() {
    let (_dir, s) = service();
    fixture(&s);
    assert_eq!(
        call(
            &s,
            "POST",
            "/api/v1/apps/app/releases",
            json!({"version":"1.0.0+1","flutter_revision":"different"}),
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn analytics_are_scoped_and_distinguish_reports_from_installs() {
    let (_dir, s) = service();
    fixture(&s);
    for client in ["one", "one", "two"] {
        assert_eq!(call(&s,"POST","/api/v1/patches/events",json!({"event":{"app_id":"app","type":"__patch_download__","client_id":client,"release_version":"1.0.0+1","patch_number":1}}),false).await.0,StatusCode::OK);
    }
    let result = call(&s, "GET", "/admin/analytics?app_id=app", json!({}), true)
        .await
        .1;
    assert_eq!(result["event_counts"]["__patch_download__"], 3);
    assert_eq!(result["unique_reporting_installs"], 2);
    let empty = call(
        &s,
        "GET",
        "/admin/analytics?app_id=another",
        json!({}),
        true,
    )
    .await
    .1;
    assert_eq!(empty["retained_events"], 0);
    assert_eq!(
        call(&s, "GET", "/admin/analytics", json!({}), false)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn device_requests_are_rate_limited() {
    let (_dir, s) = service();
    *s.rate.lock().unwrap() = (std::time::Instant::now(), 100);
    let response=call(&s,"POST","/api/v1/patches/check",json!({"app_id":"x","release_version":"1","channel":"stable","platform":"android","arch":"x86_64"}),false).await;
    assert_eq!(response.0, StatusCode::TOO_MANY_REQUESTS);
}

const PASSWORD: &str = "correct-horse-saturn-42";
fn dashboard_service() -> (tempfile::TempDir, Service) {
    let (dir, mut s) = service();
    s.base_url = "https://saturn.test".into();
    s.dashboard = Some(Arc::new(
        auth::Credentials::new("admin".into(), auth::hash_password(PASSWORD).unwrap()).unwrap(),
    ));
    (dir, s)
}
async fn browser(
    s: &Service,
    method: &str,
    path: &str,
    body: Value,
    cookie: Option<&str>,
    origin: Option<&str>,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(cookie) = cookie {
        request = request.header("Cookie", cookie)
    }
    if let Some(origin) = origin {
        request = request.header("Origin", origin)
    }
    router(s.clone())
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}
async fn sign_in(s: &Service) -> String {
    let response = browser(
        s,
        "POST",
        "/auth/login",
        json!({"username":"admin","password":PASSWORD}),
        None,
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Max-Age=28800"));
    cookie.split(';').next().unwrap().to_owned()
}
#[tokio::test]
async fn password_login_and_csrf_protected_mutation() {
    let (_dir, s) = dashboard_service();
    let wrong = browser(
        &s,
        "POST",
        "/auth/login",
        json!({"username":"admin","password":"incorrect"}),
        None,
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    let cookie = sign_in(&s).await;
    assert_eq!(
        browser(&s, "GET", "/admin/state", json!({}), Some(&cookie), None)
            .await
            .status(),
        StatusCode::OK
    );
    for origin in [None, Some("https://attacker.test")] {
        assert_eq!(
            browser(
                &s,
                "POST",
                "/api/v1/apps",
                json!({"display_name":"Blocked"}),
                Some(&cookie),
                origin
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        browser(
            &s,
            "POST",
            "/api/v1/apps",
            json!({"display_name":"Allowed"}),
            Some(&cookie),
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::OK
    );
    let stored: String = s
        .store
        .lock()
        .unwrap()
        .connection
        .query_row("SELECT token_hash FROM dashboard_sessions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(stored.len(), 64);
    assert!(!cookie.contains(&stored));
}
#[tokio::test]
async fn logout_revokes_the_session_and_clears_cookie() {
    let (_dir, s) = dashboard_service();
    let cookie = sign_in(&s).await;
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/logout",
            json!({}),
            Some(&cookie),
            Some("https://attacker.test")
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let response = browser(
        &s,
        "POST",
        "/auth/logout",
        json!({}),
        Some(&cookie),
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    assert_eq!(
        browser(&s, "GET", "/admin/state", json!({}), Some(&cookie), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn sessions_survive_restarts_and_expire() {
    let (dir, s) = dashboard_service();
    let cookie = sign_in(&s).await;
    let mut restarted = Service::open(dir.path().into(), s.base_url.clone(), TOKEN.into()).unwrap();
    restarted.dashboard = s.dashboard.clone();
    assert_eq!(
        browser(
            &restarted,
            "GET",
            "/auth/session",
            json!({}),
            Some(&cookie),
            None
        )
        .await
        .status(),
        StatusCode::OK
    );
    restarted
        .store
        .lock()
        .unwrap()
        .connection
        .execute("UPDATE dashboard_sessions SET expires_at=0", [])
        .unwrap();
    assert_eq!(
        browser(
            &restarted,
            "GET",
            "/auth/session",
            json!({}),
            Some(&cookie),
            None
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn password_reset_invalidates_existing_sessions() {
    let (_dir, mut s) = dashboard_service();
    let cookie = sign_in(&s).await;
    s.dashboard = Some(Arc::new(
        auth::Credentials::new(
            "admin".into(),
            auth::hash_password("a-different-password-42").unwrap(),
        )
        .unwrap(),
    ));
    assert_eq!(
        browser(&s, "GET", "/admin/state", json!({}), Some(&cookie), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn login_has_origin_and_attempt_limits() {
    let (_dir, s) = dashboard_service();
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/login",
            json!({"username":"admin","password":PASSWORD}),
            None,
            None
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    *s.login_rate.lock().unwrap() = (std::time::Instant::now(), 10);
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/login",
            json!({"username":"admin","password":PASSWORD}),
            None,
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn asset_bundle_is_scoped_to_running_patch_and_rolls_back() {
    let (_dir, s) = service();
    fixture(&s);
    s.write(|d| {
        d.patches.push(Patch {
            id: 3,
            release_id: 2,
            number: 1,
            channel_ids: vec![1],
            is_rolled_back: false,
            created_at: now(),
        });
        d.sequence = 3;
        Ok(())
    })
    .unwrap();
    let bytes = b"test asset archive";
    let digest = format!("{:x}", Sha256::digest(bytes));
    let (status, registered) = call(
        &s,
        "POST",
        "/api/v1/apps/app/patches/3/asset-bundle",
        json!({"sha256":digest,"size":bytes.len(),"platform":"android"}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let upload_key = registered["upload_url"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let response = router(s.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/asset-uploads/{upload_key}"))
                .header("content-type", "application/octet-stream")
                .body(Body::from(bytes.to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let path = "/api/v1/assets/check?app_id=app&release_version=1.0.0%2B1&channel=stable&platform=android&patch_number=1";
    let (status, selected) = call(&s, "GET", path, json!({}), false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(selected["asset_bundle_available"], true);
    assert_eq!(selected["sha256"], digest);
    let download_key = selected["download_url"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let response = router(s.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/asset-objects/{download_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        bytes.as_slice()
    );
    let (_, baseline) = call(
        &s,
        "GET",
        &path.replace("patch_number=1", "patch_number=0"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(baseline["asset_bundle_available"], false);
    let (status, _) = call(
        &s,
        "POST",
        "/api/v1/apps/app/releases/2/patches/3/rollback",
        json!({}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, rolled) = call(&s, "GET", path, json!({}), false).await;
    assert_eq!(rolled["asset_bundle_available"], false);
}
