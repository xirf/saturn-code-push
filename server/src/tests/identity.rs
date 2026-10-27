use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
        .database
        .transaction(|connection| {
            connection
                .query_one("SELECT token_hash FROM dashboard_sessions", &[])
                .map_err(internal)?
                .try_get(0)
                .map_err(internal)
        })
        .unwrap();
    assert_eq!(stored.len(), 64);
    assert!(!cookie.contains(&stored));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sessions_survive_restarts_and_expire() {
    let (dir, s) = dashboard_service();
    let cookie = sign_in(&s).await;
    let restarted = Service::open(dir.path().into(), s.base_url.clone(), TOKEN.into()).unwrap();

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
        .database
        .transaction(|connection| {
            connection
                .execute("UPDATE dashboard_sessions SET expires_at=0", &[])
                .map_err(internal)?;
            Ok(())
        })
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn password_reset_invalidates_existing_sessions() {
    let (_dir, s) = dashboard_service();
    let cookie = sign_in(&s).await;
    s.database
        .transaction(|c| {
            c.execute("DELETE FROM dashboard_sessions", &[])
                .map_err(internal)?;
            c.execute(
                "UPDATE identity_accounts SET password_hash=$1",
                &[&auth::hash_password("a-different-password-42")?],
            )
            .map_err(internal)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        browser(&s, "GET", "/admin/state", json!({}), Some(&cookie), None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
