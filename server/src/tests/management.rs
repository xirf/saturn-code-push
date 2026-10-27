use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
    let (_, response) = call(
        &s,
        "POST",
        "/api/v1/patches/check",
        json!({
            "app_id": "unknown",
            "release_version": "1",
            "platform": "android",
            "arch": "x86_64",
            "channel": "stable",
        }),
        false,
    )
    .await;
    assert_eq!(response["patch_available"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_mutations_do_not_commit() {
    let (_dir, s) = service();
    let result: Result<()> = s.write(|d| {
        d.sequence = 999;
        Err(bad("Fail"))
    });
    assert!(result.is_err());
    assert_eq!(s.read(|d| Ok(d.sequence)).unwrap(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
