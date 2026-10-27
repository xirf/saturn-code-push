use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn declared_assets_block_promotion_until_uploaded() {
    let (_dir, s) = service();
    fixture(&s);
    let (status, patch) = call(
        &s,
        "POST",
        "/api/v1/apps/app/patches",
        json!({"release_id":2,"required_asset_platforms":["android"]}),
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = patch["id"].as_i64().unwrap();
    // Uploading code alone must never make an asset-dependent patch eligible.
    s.write(|d| {
        d.artifacts.push(Artifact {
            id: 100,
            app_id: "app".into(),
            release_id: 2,
            patch_id: Some(id),
            arch: "aarch64".into(),
            platform: "android".into(),
            hash: "a".repeat(64),
            hash_signature: None,
            size: 3,
            can_sideload: false,
            signing: None,
            upload_key: uid(),
            download_key: uid(),
            uploaded: true,
        });
        Ok(())
    })
    .unwrap();
    let (_, selected) = call(&s, "POST", "/api/v1/patches/check", json!({"app_id":"app", "release_version":"1.0.0+1", "platform":"android", "arch":"aarch64", "channel":"stable"}), false).await;
    assert_eq!(selected["patch_available"], false);
    assert_eq!(
        call(
            &s,
            "POST",
            "/api/v1/apps/app/patches/promote",
            json!({"patch_id":id,"channel_id":1}),
            true
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}
