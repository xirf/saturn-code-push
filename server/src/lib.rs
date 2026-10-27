mod bootstrap;
mod config;
mod http;
mod migration;
mod modules;
mod platform;
mod sqlite_import;

pub use config::Config;
pub use platform::error::ApiError;

pub fn application(config: Config) -> std::result::Result<axum::Router, ApiError> {
    let mut runtime = bootstrap::Runtime::open_database(
        config.data_dir,
        config.base_url,
        config.admin_token,
        &config.database_url,
    )?;
    runtime.console_dir = config.console_dir;
    if let Some(credentials) = config.credentials {
        runtime
            .database
            .transaction(|c| modules::identity::api::bootstrap(c, &credentials))?;
    }
    use sha2::{Digest, Sha256};
    runtime.setup_token_digest = config
        .setup_token
        .map(|token| Sha256::digest(token.as_bytes()).into());
    let needs_setup = runtime
        .database
        .transaction(modules::identity::api::needs_setup)?;
    if needs_setup {
        tracing::info!(
            "Open {}/setup to create the first administrator. Set SETUP_TOKEN in deployment settings to enable setup.",
            runtime.base_url
        );
    }
    Ok(http::router(runtime))
}

pub fn hash_password(password: &str) -> std::result::Result<String, ApiError> {
    modules::identity::api::hash_password(password)
}

#[cfg(test)]
mod tests;

/// Import a stopped SQLite catalog into an empty PostgreSQL application catalog.
/// Keep the DATA_DIR/objects directory from the old server unchanged.
pub fn import_sqlite(
    database_url: &str,
    path: &std::path::Path,
) -> std::result::Result<(), ApiError> {
    let database = platform::database::Database::open(database_url)?;
    sqlite_import::import(&database, path)
}

/// Reset an existing account password through the local maintenance CLI.
pub fn reset_password(
    database_url: &str,
    username: &str,
    password: &str,
) -> std::result::Result<(), ApiError> {
    let database = platform::database::Database::open(database_url)?;
    database.transaction(|c| modules::identity::api::reset_password(c, username, password))
}
