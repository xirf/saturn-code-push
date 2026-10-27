use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_management_subtree_keeps_authentication_after_nesting() {
    let (_directory, service) = service();
    let routes = [
        ("GET", "/api/v1/users/me"),
        ("GET", "/api/v1/plan"),
        ("GET", "/api/v1/organizations"),
        ("GET", "/api/v1/apps"),
        ("POST", "/api/v1/apps"),
        ("GET", "/api/v1/organizations/1/apps"),
        ("GET", "/api/v1/apps/app"),
        ("GET", "/api/v1/apps/app/channels"),
        ("POST", "/api/v1/apps/app/channels"),
        ("GET", "/api/v1/apps/app/releases"),
        ("POST", "/api/v1/apps/app/releases"),
        ("PATCH", "/api/v1/apps/app/releases/1"),
        ("GET", "/api/v1/apps/app/releases/1/artifacts"),
        ("POST", "/api/v1/apps/app/releases/1/artifacts"),
        ("GET", "/api/v1/apps/app/releases/1/patches"),
        ("POST", "/api/v1/apps/app/patches"),
        ("POST", "/api/v1/apps/app/patches/promote"),
        ("POST", "/api/v1/apps/app/patches/1/artifacts"),
        ("POST", "/api/v1/apps/app/patches/1/asset-bundle"),
        ("POST", "/api/v1/apps/app/releases/1/patches/1/rollback"),
        ("POST", "/api/v1/apps/app/releases/1/patches/1/rollforward"),
        ("GET", "/admin/state"),
        ("GET", "/admin/analytics"),
    ];

    for (method, path) in routes {
        let response = router(service.clone())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn console_deep_links_do_not_turn_unknown_api_paths_into_html() {
    let (directory, mut service) = service();
    service.console_dir = directory.path().join("console");
    std::fs::create_dir(&service.console_dir).unwrap();
    std::fs::write(
        service.console_dir.join("index.html"),
        "<html>Console shell</html>",
    )
    .unwrap();

    for path in [
        "/",
        "/login",
        "/applications",
        "/releases",
        "/releases/12",
        "/patches",
        "/analytics",
        "/settings",
    ] {
        let response = router(service.clone())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }

    for path in [
        "/api/v1/missing",
        "/auth/missing",
        "/admin/missing",
        "/_nuxt/missing.js",
    ] {
        let response = router(service.clone())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn management_request_bodies_are_bounded() {
    let (_directory, service) = service();
    let response = router(service)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("Authorization", format!("Bearer {TOKEN}"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"display_name": "a".repeat(65_536)}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_uploads_publish_one_immutable_object() {
    let (_directory, service) = service();
    fixture(&service);
    let patch_id = patch(&service, 1);
    let artifact = artifact(&service, Some(patch_id), &"a".repeat(64), 3);
    let (first, second) = tokio::join!(
        upload_bytes(&service, &artifact, b"abc"),
        upload_bytes(&service, &artifact, b"xyz"),
    );
    assert_eq!(
        [first, second]
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1
    );
    let bytes = std::fs::read(binary_path(&service, artifact.id).await).unwrap();
    assert_eq!(
        bytes,
        if first == StatusCode::OK {
            b"abc"
        } else {
            b"xyz"
        }
    );
    assert!(
        std::fs::read_dir(service.objects.directory())
            .unwrap()
            .all(|entry| !entry.unwrap().path().to_string_lossy().ends_with(".tmp"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pending_upload_recovers_an_object_left_before_commit() {
    let (_directory, service) = service();
    fixture(&service);
    let artifact = artifact(&service, None, &hex::encode(Sha256::digest(b"abc")), 3);
    let location = service.objects.destination(
        crate::platform::objects::Kind::Binary,
        &artifact.app_id,
        artifact.release_id,
        artifact.patch_id,
        artifact.id,
    );
    let orphan = service.objects.local_path(&location);
    std::fs::create_dir_all(orphan.parent().unwrap()).unwrap();
    std::fs::write(orphan, b"orphan").unwrap();
    assert_eq!(
        upload_bytes(&service, &artifact, b"abc").await,
        StatusCode::OK
    );
    assert_eq!(
        std::fs::read(binary_path(&service, artifact.id).await).unwrap(),
        b"abc"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn database_commit_failure_removes_the_published_file() {
    let (_directory, service) = service();
    fixture(&service);
    let artifact = artifact(&service, None, &hex::encode(Sha256::digest(b"abc")), 3);
    service
        .database
        .transaction(|connection| {
            connection
                .batch_execute(
                    "CREATE TABLE publication_failure (
                app_id TEXT REFERENCES applications_apps(id) DEFERRABLE INITIALLY DEFERRED
            );
            CREATE FUNCTION fail_publication() RETURNS TRIGGER LANGUAGE plpgsql AS $$
            BEGIN INSERT INTO publication_failure VALUES('missing'); RETURN NEW; END $$;
            CREATE TRIGGER fail_upload_commit AFTER UPDATE OF uploaded ON artifacts_binaries
            FOR EACH ROW WHEN (NEW.uploaded) EXECUTE FUNCTION fail_publication();",
                )
                .map_err(internal)
        })
        .unwrap();
    assert_eq!(
        upload_bytes(&service, &artifact, b"abc").await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let location = service.objects.destination(
        crate::platform::objects::Kind::Binary,
        &artifact.app_id,
        artifact.release_id,
        artifact.patch_id,
        artifact.id,
    );
    assert!(!service.objects.local_path(&location).exists());
    service
        .database
        .transaction(|connection| {
            connection
                .batch_execute("DROP TRIGGER fail_upload_commit ON artifacts_binaries;")
                .map_err(internal)
        })
        .unwrap();
    assert_eq!(
        upload_bytes(&service, &artifact, b"abc").await,
        StatusCode::OK
    );
}
