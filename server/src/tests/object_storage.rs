use super::*;
use crate::platform::objects::{Kind, S3};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn local_files_are_organized_and_legacy_objects_still_download() {
    let (_dir, s) = service();
    fixture(&s);
    let a = artifact(&s, None, &hex::encode(Sha256::digest(b"abc")), 3);
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::OK);
    let path = binary_path(&s, a.id).await;
    assert!(path.ends_with(format!("apps/app/releases/2/binaries/{}", a.id)));
    let p = patch(&s, 1);
    let b = artifact(&s, Some(p), &"a".repeat(64), 3);
    assert_eq!(upload_bytes(&s, &b, b"xyz").await, StatusCode::OK);
    assert!(
        binary_path(&s, b.id)
            .await
            .ends_with(format!("apps/app/releases/2/patches/{p}/binaries/{}", b.id))
    );
    // Simulate an older catalog: no location record and the historical flat file.
    std::fs::rename(&path, s.objects.binary(a.id)).unwrap();
    s.database
        .transaction(|c| {
            c.execute("DELETE FROM artifact_objects WHERE binary_id=$1", &[&a.id])
                .map_err(internal)?;
            Ok(())
        })
        .unwrap();
    let r = router(s.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/objects/{}", a.download_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.into_body().collect().await.unwrap().to_bytes(), "abc");
    for app in ["../evil", "/absolute", "a\\b", ".."] {
        let location = s.objects.destination(Kind::Binary, app, 2, None, 1);
        assert!(location.key.starts_with("apps/"));
        assert!(!location.key.contains(".."));
        assert!(!location.key.contains('\\'));
    }
    // Configuration metadata is restricted to account administrators.
    assert_eq!(
        call(&s, "GET", "/admin/storage", json!({}), false).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&s, "GET", "/admin/storage", json!({}), true).await.0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires disposable S3 bucket configured with STORAGE_S3_* and AWS credentials"]
async fn s3_publication_download_inspection_and_rollback() {
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};
    let (_dir, mut s) = service();
    fixture(&s);
    let remote = S3::from_env().unwrap().expect("configure test S3 bucket");
    s.objects.configure(Some(remote.clone()), true);
    let a = artifact(&s, None, &hex::encode(Sha256::digest(b"abc")), 3);
    let before = remote.keys().await.len();
    // Hash errors do not transfer any bytes to the bucket.
    assert_eq!(upload_bytes(&s, &a, b"bad").await, StatusCode::BAD_REQUEST);
    assert_eq!(remote.keys().await.len(), before);
    assert_eq!(upload_bytes(&s, &a, b"abc").await, StatusCode::OK);
    assert!(!s.objects.binary(a.id).exists());
    let location = s
        .objects
        .location(&s.database, Kind::Binary, a.id)
        .await
        .unwrap();
    assert!(location.remote.is_some());
    assert!(
        location
            .key
            .contains(&format!("apps/app/releases/2/binaries/{}/", a.id))
    );
    let r = router(s.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/objects/{}", a.download_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(r.headers()[header::CACHE_CONTROL], "no-store");
    let url = r.headers()[header::LOCATION].to_str().unwrap();
    let parsed = reqwest::Url::parse(url).unwrap();
    assert!(
        parsed
            .query_pairs()
            .any(|(k, v)| k == "X-Amz-Expires" && v == "300")
    );
    assert!(parsed.query_pairs().any(|(k, _)| k == "X-Amz-Signature"));
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let body = http
        .get(url)
        .header("Range", "bytes=1-2")
        .send()
        .await
        .unwrap();
    assert_eq!(body.status(), reqwest::StatusCode::PARTIAL_CONTENT);
    assert_eq!(body.bytes().await.unwrap(), "bc");
    // Keep remote reads while switching new uploads back to disk.
    s.objects.configure(Some(remote.clone()), false);
    assert!(
        s.objects
            .materialize(&s.database, Kind::Binary, a.id)
            .await
            .is_ok()
    );
    s.objects.configure(None, false);
    assert!(
        s.objects
            .location(&s.database, Kind::Binary, a.id)
            .await
            .is_err()
    );
    s.objects.configure(Some(remote.clone()), true);
    let missing = S3::test_bucket(format!("missing-{}", uuid::Uuid::new_v4())).unwrap();
    s.objects.configure(Some(missing), true);
    let p = patch(&s, 1);
    let retryable = artifact(&s, Some(p), &"a".repeat(64), 3);
    assert_eq!(
        upload_bytes(&s, &retryable, b"abc").await,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert!(
        !s.write(|c| Ok(c
            .artifacts
            .iter()
            .find(|a| a.id == retryable.id)
            .unwrap()
            .uploaded))
            .unwrap()
    );
    s.objects.configure(Some(remote.clone()), true);
    assert_eq!(upload_bytes(&s, &retryable, b"abc").await, StatusCode::OK);
    // Use a separate patch target for the concurrent multipart upload.
    let p = patch(&s, 2);
    let first = vec![b'x'; 6 * 1024 * 1024];
    let second = vec![b'y'; first.len()];
    let b = artifact(&s, Some(p), &"a".repeat(64), first.len() as u64);
    let before = remote.keys().await.len();
    let (x, y) = tokio::join!(upload_bytes(&s, &b, &first), upload_bytes(&s, &b, &second));
    assert_eq!([x, y].iter().filter(|x| **x == StatusCode::OK).count(), 1);
    assert_eq!(remote.keys().await.len(), before + 1);
    let obj = s
        .objects
        .materialize(&s.database, Kind::Binary, b.id)
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(&obj.path).unwrap(),
        if x == StatusCode::OK { first } else { second }
    );
    drop(obj);
    // Deferred commit failure rolls back the location and removes remote bytes.
    s.database.transaction(|c|c.batch_execute("CREATE TABLE storage_failure (app_id TEXT REFERENCES applications_apps(id) DEFERRABLE INITIALLY DEFERRED); CREATE FUNCTION fail_storage() RETURNS TRIGGER LANGUAGE plpgsql AS $$ BEGIN INSERT INTO storage_failure VALUES('missing'); RETURN NEW; END $$; CREATE TRIGGER fail_storage_commit AFTER UPDATE OF uploaded ON artifacts_bundles FOR EACH ROW WHEN (NEW.uploaded) EXECUTE FUNCTION fail_storage();").map_err(internal)).unwrap();
    s.write(|c| {
        c.patches
            .iter_mut()
            .find(|patch| patch.id == p)
            .unwrap()
            .channel_ids
            .clear();
        Ok(())
    })
    .unwrap();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("assets/readme.txt", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"remote preview").unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    let (code,registered)=call(&s,"POST",&format!("/api/v1/apps/app/patches/{p}/asset-bundle"),json!({"sha256":hex::encode(Sha256::digest(&bytes)),"size":bytes.len(),"platform":"android","signature":"A".repeat(86)+"=="}),true).await;
    assert_eq!(code, StatusCode::OK);
    let path = reqwest::Url::parse(registered["upload_url"].as_str().unwrap())
        .unwrap()
        .path()
        .to_owned();
    let upload = || {
        Request::builder()
            .method("POST")
            .uri(&path)
            .body(Body::from(bytes.clone()))
            .unwrap()
    };
    let before = remote.keys().await.len();
    let r = router(s.clone()).oneshot(upload()).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(remote.keys().await.len(), before);
    s.database
        .transaction(|c| {
            c.batch_execute("DROP TRIGGER fail_storage_commit ON artifacts_bundles")
                .map_err(internal)
        })
        .unwrap();
    assert_eq!(
        router(s.clone()).oneshot(upload()).await.unwrap().status(),
        StatusCode::OK
    );
    let bundle = s.write(|c| Ok(c.asset_bundles[0].clone())).unwrap();
    let base = format!("/api/v1/apps/app/patches/{p}/asset-bundles/{}", bundle.id);
    let (code, result) = call(
        &s,
        "GET",
        &format!("{base}/files/0/preview"),
        json!({}),
        true,
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(result["content"], "remote preview");
    let r = router(s.clone())
        .oneshot(
            Request::builder()
                .uri(format!("{base}/download"))
                .header("Authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.into_body().collect().await.unwrap().to_bytes(), bytes);
    let r = router(s.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/asset-objects/{}", bundle.download_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        http.get(r.headers()[header::LOCATION].to_str().unwrap())
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap(),
        bytes
    );
    remote.delete(&location.key).await.unwrap();
}
