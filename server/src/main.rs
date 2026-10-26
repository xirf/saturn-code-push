use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::io::AsyncWriteExt;
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};
use uuid::Uuid;

mod assets;
mod auth;

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, ApiError>;
#[derive(Debug)]
struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"message":self.1}))).into_response()
    }
}
fn bad(message: impl Into<String>) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, message.into())
}
fn missing() -> ApiError {
    ApiError(StatusCode::NOT_FOUND, "Not found".into())
}
fn conflict(message: &str) -> ApiError {
    ApiError(StatusCode::CONFLICT, message.into())
}
fn internal(error: impl std::fmt::Display) -> ApiError {
    tracing::error!("storage error: {error}");
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Storage operation failed".into(),
    )
}
fn now() -> String {
    Utc::now().to_rfc3339()
}
fn uid() -> String {
    Uuid::new_v4().to_string()
}
fn text(v: &Value, key: &str) -> Result<String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty() && s.len() <= 256)
        .map(str::to_owned)
        .ok_or_else(|| bad(format!("Invalid {key}")))
}

#[derive(Clone, Serialize, Deserialize)]
struct App {
    id: String,
    display_name: String,
    created_at: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Channel {
    id: i64,
    app_id: String,
    name: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Release {
    id: i64,
    app_id: String,
    version: String,
    flutter_revision: String,
    flutter_version: Option<String>,
    platforms: HashMap<String, Value>,
    created_at: String,
    updated_at: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Patch {
    id: i64,
    release_id: i64,
    number: i64,
    channel_ids: Vec<i64>,
    is_rolled_back: bool,
    created_at: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Artifact {
    id: i64,
    app_id: String,
    release_id: i64,
    patch_id: Option<i64>,
    arch: String,
    platform: String,
    hash: String,
    hash_signature: Option<String>,
    size: u64,
    can_sideload: bool,
    signing: Option<Value>,
    upload_key: String,
    download_key: String,
    uploaded: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct Catalog {
    sequence: i64,
    apps: Vec<App>,
    channels: Vec<Channel>,
    releases: Vec<Release>,
    patches: Vec<Patch>,
    artifacts: Vec<Artifact>,
    events: Vec<Value>,
    #[serde(default)]
    asset_bundles: Vec<assets::AssetBundleRow>,
}
impl Catalog {
    fn next(&mut self) -> i64 {
        self.sequence += 1;
        self.sequence
    }
    fn app(&self, id: &str) -> Result<&App> {
        self.apps.iter().find(|a| a.id == id).ok_or_else(missing)
    }
    fn release(&self, app: &str, id: i64) -> Result<&Release> {
        self.releases
            .iter()
            .find(|r| r.id == id && r.app_id == app)
            .ok_or_else(missing)
    }
    fn patch(&self, app: &str, id: i64) -> Result<&Patch> {
        let p = self
            .patches
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(missing)?;
        self.release(app, p.release_id)?;
        Ok(p)
    }
}
struct Store {
    connection: Connection,
    catalog: Catalog,
}
#[derive(Clone)]
struct Service {
    store: Arc<Mutex<Store>>,
    root: PathBuf,
    base_url: String,
    token: String,
    dashboard: Option<Arc<auth::Credentials>>,
    login_rate: Arc<Mutex<(std::time::Instant, u32)>>,
    login_slots: Arc<tokio::sync::Semaphore>,
    rate: Arc<Mutex<(std::time::Instant, u32)>>,
}
impl Service {
    fn open(root: PathBuf, base_url: String, token: String) -> Result<Self> {
        std::fs::create_dir_all(root.join("objects")).map_err(internal)?;
        let connection = Connection::open(root.join("catalog.sqlite3")).map_err(internal)?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS catalog(id INTEGER PRIMARY KEY CHECK(id=1), body TEXT NOT NULL);").map_err(internal)?;
        auth::initialize(&connection)?;
        let body: Option<String> = connection
            .query_row("SELECT body FROM catalog WHERE id=1", [], |row| row.get(0))
            .ok();
        let catalog = match body {
            Some(body) => serde_json::from_str(&body).map_err(internal)?,
            None => Catalog::default(),
        };
        Ok(Self {
            store: Arc::new(Mutex::new(Store {
                connection,
                catalog,
            })),
            root,
            base_url,
            token,
            dashboard: None,
            login_rate: Arc::new(Mutex::new((std::time::Instant::now(), 0))),
            login_slots: Arc::new(tokio::sync::Semaphore::new(2)),
            rate: Arc::new(Mutex::new((std::time::Instant::now(), 0))),
        })
    }
    fn read<T>(&self, f: impl FnOnce(&Catalog) -> Result<T>) -> Result<T> {
        f(&self.store.lock().map_err(internal)?.catalog)
    }
    // Copy, mutate and commit before publishing the new in-memory state. A failed request
    // or disk write cannot leave a partially mutated catalog visible to another request.
    fn write<T>(&self, f: impl FnOnce(&mut Catalog) -> Result<T>) -> Result<T> {
        let mut store = self.store.lock().map_err(internal)?;
        let mut next = store.catalog.clone();
        let result = f(&mut next)?;
        let body = serde_json::to_string(&next).map_err(internal)?;
        store
            .connection
            .execute(
                "INSERT INTO catalog(id,body) VALUES(1,?1)
            ON CONFLICT(id) DO UPDATE SET body=excluded.body",
                params![body],
            )
            .map_err(internal)?;
        store.catalog = next;
        Ok(result)
    }
    fn artifact_json(&self, a: &Artifact, upload: bool) -> Value {
        json!({"id":a.id,"release_id":a.release_id,"patch_id":a.patch_id,"arch":a.arch,
            "platform":a.platform,"hash":a.hash,"size":a.size,"can_sideload":a.can_sideload,
            "signing":a.signing,"created_at":now(),"url":format!("{}/{}/{}",self.base_url,
                if upload {"uploads"} else {"objects"}, if upload {&a.upload_key} else {&a.download_key}),
            "upload_method":"multipart"})
    }
}
async fn authorize(State(s): State<Service>, request: Request, next: Next) -> Response {
    let expected = format!("Bearer {}", s.token);
    let bearer = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|x| x.to_str().ok())
        == Some(&expected);
    if !bearer {
        match auth::session_username(&s, request.headers()) {
            Ok(Some(_)) => {
                if !matches!(
                    *request.method(),
                    axum::http::Method::GET | axum::http::Method::HEAD
                ) && !auth::same_origin(&s, request.headers())
                {
                    return ApiError(
                        StatusCode::FORBIDDEN,
                        "Request origin is not allowed".into(),
                    )
                    .into_response();
                }
            }
            Ok(None) => {
                return ApiError(StatusCode::UNAUTHORIZED, "Sign in to continue".into())
                    .into_response();
            }
            Err(e) => return e.into_response(),
        }
    }
    next.run(request).await
}
// Shorebird's Dart client sends encoded JSON strings as text/plain. Match the
// upstream wire behavior without changing multipart registrations or uploads.
async fn adapt_dart_json(mut request: Request, next: Next) -> Response {
    if request.uri().path().starts_with("/api/v1/")
        && request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("text/plain"))
    {
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }
    next.run(request).await
}
async fn throttle(State(s): State<Service>, request: Request, next: Next) -> Response {
    let allowed = match s.rate.lock() {
        Ok(mut rate) => {
            if rate.0.elapsed().as_secs() >= 1 {
                *rate = (std::time::Instant::now(), 0);
            }
            rate.1 += 1;
            rate.1 <= 100
        }
        Err(_) => false,
    };
    if !allowed {
        return ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Device request limit reached; retry later".into(),
        )
        .into_response();
    }
    next.run(request).await
}
fn router(s: Service) -> Router {
    let console =
        PathBuf::from(std::env::var("CONSOLE_DIR").unwrap_or_else(|_| "console/dist".into()));
    let management = Router::new()
        .route("/api/v1/users/me", get(|| async { Json(json!({"id":1,"email":"operator@localhost","display_name":"Saturn operator","has_active_subscription":true,"jwt_issuer":"saturn"})) }))
        .route("/api/v1/plan", get(|| async { Json(json!({"level":"self-hosted"})) }))
        .route("/api/v1/organizations", get(|| async { Json(json!({"organizations":[{"organization":{"id":1,"name":"Saturn","organization_type":"team","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},"role":"owner"}]})) }))
        .route("/api/v1/apps", get(list_apps).post(create_app))
        .route("/api/v1/organizations/{org}/apps", get(list_apps))
        .route("/api/v1/apps/{app}", get(get_app))
        .route("/api/v1/apps/{app}/channels", get(list_channels).post(create_channel))
        .route("/api/v1/apps/{app}/releases", get(list_releases).post(create_release))
        .route("/api/v1/apps/{app}/releases/{release}", axum::routing::patch(update_release))
        .route("/api/v1/apps/{app}/releases/{release}/artifacts", get(list_artifacts).post(register_release_artifact))
        .route("/api/v1/apps/{app}/releases/{release}/patches", get(list_patches))
        .route("/api/v1/apps/{app}/patches", post(create_patch))
        .route("/api/v1/apps/{app}/patches/{patch}/artifacts", post(register_patch_artifact))
        .route("/api/v1/apps/{app}/patches/promote", post(promote))
        .route("/api/v1/apps/{app}/releases/{release}/patches/{patch}/rollback", post(rollback))
        .route("/api/v1/apps/{app}/releases/{release}/patches/{patch}/rollforward", post(rollforward))
        .route("/api/v1/apps/{app}/patches/{patch}/asset-bundle", post(assets::register))
        .route("/admin/state", get(snapshot))
        .route("/admin/analytics", get(analytics))
        .route_layer(middleware::from_fn_with_state(s.clone(), authorize));
    Router::new()
        .merge(management)
        .route(
            "/auth/login",
            post(auth::login).layer(DefaultBodyLimit::max(4096)),
        )
        .route("/auth/logout", post(auth::logout))
        .route("/auth/session", get(auth::session))
        .route("/", get(console_index))
        .route(
            "/health",
            get(|| async { Json(json!({"status":"ok","service":"saturn-code-push"})) }),
        )
        .route(
            "/api/v1/patches/check",
            post(check_patch)
                .layer(DefaultBodyLimit::max(65536))
                .layer(middleware::from_fn_with_state(s.clone(), throttle)),
        )
        .route(
            "/api/v1/patches/events",
            post(event)
                .layer(DefaultBodyLimit::max(65536))
                .layer(middleware::from_fn_with_state(s.clone(), throttle)),
        )
        .nest_service("/assets", ServeDir::new(console.join("assets")))
        .nest_service("/licenses", ServeDir::new(console.join("licenses")))
        .route_service("/favicon.svg", ServeFile::new(console.join("favicon.svg")))
        .route("/api/v1/assets/check", get(assets::check)
            .layer(middleware::from_fn_with_state(s.clone(), throttle)))
        .route("/asset-uploads/{key}", post(assets::upload)
            .layer(DefaultBodyLimit::max(32 * 1024 * 1024)))
        .route("/asset-objects/{key}", get(assets::download))
        .route("/uploads/{key}", post(upload))
        .route("/objects/{key}", get(download))
        .layer(DefaultBodyLimit::max(256 * 1024 * 1024))
        .layer(middleware::from_fn(adapt_dart_json))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'; object-src 'none'"),
        ))
        .with_state(s)
}
async fn console_index() -> Response {
    let root =
        PathBuf::from(std::env::var("CONSOLE_DIR").unwrap_or_else(|_| "console/dist".into()));
    match tokio::fs::read_to_string(root.join("index.html")).await {
        Ok(html)=>Html(html).into_response(),
        Err(_)=>(StatusCode::SERVICE_UNAVAILABLE,Html("<h1>Saturn console needs a build</h1><p>Run npm ci and npm run build in console/, or start the Docker service.</p>")).into_response(),
    }
}
fn app_json(a: &App) -> Value {
    json!({"id":a.id,"app_id":a.id,"organization_id":1,"display_name":a.display_name,"created_at":a.created_at,"updated_at":a.created_at})
}
async fn list_apps(State(s): State<Service>) -> Result<Json<Value>> {
    s.read(|d| {
        Ok(Json(
            json!({"apps":d.apps.iter().map(app_json).collect::<Vec<_>>()}),
        ))
    })
}
async fn get_app(State(s): State<Service>, Path(app): Path<String>) -> Result<Json<Value>> {
    s.read(|d| Ok(Json(app_json(d.app(&app)?))))
}
async fn create_app(State(s): State<Service>, Json(v): Json<Value>) -> Result<Json<Value>> {
    let name = text(&v, "display_name")?;
    s.write(|d| {
        let a = App {
            id: uid(),
            display_name: name,
            created_at: now(),
        };
        for name in ["stable", "beta", "staging"] {
            let id = d.next();
            d.channels.push(Channel {
                id,
                app_id: a.id.clone(),
                name: name.into(),
            });
        }
        let response = app_json(&a);
        d.apps.push(a);
        Ok(Json(response))
    })
}
async fn list_channels(State(s): State<Service>, Path(app): Path<String>) -> Result<Json<Value>> {
    s.read(|d| {
        d.app(&app)?;
        Ok(Json(json!(
            d.channels
                .iter()
                .filter(|c| c.app_id == app)
                .collect::<Vec<_>>()
        )))
    })
}
async fn create_channel(
    State(s): State<Service>,
    Path(app): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let name = text(&v, "channel")?;
    s.write(|d| {
        d.app(&app)?;
        if d.channels.iter().any(|c| c.app_id == app && c.name == name) {
            return Err(conflict("Channel exists"));
        }
        let c = Channel {
            id: d.next(),
            app_id: app,
            name,
        };
        let result = json!(c);
        d.channels.push(c);
        Ok(Json(result))
    })
}
async fn list_releases(State(s): State<Service>, Path(app): Path<String>) -> Result<Json<Value>> {
    s.read(|d| { d.app(&app)?; Ok(Json(json!({"releases":d.releases.iter().rev().filter(|r| r.app_id==app).collect::<Vec<_>>()}))) })
}
async fn create_release(
    State(s): State<Service>,
    Path(app): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let version = text(&v, "version")?;
    let revision = text(&v, "flutter_revision")?;
    s.write(|d| {
        d.app(&app)?;
        if let Some(r) = d
            .releases
            .iter()
            .find(|r| r.app_id == app && r.version == version)
        {
            if r.platforms.values().any(|p| p["status"] == "active") {
                return Err(conflict(
                    "An active release with this version already exists",
                ));
            }
            return Ok(Json(json!({"release":r})));
        }
        let r = Release {
            id: d.next(),
            app_id: app,
            version,
            flutter_revision: revision,
            flutter_version: v["flutter_version"].as_str().map(str::to_owned),
            platforms: HashMap::new(),
            created_at: now(),
            updated_at: now(),
        };
        let result = json!({"release":r});
        d.releases.push(r);
        Ok(Json(result))
    })
}
async fn update_release(
    State(s): State<Service>,
    Path((app, id)): Path<(String, i64)>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let platform = text(&v, "platform")?;
    let status = text(&v, "status")?;
    if !["draft", "active"].contains(&status.as_str()) {
        return Err(bad("Invalid release status"));
    }
    s.write(|d| {
        d.release(&app, id)?;
        if status == "active"
            && !d.artifacts.iter().any(|a| {
                a.release_id == id && a.patch_id.is_none() && a.platform == platform && a.uploaded
            })
        {
            return Err(bad("No uploaded release artifacts"));
        }
        let signing = d
            .artifacts
            .iter()
            .find(|a| a.release_id == id && a.platform == platform && a.patch_id.is_none())
            .and_then(|a| a.signing.as_ref())
            .and_then(|v| v["mode"].as_str())
            .unwrap_or("unsigned")
            .to_owned();
        let r = d.releases.iter_mut().find(|r| r.id == id).unwrap();
        r.platforms
            .insert(platform, json!({"status":status,"signing_mode":signing}));
        r.updated_at = now();
        Ok(Json(json!({})))
    })
}
async fn list_artifacts(
    State(s): State<Service>,
    Path((app, id)): Path<(String, i64)>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<Value>> {
    s.read(|d| { d.release(&app,id)?; Ok(Json(json!({"artifacts":d.artifacts.iter().filter(|a| a.release_id==id && a.patch_id.is_none() && a.uploaded && q.get("arch").is_none_or(|x| *x==a.arch) && q.get("platform").is_none_or(|x| *x==a.platform)).map(|a| s.artifact_json(a,false)).collect::<Vec<_>>()}))) })
}
async fn create_patch(
    State(s): State<Service>,
    Path(app): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let id = v["release_id"]
        .as_i64()
        .ok_or_else(|| bad("Invalid release_id"))?;
    s.write(|d| {
        d.release(&app, id)?;
        let number = d
            .patches
            .iter()
            .filter(|p| p.release_id == id)
            .map(|p| p.number)
            .max()
            .unwrap_or(0)
            + 1;
        let p = Patch {
            id: d.next(),
            release_id: id,
            number,
            channel_ids: vec![],
            is_rolled_back: false,
            created_at: now(),
        };
        let result = json!(p);
        d.patches.push(p);
        Ok(Json(result))
    })
}
async fn metadata(mut multipart: Multipart) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| bad(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "file" {
            return Err(bad("Register metadata first; upload to the returned URL"));
        }
        let value = field.text().await.map_err(|e| bad(e.to_string()))?;
        if value.len() > 8192 {
            return Err(bad("Metadata too large"));
        }
        values.insert(name, value);
    }
    Ok(values)
}
fn register(
    s: &Service,
    app: String,
    release: i64,
    patch: Option<i64>,
    m: HashMap<String, String>,
) -> Result<Json<Value>> {
    let v = json!(m);
    let hash = text(&v, "hash")?;
    if hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(bad("Expected SHA-256 of the reconstructed artifact"));
    }
    let arch = text(&v, "arch")?;
    let platform = text(&v, "platform")?;
    let size = text(&v, "size")?
        .parse::<u64>()
        .map_err(|_| bad("Invalid size"))?;
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err(bad("Artifact size must be between 1 and 256 MiB"));
    }
    s.write(|d| {
        d.release(&app, release)?;
        if let Some(p) = patch {
            d.patch(&app, p)?;
        }
        if let Some(a) = d.artifacts.iter().find(|a| {
            a.release_id == release
                && a.patch_id == patch
                && a.arch == arch
                && a.platform == platform
        }) {
            if a.hash != hash || a.size != size {
                return Err(bad("An immutable artifact already has different content"));
            }
            if a.uploaded {
                return Err(conflict("Artifact already registered"));
            }
            return Ok(Json(s.artifact_json(a, true)));
        }
        let a = Artifact {
            id: d.next(),
            app_id: app,
            release_id: release,
            patch_id: patch,
            arch,
            platform,
            hash: hash.to_lowercase(),
            hash_signature: v["hash_signature"].as_str().map(str::to_owned),
            size,
            can_sideload: v["can_sideload"] == "true",
            signing: v["signing"]
                .as_str()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|_| bad("Invalid signing metadata"))?,
            upload_key: uid(),
            download_key: uid(),
            uploaded: false,
        };
        let response = s.artifact_json(&a, true);
        d.artifacts.push(a);
        Ok(Json(response))
    })
}
async fn register_release_artifact(
    State(s): State<Service>,
    Path((app, id)): Path<(String, i64)>,
    multipart: Multipart,
) -> Result<Json<Value>> {
    register(&s, app, id, None, metadata(multipart).await?)
}
async fn register_patch_artifact(
    State(s): State<Service>,
    Path((app, id)): Path<(String, i64)>,
    multipart: Multipart,
) -> Result<Json<Value>> {
    let release = s.read(|d| Ok(d.patch(&app, id)?.release_id))?;
    register(&s, app, release, Some(id), metadata(multipart).await?)
}
async fn upload(
    State(s): State<Service>,
    Path(key): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    let a = s.read(|d| {
        d.artifacts
            .iter()
            .find(|a| a.upload_key == key && !a.uploaded)
            .cloned()
            .ok_or_else(missing)
    })?;
    let tmp = s.root.join("objects").join(format!("{}.tmp", uid()));
    let outcome = async {
        let mut file = tokio::fs::File::create(&tmp).await.map_err(internal)?;
        let mut count = 0u64;
        let mut hash = Sha256::new();
        let mut found = false;
        while let Some(mut field) = multipart
            .next_field()
            .await
            .map_err(|e| bad(e.to_string()))?
        {
            if field.name() != Some("file") || found {
                return Err(bad("Expected exactly one file"));
            }
            found = true;
            while let Some(chunk) = field.chunk().await.map_err(|e| bad(e.to_string()))? {
                count += chunk.len() as u64;
                if count > a.size {
                    return Err(bad("Upload exceeds registered size"));
                }
                hash.update(&chunk);
                file.write_all(&chunk).await.map_err(internal)?;
            }
        }
        if !found || count != a.size {
            return Err(bad("Upload size does not match registration"));
        }
        if a.patch_id.is_none() && format!("{:x}", hash.finalize()) != a.hash {
            return Err(bad("Release artifact hash mismatch"));
        }
        file.sync_all().await.map_err(internal)?;
        drop(file);
        s.write(|d| {
            let row = d
                .artifacts
                .iter_mut()
                .find(|x| x.id == a.id && !x.uploaded)
                .ok_or_else(|| conflict("Upload already completed"))?;
            // Unique temporary files and the catalog lock prevent concurrent replacement.
            std::fs::rename(&tmp, s.root.join("objects").join(a.id.to_string()))
                .map_err(internal)?;
            row.uploaded = true;
            Ok(Json(json!({"uploaded":true})))
        })
    }
    .await;
    if outcome.is_err() {
        let _ = tokio::fs::remove_file(&tmp).await;
    }
    outcome
}
async fn download(
    State(s): State<Service>,
    Path(key): Path<String>,
    request: Request,
) -> Result<Response> {
    let id = s.read(|d| {
        d.artifacts
            .iter()
            .find(|a| a.download_key == key && a.uploaded)
            .map(|a| a.id)
            .ok_or_else(missing)
    })?;
    let response = ServeFile::new(s.root.join("objects").join(id.to_string()))
        .oneshot(request)
        .await
        .map_err(internal)?;
    Ok(response.map(Body::new))
}
async fn list_patches(
    State(s): State<Service>,
    Path((app, id)): Path<(String, i64)>,
) -> Result<Json<Value>> {
    s.read(|d| {
        d.release(&app,id)?;
        Ok(Json(json!({"patches":d.patches.iter().filter(|p| p.release_id==id).map(|p| {
            let mut v=json!(p);
            v["artifacts"]=json!(d.artifacts.iter().filter(|a| a.patch_id==Some(p.id) && a.uploaded).map(|a|s.artifact_json(a,false)).collect::<Vec<_>>());
            v["channel"]=json!(p.channel_ids.first().and_then(|id|d.channels.iter().find(|c|c.id==*id)).map(|c| &c.name)); v
        }).collect::<Vec<_>>()})))
    })
}
async fn promote(
    State(s): State<Service>,
    Path(app): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    let patch = v["patch_id"]
        .as_i64()
        .ok_or_else(|| bad("Invalid patch_id"))?;
    let channel = v["channel_id"]
        .as_i64()
        .ok_or_else(|| bad("Invalid channel_id"))?;
    s.write(|d| {
        let p = d.patch(&app, patch)?;
        if p.is_rolled_back {
            return Err(bad("Cannot promote a rolled-back patch"));
        }
        if !d
            .channels
            .iter()
            .any(|c| c.id == channel && c.app_id == app)
        {
            return Err(missing());
        }
        let artifacts = d
            .artifacts
            .iter()
            .filter(|a| a.patch_id == Some(patch))
            .collect::<Vec<_>>();
        if artifacts.is_empty() || artifacts.iter().any(|a| !a.uploaded) {
            return Err(bad("All registered patch artifacts must be uploaded"));
        }
        for a in artifacts {
            if !d.artifacts.iter().any(|b| {
                b.release_id == a.release_id
                    && b.patch_id.is_none()
                    && b.uploaded
                    && b.arch == a.arch
                    && b.platform == a.platform
            }) {
                return Err(bad("No matching release baseline for patch"));
            }
            if d.release(&app, a.release_id)?
                .platforms
                .get(&a.platform)
                .and_then(|v| v["status"].as_str())
                != Some("active")
            {
                return Err(bad("Release platform is not active"));
            }
        }
        let p = d.patches.iter_mut().find(|p| p.id == patch).unwrap();
        if !p.channel_ids.contains(&channel) {
            p.channel_ids.push(channel);
        }
        Ok(Json(json!({})))
    })
}
fn set_rollback(
    s: &Service,
    app: String,
    release: i64,
    patch: i64,
    value: bool,
) -> Result<Json<Value>> {
    s.write(|d| {
        if d.patch(&app, patch)?.release_id != release {
            return Err(missing());
        }
        d.patches
            .iter_mut()
            .find(|p| p.id == patch)
            .unwrap()
            .is_rolled_back = value;
        Ok(Json(json!({})))
    })
}
async fn rollback(
    State(s): State<Service>,
    Path((app, release, patch)): Path<(String, i64, i64)>,
) -> Result<Json<Value>> {
    set_rollback(&s, app, release, patch, true)
}
async fn rollforward(
    State(s): State<Service>,
    Path((app, release, patch)): Path<(String, i64, i64)>,
) -> Result<Json<Value>> {
    set_rollback(&s, app, release, patch, false)
}
#[derive(Deserialize)]
struct Check {
    app_id: String,
    channel: String,
    release_version: String,
    platform: String,
    arch: String,
    current_patch_number: Option<i64>,
    patch_number: Option<i64>,
}
fn select_patch(s: &Service, q: &Check) -> Result<Value> {
    s.read(|d| {
        let mut response=json!({"patch_available":false,"rolled_back_patch_numbers":[]});
        let Some(r)=d.releases.iter().find(|r|r.app_id==q.app_id && r.version==q.release_version) else {return Ok(response)};
        let rolled=d.patches.iter().filter(|p|p.release_id==r.id && p.is_rolled_back).map(|p|p.number).collect::<Vec<_>>();
        response["rolled_back_patch_numbers"]=json!(rolled);
        if r.platforms.get(&q.platform).and_then(|v|v["status"].as_str())!=Some("active") {return Ok(response)}
        let Some(c)=d.channels.iter().find(|c|c.app_id==q.app_id && c.name==q.channel) else {return Ok(response)};
        let current=q.current_patch_number.or(q.patch_number).unwrap_or(0);
        let selected=d.patches.iter().filter(|p|p.release_id==r.id && !p.is_rolled_back && p.channel_ids.contains(&c.id))
            .filter_map(|p| d.artifacts.iter().find(|a|a.patch_id==Some(p.id) && a.platform==q.platform && a.arch==q.arch && a.uploaded).map(|a|(p,a)))
            .max_by_key(|(p,_)|p.number);
        if let Some((p,a))=selected
            && (p.number>current || (rolled.contains(&current) && p.number!=current)) {
                response["patch_available"]=json!(true);
                response["patch"]=json!({"number":p.number,"hash":a.hash,"hash_signature":a.hash_signature,"download_url":format!("{}/objects/{}",s.base_url,a.download_key)});
        }
        Ok(response)
    })
}
async fn check_patch(State(s): State<Service>, Json(q): Json<Check>) -> Result<Json<Value>> {
    Ok(Json(select_patch(&s, &q)?))
}
async fn event(State(s): State<Service>, Json(v): Json<Value>) -> Result<Json<Value>> {
    let e = v
        .get("event")
        .filter(|e| e.is_object())
        .cloned()
        .ok_or_else(|| bad("Expected event"))?;
    let app = text(&e, "app_id")?;
    let kind = text(&e, "type")?;
    if ![
        "__patch_download__",
        "__patch_install__",
        "__patch_install_failure__",
        "__patch_update_failure__",
    ]
    .contains(&kind.as_str())
    {
        return Err(bad("Unknown event type"));
    }
    s.write(|d| {
        d.app(&app)?;
        if d.events.len() >= 10000 {
            d.events.drain(0..1000);
        }
        d.events.push(json!({"received_at":now(),"event":e}));
        Ok(Json(json!({})))
    })
}
async fn snapshot(State(s): State<Service>) -> Result<Json<Value>> {
    s.read(|d| Ok(Json(json!({"apps":d.apps,"channels":d.channels,"releases":d.releases,"patches":d.patches,
        "asset_bundles":d.asset_bundles.iter().map(|b|json!({"id":b.id,"app_id":b.app_id,"patch_id":b.patch_id,"platform":b.platform,"uploaded":b.uploaded,"size":b.size})).collect::<Vec<_>>(),
        "artifacts":d.artifacts.iter().map(|a|json!({"id":a.id,"app_id":a.app_id,"release_id":a.release_id,"patch_id":a.patch_id,"arch":a.arch,"platform":a.platform,"uploaded":a.uploaded,"size":a.size})).collect::<Vec<_>>(),"events":d.events}))))
}
async fn analytics(
    State(s): State<Service>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<Value>> {
    s.read(|d| {
        let mut counts: HashMap<String,u64> = HashMap::new();
        let mut daily: std::collections::BTreeMap<String,u64> = std::collections::BTreeMap::new();
        let mut clients = HashSet::new();
        let mut patches: std::collections::BTreeMap<String,HashMap<String,u64>> = std::collections::BTreeMap::new();
        let mut total=0;
        for record in &d.events {
            let e=&record["event"];
            if q.get("app_id").is_some_and(|id| e["app_id"].as_str()!=Some(id)) {continue}
            total+=1;
            let kind=e["type"].as_str().unwrap_or("unknown").to_owned();
            *counts.entry(kind.clone()).or_default()+=1;
            if let Some(client)=e["client_id"].as_str().filter(|v|!v.is_empty()) {clients.insert((e["app_id"].as_str().unwrap_or(""),client));}
            let day=record["received_at"].as_str().unwrap_or("").split('T').next().unwrap_or("").to_owned();
            *daily.entry(day).or_default()+=1;
            let key=format!("{} / {} / patch {}",e["app_id"].as_str().unwrap_or(""),e["release_version"].as_str().unwrap_or(""),e["patch_number"]);
            *patches.entry(key).or_default().entry(kind).or_default()+=1;
        }
        Ok(Json(json!({"event_counts":counts,"unique_reporting_installs":clients.len(),"daily_events":daily,"patch_events":patches,"retained_events":total,"retention_limit":10000,"measurement":"Client-reported events in the retained window; duplicates are counted."})))
    })
}
#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("--hash-password") {
        use std::io::Read;
        let mut password = String::new();
        std::io::stdin()
            .read_to_string(&mut password)
            .expect("Read password from stdin");
        let password = password.trim_end_matches(['\r', '\n']);
        assert!(
            (12..=1024).contains(&password.len()),
            "Password must contain 12 to 1024 bytes"
        );
        println!("{}", auth::hash_password(password).expect("Hash password"));
        return;
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let token = std::env::var("ADMIN_TOKEN").expect("ADMIN_TOKEN is required");
    assert!(
        token.starts_with("sb_api_") && token.len() >= 40,
        "Use a random sb_api_ token of at least 40 characters"
    );
    let base = std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    assert!(
        base.starts_with("https://") || base.starts_with("http://"),
        "PUBLIC_BASE_URL must be an absolute HTTP(S) URL"
    );
    let root = std::env::var("DATA_DIR").unwrap_or_else(|_| "data".into());
    let mut s = Service::open(root.into(), base.trim_end_matches('/').to_owned(), token)
        .expect("Open storage");
    if let Ok(hash) = std::env::var("ADMIN_PASSWORD_HASH") {
        let username = std::env::var("ADMIN_USERNAME")
            .expect("ADMIN_USERNAME is required for dashboard login");
        s.dashboard = Some(Arc::new(
            auth::Credentials::new(username, hash).expect("Valid dashboard credentials"),
        ));
    }
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Bind port 3000");
    println!("Saturn Code Push listening on port 3000");
    axum::serve(listener, router(s))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
}
