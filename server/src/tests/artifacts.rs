use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn asset_inspection_requires_auth_and_checks_app_patch_bundle_ownership() {
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};

    let (_directory, runtime) = service();
    fixture(&runtime);
    let patch_id = patch(&runtime, 1);
    let bundle = runtime
        .write(|catalog| {
            let bundle = modules::artifacts::api::AssetBundle {
                id: catalog.next(),
                app_id: "app".into(),
                release_id: 2,
                patch_id,
                platform: "android".into(),
                sha256: "a".repeat(64),
                size: 1,
                upload_key: uid(),
                download_key: uid(),
                uploaded: true,
            };
            catalog.asset_bundles.push(bundle.clone());
            Ok(bundle)
        })
        .unwrap();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("assets/readme.txt", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"inspect me").unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    std::fs::write(runtime.objects.bundle(bundle.id), &bytes).unwrap();
    let (status, snapshot) = call(&runtime, "GET", "/admin/state", json!({}), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot["asset_bundles"][0]["sha256"], bundle.sha256);
    assert!(snapshot["asset_bundles"][0].get("upload_key").is_none());
    assert!(snapshot["asset_bundles"][0].get("download_key").is_none());
    let base = format!(
        "/api/v1/apps/app/patches/{patch_id}/asset-bundles/{}",
        bundle.id
    );
    for suffix in ["/files", "/files/0/preview", "/download"] {
        let path = format!("{base}{suffix}");
        let (status, _) = call(&runtime, "GET", &path, json!({}), false).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let wrong_app = path.replace("/apps/app/", "/apps/other/");
        let (status, _) = call(&runtime, "GET", &wrong_app, json!({}), true).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let wrong_patch = path.replace(&format!("/patches/{patch_id}/"), "/patches/999/");
        let (status, _) = call(&runtime, "GET", &wrong_patch, json!({}), true).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
    let (status, listing) = call(&runtime, "GET", &format!("{base}/files"), json!({}), true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listing["files"][0]["path"], "assets/readme.txt");
    let (status, preview) = call(
        &runtime,
        "GET",
        &format!("{base}/files/0/preview"),
        json!({}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["content"], "inspect me");
    let response = router(runtime.clone())
        .oneshot(
            Request::builder()
                .uri(format!("{base}/download"))
                .header("Authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/zip");
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(
        response.headers()[header::CONTENT_DISPOSITION]
            .to_str()
            .unwrap()
            .starts_with("attachment;")
    );
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .as_ref(),
        bytes
    );

    runtime
        .write(|catalog| {
            catalog.asset_bundles[0].uploaded = false;
            Ok(())
        })
        .unwrap();
    let (status, _) = call(&runtime, "GET", &format!("{base}/files"), json!({}), true).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn uploads_check_hash_size_and_are_immutable() {
    let (_dir, s) = service();
    fixture(&s);
    let a = artifact(&s, None, &hex::encode(Sha256::digest(b"abc")), 3);
    assert_eq!(upload_bytes(&s, &a, b"bad").await, StatusCode::BAD_REQUEST);
    assert_eq!(upload_bytes(&s, &a, b"abcd").await, StatusCode::BAD_REQUEST);
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::OK);
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::NOT_FOUND);
    assert_eq!(
        std::fs::read(s.objects.directory().join(a.id.to_string())).unwrap(),
        b"abc"
    );
    assert!(
        std::fs::read_dir(s.objects.directory()).unwrap().all(|p| !p
            .unwrap()
            .path()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifact_download_supports_byte_ranges() {
    let (_dir, s) = service();
    fixture(&s);
    let a = artifact(&s, None, &hex::encode(Sha256::digest(b"abcdef")), 6);
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn asset_bundle_is_scoped_to_running_patch_and_rolls_back() {
    let (_dir, s) = service();
    fixture(&s);
    s.write(|d| {
        d.patches.push(Patch {
            id: 3,
            release_id: 2,
            number: 1,
            channel_ids: vec![],
            is_rolled_back: false,
            created_at: now(),
        });
        d.sequence = 3;
        Ok(())
    })
    .unwrap();
    let bytes = b"test asset archive";
    let digest = hex::encode(Sha256::digest(bytes));
    let (status, registered) = call(
        &s,
        "POST",
        "/api/v1/apps/app/patches/3/asset-bundle",
        json!({"sha256":digest,"size":bytes.len(),"platform":"android", "signature": "A".repeat(86) + "=="}),
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
    s.write(|d| {
        d.patches
            .iter_mut()
            .find(|p| p.id == 3)
            .unwrap()
            .channel_ids = vec![1];
        Ok(())
    })
    .unwrap();
    let path = "/api/v1/assets/check?app_id=app&release_version=1.0.0%2B1&channel=stable&platform=android&patch_number=1";
    let (status, selected) = call(&s, "GET", path, json!({}), false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(selected["asset_bundle_available"], true);
    assert_eq!(selected["sha256"], digest);
    assert_eq!(selected["signature"], "A".repeat(86) + "==");
    assert_eq!(
        serde_json::from_str::<Value>(selected["manifest"].as_str().unwrap()).unwrap(),
        json!([
            "saturn-assets-v1",
            "app",
            "1.0.0+1",
            1,
            "android",
            digest,
            bytes.len()
        ])
    );
    assert_eq!(
        call(
            &s,
            "POST",
            "/api/v1/apps/app/patches/3/asset-bundle",
            json!({"sha256":digest,"size":bytes.len(),"platform":"android"}),
            true
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
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
