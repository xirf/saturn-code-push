use super::*;

pub(super) const TOKEN: &str = "sb_api_1234567890123456789012345678901234567890";
pub(super) const PASSWORD: &str = "correct-horse-saturn-42";

pub(super) fn service() -> (tempfile::TempDir, Service) {
    let dir = tempfile::tempdir().unwrap();
    let s = Service::open(
        dir.path().into(),
        "http://localhost:3000".into(),
        TOKEN.into(),
    )
    .unwrap();
    (dir, s)
}

pub(super) async fn call(
    s: &Service,
    method: &str,
    path: &str,
    body: Value,
    auth: bool,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if auth {
        request = request.header("Authorization", format!("Bearer {TOKEN}"));
    }
    let r = router(s.clone())
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = r.status();
    let bytes = r.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

pub(super) fn fixture(s: &Service) {
    s.write(|d| {
        d.apps.push(App {
            id: "app".into(),
            display_name: "Test".into(),
            created_at: now(),
        });
        d.channels.push(Channel {
            id: 1,
            app_id: "app".into(),
            name: "stable".into(),
        });
        d.releases.push(Release {
            id: 2,
            app_id: "app".into(),
            version: "1.0.0+1".into(),
            flutter_revision: "revision".into(),
            flutter_version: None,
            platforms: HashMap::from([("android".into(), json!({"status":"active"}))]),
            created_at: now(),
            updated_at: now(),
        });
        d.sequence = 2;
        Ok(())
    })
    .unwrap();
}

pub(super) fn query() -> Check {
    Check {
        app_id: "app".into(),
        release_version: "1.0.0+1".into(),
        channel: "stable".into(),
        platform: "android".into(),
        arch: "x86_64".into(),
        current_patch_number: None,
        patch_number: None,
        client_id: None,
    }
}

pub(super) fn artifact(s: &Service, patch: Option<i64>, hash: &str, size: u64) -> Artifact {
    s.write(|d| {
        let a = Artifact {
            id: d.next(),
            app_id: "app".into(),
            release_id: 2,
            patch_id: patch,
            arch: "x86_64".into(),
            platform: "android".into(),
            hash: hash.into(),
            hash_signature: None,
            size,
            can_sideload: false,
            signing: None,
            upload_key: uid(),
            download_key: uid(),
            uploaded: false,
        };
        d.artifacts.push(a.clone());
        Ok(a)
    })
    .unwrap()
}

pub(super) fn patch(s: &Service, number: i64) -> i64 {
    s.write(|d| {
        let id = d.next();
        d.patches.push(Patch {
            id,
            release_id: 2,
            number,
            channel_ids: vec![1],
            rollout_percentages: HashMap::new(),
            is_rolled_back: false,
            created_at: now(),
        });
        Ok(id)
    })
    .unwrap()
}

pub(super) async fn upload_bytes(s: &Service, a: &Artifact, bytes: &[u8]) -> StatusCode {
    let mut body = concat!(
        "--boundary\r\n",
        "Content-Disposition: form-data; name=\"file\"; filename=\"artifact\"\r\n",
        "Content-Type: application/octet-stream\r\n\r\n",
    )
    .as_bytes()
    .to_vec();
    body.extend(bytes);
    body.extend(b"\r\n--boundary--\r\n");
    router(s.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/uploads/{}", a.upload_key))
                .header("Content-Type", "multipart/form-data; boundary=boundary")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

pub(super) fn dashboard_service() -> (tempfile::TempDir, Service) {
    let (dir, mut s) = service();
    s.base_url = "https://saturn.test".into();
    let credentials =
        auth::Credentials::new("admin".into(), auth::hash_password(PASSWORD).unwrap()).unwrap();
    s.database
        .transaction(|c| auth::bootstrap(c, &credentials))
        .unwrap();
    (dir, s)
}

pub(super) async fn browser(
    s: &Service,
    method: &str,
    path: &str,
    body: Value,
    cookie: Option<&str>,
    origin: Option<&str>,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(cookie) = cookie {
        request = request.header("Cookie", cookie)
    }
    if let Some(origin) = origin {
        request = request.header("Origin", origin)
    }
    router(s.clone())
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

pub(super) async fn sign_in(s: &Service) -> String {
    let response = browser(
        s,
        "POST",
        "/auth/login",
        json!({"username":"admin","password":PASSWORD}),
        None,
        Some("https://saturn.test"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Max-Age=28800"));
    cookie.split(';').next().unwrap().to_owned()
}

/// Fixtures compose module APIs only. Production code has no shared catalog.
#[derive(Default, serde::Serialize)]
pub(super) struct FixtureCatalog {
    pub sequence: i64,
    pub apps: Vec<App>,
    pub channels: Vec<Channel>,
    pub releases: Vec<Release>,
    pub patches: Vec<Patch>,
    pub artifacts: Vec<Artifact>,
    pub asset_bundles: Vec<modules::artifacts::api::AssetBundle>,
    pub events: Vec<Value>,
}

impl FixtureCatalog {
    pub fn next(&mut self) -> i64 {
        self.sequence += 1;
        self.sequence
    }

    fn load(connection: &mut crate::platform::database::Connection<'_>) -> Result<Self> {
        Ok(Self {
            sequence: connection
                .query_one("SELECT value FROM saturn_sequence WHERE id=1", &[])
                .map_err(internal)?
                .try_get(0)
                .map_err(internal)?,
            apps: modules::applications::api::apps(connection)?,
            channels: modules::applications::api::channels(connection)?,
            releases: modules::distribution::api::releases(connection)?,
            patches: modules::distribution::api::patches(connection)?,
            artifacts: modules::artifacts::api::artifacts(connection)?,
            asset_bundles: modules::artifacts::api::asset_bundles(connection)?,
            events: modules::telemetry::api::events(connection)?,
        })
    }

    fn save(self, connection: &mut crate::platform::database::Connection<'_>) -> Result<()> {
        modules::applications::api::import(connection, self.apps, self.channels)?;
        modules::distribution::api::import(connection, self.releases, self.patches)?;
        modules::artifacts::api::import(connection, self.artifacts, self.asset_bundles)?;
        modules::telemetry::api::import(connection, self.events)?;
        connection
            .execute(
                "UPDATE saturn_sequence SET value=$1 WHERE id=1",
                &[&self.sequence],
            )
            .map_err(internal)?;
        Ok(())
    }
}

impl Service {
    pub(super) fn read<T>(
        &self,
        operation: impl FnOnce(&FixtureCatalog) -> Result<T>,
    ) -> Result<T> {
        self.database
            .transaction(|connection| operation(&FixtureCatalog::load(connection)?))
    }

    pub(super) fn write<T>(
        &self,
        operation: impl FnOnce(&mut FixtureCatalog) -> Result<T>,
    ) -> Result<T> {
        self.database.transaction(|connection| {
            let mut catalog = FixtureCatalog::load(connection)?;
            let result = operation(&mut catalog)?;
            catalog.save(connection)?;
            Ok(result)
        })
    }
}

pub(super) fn select_patch(service: &Service, input: &Check) -> Result<Value> {
    service.database.transaction(|connection| {
        modules::distribution::api::select_for_test(service, connection, input)
    })
}

pub(super) fn set_rollback(
    service: &Service,
    app: String,
    release: i64,
    patch: i64,
    value: bool,
) -> Result<Value> {
    service.database.transaction(|connection| {
        modules::distribution::api::rollback_for_test(connection, app, release, patch, value)
    })
}

// Resolve the recorded location rather than assuming the historical flat layout.
pub(super) async fn binary_path(s: &Service, id: i64) -> std::path::PathBuf {
    let location = s
        .objects
        .location(&s.database, crate::platform::objects::Kind::Binary, id)
        .await
        .unwrap();
    s.objects.local_path(&location)
}
