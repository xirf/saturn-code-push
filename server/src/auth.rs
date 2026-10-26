use super::*;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::http::HeaderMap;

const COOKIE: &str = "saturn_session";
const SESSION_SECONDS: i64 = 8 * 60 * 60;

pub struct Credentials {
    pub username: String,
    hash: String,
    identity: String,
}
impl Credentials {
    pub fn new(username: String, hash: String) -> Result<Self> {
        if username.is_empty() || username.len() > 128 {
            return Err(bad("Invalid dashboard username"));
        }
        let parsed = PasswordHash::new(&hash).map_err(|_| bad("Invalid password hash"))?;
        if parsed.algorithm.as_str() != "argon2id" {
            return Err(bad("Use an Argon2id password hash"));
        }
        let identity = format!("{:x}", Sha256::digest(format!("{username}\0{hash}")));
        Ok(Self {
            username,
            hash,
            identity,
        })
    }
}
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes()).map_err(internal)?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(internal)
}
pub fn initialize(connection: &Connection) -> Result<()> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS dashboard_sessions(token_hash TEXT PRIMARY KEY, identity TEXT NOT NULL, expires_at INTEGER NOT NULL);").map_err(internal)
}
fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == COOKIE
                && value.len() == 72
                && value.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-'))
            .then_some(value)
        })
}
fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
pub fn same_origin(s: &Service, headers: &HeaderMap) -> bool {
    headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) == Some(s.base_url.as_str())
}
pub fn session_username(s: &Service, headers: &HeaderMap) -> Result<Option<String>> {
    let Some(credentials) = &s.dashboard else {
        return Ok(None);
    };
    let Some(token) = cookie_token(headers) else {
        return Ok(None);
    };
    let store = s.store.lock().map_err(internal)?;
    let valid: bool=store.connection.query_row("SELECT EXISTS(SELECT 1 FROM dashboard_sessions WHERE token_hash=?1 AND identity=?2 AND expires_at>?3)",
        params![token_hash(token),credentials.identity,Utc::now().timestamp()],|r|r.get(0)).map_err(internal)?;
    Ok(valid.then(|| credentials.username.clone()))
}
fn response(s: &Service, token: &str, age: i64, value: Value) -> Response {
    let cookie = format!(
        "{COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={age}{}",
        if s.base_url.starts_with("https://") {
            "; Secure"
        } else {
            ""
        }
    );
    let mut response = Json(value).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("Generated cookie"),
    );
    response
}
#[derive(Deserialize)]
pub struct Login {
    username: String,
    password: String,
}
pub async fn login(
    State(s): State<Service>,
    headers: HeaderMap,
    Json(input): Json<Login>,
) -> Result<Response> {
    if !same_origin(&s, &headers) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Request origin is not allowed".into(),
        ));
    }
    let credentials = s.dashboard.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Dashboard login has not been configured".into(),
        )
    })?;
    let allowed = {
        let mut rate = s.login_rate.lock().map_err(internal)?;
        if rate.0.elapsed().as_secs() >= 60 {
            *rate = (std::time::Instant::now(), 0)
        }
        rate.1 += 1;
        rate.1 <= 10
    };
    if !allowed {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many sign-in attempts. Try again in a minute.".into(),
        ));
    }
    if input.username.len() > 128 || input.password.is_empty() || input.password.len() > 1024 {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Incorrect username or password".into(),
        ));
    }
    let slot = s.login_slots.clone().try_acquire_owned().map_err(|_| {
        ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Sign-in is busy. Try again shortly.".into(),
        )
    })?;
    let check = credentials.clone();
    let valid = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        let parsed = PasswordHash::new(&check.hash).expect("Validated hash");
        Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed)
            .is_ok()
            && input.username == check.username
    })
    .await
    .map_err(internal)?;
    if !valid {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Incorrect username or password".into(),
        ));
    }
    let token = format!("{}{}", uid(), uid());
    let store = s.store.lock().map_err(internal)?;
    store
        .connection
        .execute(
            "DELETE FROM dashboard_sessions WHERE expires_at<=?1 OR identity!=?2",
            params![Utc::now().timestamp(), credentials.identity],
        )
        .map_err(internal)?;
    // A new login rotates an existing browser session rather than keeping it alive.
    if let Some(old) = cookie_token(&headers) {
        store
            .connection
            .execute(
                "DELETE FROM dashboard_sessions WHERE token_hash=?1",
                params![token_hash(old)],
            )
            .map_err(internal)?;
    }
    store
        .connection
        .execute(
            "INSERT INTO dashboard_sessions(token_hash,identity,expires_at) VALUES(?1,?2,?3)",
            params![
                token_hash(&token),
                credentials.identity,
                Utc::now().timestamp() + SESSION_SECONDS
            ],
        )
        .map_err(internal)?;
    Ok(response(
        &s,
        &token,
        SESSION_SECONDS,
        json!({"username":credentials.username}),
    ))
}
pub async fn session(State(s): State<Service>, headers: HeaderMap) -> Result<Json<Value>> {
    let username = session_username(&s, &headers)?
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Sign in to continue".into()))?;
    Ok(Json(json!({"username":username})))
}
pub async fn logout(State(s): State<Service>, headers: HeaderMap) -> Result<Response> {
    if !same_origin(&s, &headers) {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Request origin is not allowed".into(),
        ));
    }
    if let Some(token) = cookie_token(&headers) {
        s.store
            .lock()
            .map_err(internal)?
            .connection
            .execute(
                "DELETE FROM dashboard_sessions WHERE token_hash=?1",
                params![token_hash(token)],
            )
            .map_err(internal)?;
    }
    Ok(response(&s, "", 0, json!({})))
}
