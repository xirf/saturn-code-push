use super::*;

const SETUP_TOKEN: &str = "first-run-setup-token-12345678901234567890";
fn unconfigured() -> (tempfile::TempDir, Service) {
    let (dir, mut s) = service();
    s.base_url = "https://saturn.test".into();
    s.setup_token_digest = Some(Sha256::digest(SETUP_TOKEN.as_bytes()).into());
    (dir, s)
}
fn input(username: &str) -> Value {
    json!({"username":username,"password":PASSWORD,"setup_token":SETUP_TOKEN})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_account_setup_creates_a_session_and_closes_forever() {
    let (_dir, s) = unconfigured();
    assert_eq!(
        call(&s, "GET", "/auth/setup", json!({}), false).await.1,
        json!({"required":true,"enabled":true})
    );
    let response = browser(
        &s,
        "POST",
        "/auth/setup",
        input("owner"),
        None,
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    assert!(
        response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("HttpOnly")
    );
    let user: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(user["username"], "owner");
    assert_eq!(user["role"], "admin");
    assert_eq!(
        browser(&s, "GET", "/admin/state", json!({}), Some(&cookie), None)
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        call(&s, "GET", "/auth/setup", json!({}), false).await.1,
        json!({"required":false,"enabled":false})
    );
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/setup",
            input("attacker"),
            None,
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    s.database
        .transaction(|c| {
            assert_eq!(
                c.query_one("SELECT COUNT(*) FROM identity_accounts", &[])
                    .map_err(internal)?
                    .get::<_, i64>(0),
                1
            );
            let hash: String = c
                .query_one("SELECT password_hash FROM identity_accounts", &[])
                .map_err(internal)?
                .get(0);
            assert!(hash.starts_with("$argon2id$"));
            assert_ne!(hash, PASSWORD);
            assert_eq!(
                c.query_one("SELECT action FROM identity_audit", &[])
                    .map_err(internal)?
                    .get::<_, String>(0),
                "SETUP"
            );
            c.execute("UPDATE identity_accounts SET disabled=TRUE", &[])
                .map_err(internal)?;
            Ok(())
        })
        .unwrap();
    // Disabled accounts never reopen first-run registration.
    assert_eq!(
        call(&s, "GET", "/auth/setup", json!({}), false).await.1["required"],
        false
    );
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn setup_requires_origin_token_valid_credentials_and_attempt_budget() {
    let (_dir, s) = unconfigured();
    for origin in [None, Some("https://attacker.test")] {
        assert_eq!(
            browser(&s, "POST", "/auth/setup", input("owner"), None, origin)
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    let mut wrong = input("owner");
    wrong["setup_token"] = json!("wrong");
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/setup",
            wrong,
            None,
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut weak = input("owner");
    weak["password"] = json!("short");
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/setup",
            weak,
            None,
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    assert!(s.database.transaction(auth::needs_setup).unwrap());
    *s.login_rate.lock().unwrap() = (std::time::Instant::now(), 10);
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/setup",
            input("owner"),
            None,
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn setup_fails_closed_without_deployment_token_and_has_one_concurrent_winner() {
    let (_dir, mut s) = unconfigured();
    s.setup_token_digest = None;
    assert_eq!(
        call(&s, "GET", "/auth/setup", json!({}), false).await.1["enabled"],
        false
    );
    assert_eq!(
        browser(
            &s,
            "POST",
            "/auth/setup",
            input("owner"),
            None,
            Some("https://saturn.test")
        )
        .await
        .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    s.setup_token_digest = Some(Sha256::digest(SETUP_TOKEN.as_bytes()).into());
    let (first, second) = tokio::join!(
        browser(
            &s,
            "POST",
            "/auth/setup",
            input("one"),
            None,
            Some("https://saturn.test")
        ),
        browser(
            &s,
            "POST",
            "/auth/setup",
            input("two"),
            None,
            Some("https://saturn.test")
        )
    );
    let mut statuses = [first.status(), second.status()];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::CONFLICT]);
}
