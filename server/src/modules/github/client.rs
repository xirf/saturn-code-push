use crate::platform::error::{ApiError, Result, bad, internal};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
    sign::Signer,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) struct App {
    pub id: i64,
    pub slug: String,
    pub secret: Vec<u8>,
    key: PKey<Private>,
    client: reqwest::Client,
    pub notify: tokio::sync::Notify,
    #[cfg(test)]
    pub test_api: Option<String>,
}
pub(super) fn valid_repository(value: &str) -> bool {
    let parts: Vec<_> = value.split('/').collect();
    value.len() <= 256
        && parts.len() == 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && *p != "."
                && *p != ".."
                && p.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        })
}
impl App {
    pub fn from_env() -> Result<Option<Arc<Self>>> {
        let values = [
            "GITHUB_APP_ID",
            "GITHUB_APP_SLUG",
            "GITHUB_APP_PRIVATE_KEY_FILE",
            "GITHUB_WEBHOOK_SECRET",
        ]
        .map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()));
        if values.iter().all(Option::is_none) {
            return Ok(None);
        }
        let [Some(id), Some(slug), Some(path), Some(secret)] = values else {
            return Err(bad(
                "Configure all GitHub App variables: ID, slug, private key file, and webhook secret",
            ));
        };
        let id = id
            .parse::<i64>()
            .ok()
            .filter(|id| *id > 0)
            .ok_or_else(|| bad("Invalid GITHUB_APP_ID"))?;
        if slug.is_empty()
            || slug.len() > 100
            || !slug.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || !(32..=1024).contains(&secret.len())
        {
            return Err(bad(
                "Invalid GitHub App slug or webhook secret (minimum 32 bytes)",
            ));
        }
        let key = PKey::private_key_from_pem(
            &std::fs::read(path).map_err(|_| bad("Cannot read GITHUB_APP_PRIVATE_KEY_FILE"))?,
        )
        .map_err(|_| bad("Invalid GitHub App private key"))?;
        if key.id() != openssl::pkey::Id::RSA || key.bits() < 2048 {
            return Err(bad(
                "GitHub App requires an RSA private key of at least 2048 bits",
            ));
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(20))
            .user_agent("Saturn-Code-Push")
            .build()
            .map_err(internal)?;
        Ok(Some(Arc::new(Self {
            id,
            slug,
            secret: secret.into_bytes(),
            key,
            client,
            notify: tokio::sync::Notify::new(),
            #[cfg(test)]
            test_api: None,
        })))
    }
    fn jwt(&self) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp();
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(
                &json!({"iat":timestamp-60,"exp":timestamp+540,"iss":self.id.to_string()}),
            )
            .map_err(internal)?,
        );
        let message = format!("{header}.{payload}");
        let mut signer = Signer::new(MessageDigest::sha256(), &self.key).map_err(internal)?;
        signer.update(message.as_bytes()).map_err(internal)?;
        Ok(format!(
            "{message}.{}",
            URL_SAFE_NO_PAD.encode(signer.sign_to_vec().map_err(internal)?)
        ))
    }
    async fn json(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        #[cfg(test)]
        let base = self.test_api.as_deref().unwrap_or("https://api.github.com");
        #[cfg(not(test))]
        let base = "https://api.github.com";
        let mut request = self
            .client
            .request(method, format!("{base}{path}"))
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(body) = body {
            request = request.json(&body)
        }
        let mut response = request.send().await.map_err(|_| {
            ApiError(
                StatusCode::BAD_GATEWAY,
                "GitHub could not be reached".into(),
            )
        })?;
        if !response.status().is_success() {
            return Err(ApiError(
                StatusCode::BAD_GATEWAY,
                match response.status().as_u16() {
                    401 | 403 | 404 => "GitHub installation or repository access is unavailable",
                    429 => "GitHub rate limit reached; synchronization will retry",
                    _ => "GitHub returned an unsuccessful response",
                }
                .into(),
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| bad("Incomplete GitHub response"))?
        {
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err(bad("GitHub response is too large"));
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| bad("Invalid GitHub response"))
    }
    pub(super) async fn token(&self, installation: i64, repository: &str) -> Result<String> {
        if installation <= 0 || !valid_repository(repository) {
            return Err(bad("Invalid installation or repository"));
        }
        let jwt = self.jwt()?;
        let result=self.json(reqwest::Method::POST,&format!("/app/installations/{installation}/access_tokens"),&jwt,Some(json!({"repositories":[repository.split('/').nth(1).unwrap()],"permissions":{"actions":"read","metadata":"read"}}))).await?;
        result["token"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| bad("GitHub did not return an installation token"))
    }
    pub(super) async fn verify_installation(
        &self,
        installation: i64,
        repository: &str,
    ) -> Result<()> {
        if installation <= 0 || !valid_repository(repository) {
            return Err(bad("Use a positive installation ID and owner/repository"));
        }
        let metadata = self
            .json(
                reqwest::Method::GET,
                &format!("/app/installations/{installation}"),
                &self.jwt()?,
                None,
            )
            .await?;
        if metadata["app_id"].as_i64() != Some(self.id) || !metadata["suspended_at"].is_null() {
            return Err(bad(
                "This installation is unavailable for the configured GitHub App",
            ));
        }
        let token = self.token(installation, repository).await?;
        let repo = self
            .json(
                reqwest::Method::GET,
                &format!("/repos/{repository}"),
                &token,
                None,
            )
            .await?;
        if !repo["full_name"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(repository))
        {
            return Err(bad("Repository does not match the installation"));
        }
        Ok(())
    }
    pub(super) async fn run(
        &self,
        token: &str,
        repository: &str,
        run: i64,
        attempt: Option<i64>,
    ) -> Result<Value> {
        let suffix = attempt
            .map(|value| format!("/attempts/{value}"))
            .unwrap_or_default();
        self.json(
            reqwest::Method::GET,
            &format!("/repos/{repository}/actions/runs/{run}{suffix}"),
            token,
            None,
        )
        .await
    }
    pub(super) fn verify_signature(&self, signature: &str, body: &[u8]) -> bool {
        use subtle::ConstantTimeEq;
        let Some(hex) = signature.strip_prefix("sha256=") else {
            return false;
        };
        let Ok(supplied) = hex::decode(hex) else {
            return false;
        };
        if supplied.len() != 32 {
            return false;
        }
        let Ok(key) = PKey::hmac(&self.secret) else {
            return false;
        };
        let Ok(mut signer) = Signer::new(MessageDigest::sha256(), &key) else {
            return false;
        };
        if signer.update(body).is_err() {
            return false;
        }
        signer
            .sign_to_vec()
            .is_ok_and(|expected| bool::from(expected.ct_eq(&supplied)))
    }
    #[cfg(test)]
    pub(crate) fn for_test(api: String) -> Arc<Self> {
        Arc::new(Self {
            id: 42,
            slug: "saturn-test".into(),
            secret: b"webhook-test-secret-at-least-32-bytes".to_vec(),
            key: PKey::from_rsa(openssl::rsa::Rsa::generate(2048).unwrap()).unwrap(),
            client: reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .unwrap(),
            notify: tokio::sync::Notify::new(),
            test_api: Some(api),
        })
    }
}
