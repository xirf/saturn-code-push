use crate::{
    modules::identity::api::Credentials,
    platform::error::{Result, bad},
};
use axum::http::Uri;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

pub struct Config {
    pub(crate) data_dir: PathBuf,
    pub(crate) base_url: String,
    pub(crate) admin_token: String,
    pub(crate) console_dir: PathBuf,
    pub(crate) credentials: Option<Arc<Credentials>>,
    pub bind_address: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let admin_token =
            std::env::var("ADMIN_TOKEN").map_err(|_| bad("ADMIN_TOKEN is required"))?;
        if !admin_token.starts_with("sb_api_") || admin_token.len() < 40 {
            return Err(bad("Use a random sb_api_ token of at least 40 characters"));
        }
        let base_url =
            std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let base_url = validate_origin(&base_url)?;
        let credentials = match std::env::var("ADMIN_PASSWORD_HASH") {
            Ok(hash) => {
                let username = std::env::var("ADMIN_USERNAME")
                    .map_err(|_| bad("ADMIN_USERNAME is required for dashboard login"))?;
                Some(Arc::new(Credentials::new(username, hash)?))
            }
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => return Err(bad("ADMIN_PASSWORD_HASH must be valid Unicode")),
        };
        let bind_address = std::env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:3000".into())
            .parse()
            .map_err(|_| bad("BIND_ADDRESS must be an IP address and port"))?;
        Ok(Self {
            data_dir: std::env::var_os("DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("data")),
            console_dir: std::env::var_os("CONSOLE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("console/.output/public")),
            base_url,
            admin_token,
            credentials,
            bind_address,
        })
    }
}

fn validate_origin(value: &str) -> Result<String> {
    let value = value.trim_end_matches('/');
    let uri: Uri = value
        .parse()
        .map_err(|_| bad("PUBLIC_BASE_URL must be an HTTP(S) origin"))?;
    if !matches!(uri.scheme_str(), Some("http" | "https"))
        || uri.authority().is_none()
        || uri.host().is_none_or(str::is_empty)
        || uri
            .authority()
            .is_some_and(|authority| authority.as_str().contains('@'))
        || uri
            .path_and_query()
            .is_some_and(|path| path.as_str() != "/")
        || value.contains('#')
    {
        return Err(bad(
            "PUBLIC_BASE_URL must be an HTTP(S) origin without credentials, a path, query, or fragment",
        ));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::validate_origin;

    #[test]
    fn origin_validation_rejects_ambiguous_cookie_origins() {
        for value in [
            "ftp://example.com",
            "https://example.com/path",
            "https://example.com?x=1",
            "https://example.com#fragment",
            "https://user:pass@example.com",
            "https://",
        ] {
            assert!(validate_origin(value).is_err(), "accepted {value}");
        }
        assert_eq!(
            validate_origin("https://example.com/").unwrap(),
            "https://example.com"
        );
    }
}
