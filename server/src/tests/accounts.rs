use super::*;

async fn json_response(response: Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}
async fn admin(
    s: &Service,
    method: &str,
    path: &str,
    value: Value,
    cookie: &str,
) -> (StatusCode, Value) {
    let response = browser(
        s,
        method,
        path,
        value,
        Some(cookie),
        Some("https://saturn.test"),
    )
    .await;
    let code = response.status();
    (code, json_response(response).await)
}
async fn named_login(s: &Service, username: &str) -> String {
    let response = browser(
        s,
        "POST",
        "/auth/login",
        json!({"username":username,"password":PASSWORD}),
        None,
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .into()
}
async fn bearer(
    s: &Service,
    token: &str,
    method: &str,
    path: &str,
    value: Value,
) -> (StatusCode, Value) {
    let response = router(s.clone())
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let code = response.status();
    (code, json_response(response).await)
}
fn report() -> Value {
    json!({"release_id":2,"key":"run","repository":"owner/repo","commit":"a".repeat(40),"source_kind":"commit","source_ref":"main","destination":"google-play/production","status":"published"})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn roles_are_enforced_and_last_admin_is_preserved() {
    let (_dir, s) = dashboard_service();
    fixture(&s);
    let cookie = sign_in(&s).await;
    let (status, viewer) = admin(
        &s,
        "POST",
        "/admin/accounts",
        json!({"username":"reader","password":PASSWORD,"role":"viewer"}),
        &cookie,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(viewer.get("password_hash").is_none());
    let reader = named_login(&s, "reader").await;
    assert_eq!(
        admin(&s, "GET", "/admin/state", json!({}), &reader).await.0,
        StatusCode::OK
    );
    assert_eq!(
        admin(&s, "GET", "/admin/accounts", json!({}), &reader)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        admin(
            &s,
            "POST",
            "/api/v1/apps/app/deployments",
            report(),
            &reader
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, publisher) = admin(
        &s,
        "POST",
        "/admin/accounts",
        json!({"username":"publisher","password":PASSWORD,"role":"publisher"}),
        &cookie,
    )
    .await;
    let publisher_cookie = named_login(&s, "publisher").await;
    let (code, records) = admin(
        &s,
        "POST",
        "/api/v1/apps/app/deployments",
        report(),
        &publisher_cookie,
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(records[0]["reported_by"], "account: publisher");
    assert_eq!(
        admin(&s, "POST", "/admin/api-keys", json!({}), &publisher_cookie)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (_, session) = admin(&s, "GET", "/auth/session", json!({}), &cookie).await;
    assert_eq!(
        admin(
            &s,
            "PATCH",
            &format!("/admin/accounts/{}", session["id"].as_str().unwrap()),
            json!({"disabled":true}),
            &cookie
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        admin(
            &s,
            "PATCH",
            &format!("/admin/accounts/{}", publisher["id"].as_str().unwrap()),
            json!({"disabled":true}),
            &cookie
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        admin(&s, "GET", "/auth/session", json!({}), &publisher_cookie)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let (_, audit) = admin(&s, "GET", "/admin/audit", json!({}), &cookie).await;
    assert!(
        audit["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["actor_name"] == "publisher"
                && e["resource"] == "/api/v1/apps/app/deployments")
    );
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn api_keys_are_hashed_scoped_expiring_and_revocable() {
    let (_dir, s) = dashboard_service();
    fixture(&s);
    let cookie = sign_in(&s).await;
    let (code,key)=admin(&s,"POST","/admin/api-keys",json!({"name":"GitHub CI","app_id":"app","permission":"deployments:write","expires_in_days":1}),&cookie).await;
    assert_eq!(code, StatusCode::OK);
    let token = key["token"].as_str().unwrap();
    let id = key["id"].as_str().unwrap();
    assert_eq!(
        bearer(&s, token, "POST", "/api/v1/apps/app/deployments", report())
            .await
            .0,
        StatusCode::OK
    );
    for path in [
        "/admin/state",
        "/admin/api-keys",
        "/api/v1/apps/app/releases",
        "/api/v1/apps/other/deployments",
    ] {
        assert_eq!(
            bearer(&s, token, "GET", path, json!({})).await.0,
            StatusCode::FORBIDDEN
        );
    }
    let (_, keys) = admin(&s, "GET", "/admin/api-keys", json!({}), &cookie).await;
    assert!(keys.to_string().find(token).is_none());
    assert!(keys["keys"][0]["last_used_at"].is_string());
    assert!(keys["keys"][0].get("token_hash").is_none());
    s.database
        .transaction(|c| {
            let stored: String = c
                .query_one(
                    "SELECT token_hash FROM identity_api_keys WHERE id=$1",
                    &[&id],
                )
                .map_err(internal)?
                .get(0);
            assert_eq!(stored.len(), 64);
            assert_ne!(stored, token);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        admin(
            &s,
            "POST",
            &format!("/admin/api-keys/{id}/revoke"),
            json!({}),
            &cookie
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        bearer(&s, token, "GET", "/api/v1/apps/app/deployments", json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let (_, expiring) = admin(
        &s,
        "POST",
        "/admin/api-keys",
        json!({"name":"Old key","app_id":"app","permission":"publish","expires_in_days":1}),
        &cookie,
    )
    .await;
    let (_, other_app) = call(
        &s,
        "POST",
        "/api/v1/apps",
        json!({"display_name":"Other app"}),
        true,
    )
    .await;
    let publish_token = expiring["token"].as_str().unwrap();
    let (status, own_apps) = bearer(&s, publish_token, "GET", "/api/v1/apps", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(own_apps["apps"].as_array().unwrap().len(), 1);
    assert_eq!(own_apps["apps"][0]["id"], "app");
    assert_eq!(
        bearer(&s, publish_token, "GET", "/api/v1/users/me", json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        bearer(
            &s,
            publish_token,
            "GET",
            &format!(
                "/api/v1/apps/{}/releases",
                other_app["id"].as_str().unwrap()
            ),
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    s.database
        .transaction(|c| {
            c.execute("UPDATE identity_api_keys SET expires_at=0", &[])
                .map_err(internal)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        bearer(
            &s,
            expiring["token"].as_str().unwrap(),
            "GET",
            "/api/v1/apps/app/releases",
            json!({})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn password_changes_revoke_sessions_and_keys_and_bootstrap_does_not_reset_accounts() {
    let (_dir, s) = dashboard_service();
    fixture(&s);
    let cookie = sign_in(&s).await;
    let (_, key) = admin(
        &s,
        "POST",
        "/admin/api-keys",
        json!({"name":"CI","app_id":"app","permission":"publish","expires_in_days":1}),
        &cookie,
    )
    .await;
    assert_eq!(
        admin(
            &s,
            "POST",
            "/auth/password",
            json!({"current_password":"wrong","password":"a-new-password-123"}),
            &cookie
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        admin(
            &s,
            "POST",
            "/auth/password",
            json!({"current_password":PASSWORD,"password":"a-new-password-123"}),
            &cookie
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        admin(&s, "GET", "/auth/session", json!({}), &cookie)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        bearer(
            &s,
            key["token"].as_str().unwrap(),
            "GET",
            "/api/v1/apps/app/releases",
            json!({})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    s.database
        .transaction(|c| {
            auth::bootstrap(
                c,
                &auth::Credentials::new("admin".into(), auth::hash_password(PASSWORD)?)?,
            )
        })
        .unwrap();
    let login = browser(
        &s,
        "POST",
        "/auth/login",
        json!({"username":"admin","password":PASSWORD}),
        None,
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(login.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revoked_actor_is_rechecked_inside_the_transaction() {
    let (_dir, s) = dashboard_service();
    fixture(&s);
    let cookie = sign_in(&s).await;
    let (_, key) = admin(
        &s,
        "POST",
        "/admin/api-keys",
        json!({"name":"CI","app_id":"app","permission":"publish","expires_in_days":1}),
        &cookie,
    )
    .await;
    let actor = modules::identity::api::key_actor(&s, key["token"].as_str().unwrap())
        .await
        .unwrap()
        .unwrap();
    admin(
        &s,
        "POST",
        &format!("/admin/api-keys/{}/revoke", key["id"].as_str().unwrap()),
        json!({}),
        &cookie,
    )
    .await;
    let context = crate::platform::database::AuditContext {
        actor,
        action: "POST".into(),
        resource: "/api/v1/apps/app/deployments".into(),
        app_id: Some("app".into()),
        write: true,
    };
    let result = crate::platform::database::AUDIT_CONTEXT
        .scope(
            context,
            s.database.call(|c| {
                c.execute(
                    "UPDATE applications_apps SET display_name='Unauthorized' WHERE id='app'",
                    &[],
                )
                .map_err(internal)?;
                Ok(())
            }),
        )
        .await;
    assert_eq!(result.unwrap_err().0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        s.read(|d| Ok(d.apps[0].display_name.clone())).unwrap(),
        "Test"
    );
}

#[test]
fn local_password_recovery_preserves_role_and_records_the_operator() {
    let (_dir, s) = dashboard_service();
    s.database
        .transaction(|c| auth::reset_password(c, "admin", "new-maintenance-password"))
        .unwrap();
    s.database
        .transaction(|c| {
            let role: String = c
                .query_one(
                    "SELECT role FROM identity_accounts WHERE username='admin'",
                    &[],
                )
                .map_err(internal)?
                .get(0);
            assert_eq!(role, "admin");
            let kind: String = c
                .query_one(
                    "SELECT actor_kind FROM identity_audit WHERE action='RESET_PASSWORD'",
                    &[],
                )
                .map_err(internal)?
                .get(0);
            assert_eq!(kind, "system");
            Ok(())
        })
        .unwrap();
}
