use super::*;

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
async fn analytics_are_scoped_and_distinguish_reports_from_installs() {
    let (_dir, s) = service();
    fixture(&s);
    for client in ["one", "one", "two"] {
        let (status, _) = call(
            &s,
            "POST",
            "/api/v1/patches/events",
            json!({
                "event": {
                    "app_id": "app",
                    "type": "__patch_download__",
                    "client_id": client,
                    "release_version": "1.0.0+1",
                    "patch_number": 1,
                },
            }),
            false,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
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
    let response = call(
        &s,
        "POST",
        "/api/v1/patches/check",
        json!({
            "app_id": "x",
            "release_version": "1",
            "channel": "stable",
            "platform": "android",
            "arch": "x86_64",
        }),
        false,
    )
    .await;
    assert_eq!(response.0, StatusCode::TOO_MANY_REQUESTS);
}
