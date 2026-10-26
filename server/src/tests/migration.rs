use super::*;
use rusqlite::{Connection, params};

fn legacy_database(directory: &std::path::Path, body: &str) -> Connection {
    let connection = Connection::open(directory.join("catalog.sqlite3")).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE catalog (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            body TEXT NOT NULL
        );",
        )
        .unwrap();
    connection
        .execute("INSERT INTO catalog VALUES(1, ?1)", [body])
        .unwrap();
    connection
}

#[tokio::test]
async fn legacy_migration_preserves_records_objects_sessions_and_sequence() {
    let (_original_directory, original) = dashboard_service();
    fixture(&original);
    let patch_id = patch(&original, 1);
    let binary = artifact(&original, Some(patch_id), &"a".repeat(64), 3);
    assert_eq!(
        upload_bytes(&original, &binary, b"abc").await,
        StatusCode::OK
    );
    let cookie = sign_in(&original).await;
    original
        .write(|catalog| {
            let bundle = modules::artifacts::api::AssetBundle {
                id: catalog.next(),
                app_id: "app".into(),
                release_id: 2,
                patch_id,
                platform: "android".into(),
                sha256: hex::encode(Sha256::digest(b"bundle")),
                size: 6,
                upload_key: uid(),
                download_key: uid(),
                uploaded: true,
            };
            catalog.asset_bundles.push(bundle);
            catalog.events.push(json!({
                "received_at": now(),
                "event": {"app_id": "app", "type": "__patch_download__"},
            }));
            Ok(())
        })
        .unwrap();
    let body = original
        .read(|catalog| serde_json::to_value(catalog).map_err(internal))
        .unwrap();
    let session: (String, String, i64) = original
        .database
        .transaction(|connection| {
            connection
                .query_row(
                    "SELECT token_hash, identity, expires_at FROM dashboard_sessions",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(internal)
        })
        .unwrap();

    let migrated_directory = tempfile::tempdir().unwrap();
    let connection = legacy_database(migrated_directory.path(), &body.to_string());
    auth::initialize(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO dashboard_sessions VALUES(?1, ?2, ?3)",
            params![session.0, session.1, session.2],
        )
        .unwrap();
    drop(connection);
    let objects = migrated_directory.path().join("objects");
    std::fs::create_dir(&objects).unwrap();
    std::fs::copy(
        original.objects.binary(binary.id),
        objects.join(binary.id.to_string()),
    )
    .unwrap();
    let bundle_id = body["asset_bundles"][0]["id"].as_i64().unwrap();
    std::fs::write(objects.join(format!("asset-{bundle_id}")), b"bundle").unwrap();

    let mut migrated = Service::open(
        migrated_directory.path().into(),
        original.base_url.clone(),
        TOKEN.into(),
    )
    .unwrap();
    migrated.dashboard = original.dashboard.clone();
    let after = migrated
        .read(|catalog| serde_json::to_value(catalog).map_err(internal))
        .unwrap();
    assert_eq!(after, body);
    assert_eq!(
        browser(
            &migrated,
            "GET",
            "/auth/session",
            json!({}),
            Some(&cookie),
            None
        )
        .await
        .status(),
        StatusCode::OK,
    );
    let response = router(migrated.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/objects/{}", binary.download_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "abc"
    );
    migrated
        .database
        .transaction(|connection| {
            let old_writer_exists: bool = connection
                .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='catalog' AND type='table')",
                    [],
                    |row| row.get(0),
                )
                .map_err(internal)?;
            assert!(!old_writer_exists);
            // Simulate the old binary's startup: its CREATE is harmless and its
            // catalog deserialization fails before any request can mutate files.
            connection.execute_batch(
                "CREATE TABLE IF NOT EXISTS catalog(id INTEGER PRIMARY KEY, body TEXT NOT NULL);",
            ).map_err(internal)?;
            let guard: String = connection
                .query_row("SELECT body FROM catalog WHERE id=1", [], |row| row.get(0))
                .map_err(internal)?;
            assert!(serde_json::from_str::<Value>(&guard).is_err());
            let next = crate::platform::database::next_id(connection)?;
            assert!(next > bundle_id);
            Ok(())
        })
        .unwrap();
    drop(migrated);
    Service::open(
        migrated_directory.path().into(),
        original.base_url,
        TOKEN.into(),
    )
    .unwrap();
}

#[test]
fn corrupt_legacy_catalog_rolls_back_the_entire_migration() {
    let directory = tempfile::tempdir().unwrap();
    drop(legacy_database(directory.path(), "invalid JSON"));
    assert!(
        Service::open(
            directory.path().into(),
            "http://localhost:3000".into(),
            TOKEN.into()
        )
        .is_err()
    );
    let connection = Connection::open(directory.path().join("catalog.sqlite3")).unwrap();
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    let body: String = connection
        .query_row("SELECT body FROM catalog", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 0);
    assert_eq!(body, "invalid JSON");
    let new_table_exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='applications_apps')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!new_table_exists);
}

#[test]
fn newer_schema_is_rejected_without_modification() {
    let directory = tempfile::tempdir().unwrap();
    let connection = Connection::open(directory.path().join("catalog.sqlite3")).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    assert!(
        Service::open(
            directory.path().into(),
            "http://localhost:3000".into(),
            TOKEN.into()
        )
        .is_err()
    );
}
