use super::*;

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
