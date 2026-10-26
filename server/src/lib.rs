mod bootstrap;
mod config;
mod http;
mod migration;
mod modules;
mod platform;

pub use config::Config;
pub use platform::error::ApiError;

pub fn application(config: Config) -> std::result::Result<axum::Router, ApiError> {
    let mut runtime =
        bootstrap::Runtime::open(config.data_dir, config.base_url, config.admin_token)?;
    runtime.console_dir = config.console_dir;
    runtime.dashboard = config.credentials;
    Ok(http::router(runtime))
}

pub fn hash_password(password: &str) -> std::result::Result<String, ApiError> {
    modules::identity::api::hash_password(password)
}

#[cfg(test)]
mod tests;
