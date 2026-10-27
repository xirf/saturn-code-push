use crate::platform::{database::Database, error::Result, objects::ObjectStore};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

/// Application infrastructure and identity configuration; module repositories stay private.
#[derive(Clone)]
pub(crate) struct Runtime {
    pub database: Database,
    pub objects: ObjectStore,
    pub base_url: String,
    pub github: Option<Arc<crate::modules::github::App>>,
    pub setup_token_digest: Option<[u8; 32]>,
    pub legacy_token_enabled: bool,
    pub token_digest: [u8; 32],
    pub console_dir: PathBuf,
    pub login_rate: Arc<Mutex<(Instant, u32)>>,
    pub login_slots: Arc<tokio::sync::Semaphore>,
    pub rate: Arc<Mutex<(Instant, u32)>>,
}

impl Runtime {
    #[cfg(test)]
    pub fn open(root: PathBuf, base_url: String, token: String) -> Result<Self> {
        let database = tokio::task::block_in_place(|| Database::test(&root))?;
        Self::with_database(root, base_url, token, database)
    }
    fn with_database(
        root: PathBuf,
        base_url: String,
        token: String,
        database: Database,
    ) -> Result<Self> {
        let objects = ObjectStore::open(&root)?;
        Ok(Self {
            database,
            objects,
            base_url,
            github: None,
            setup_token_digest: None,
            legacy_token_enabled: !token.is_empty(),
            token_digest: Sha256::digest(token.as_bytes()).into(),
            console_dir: PathBuf::from("console/.output/public"),
            login_rate: Arc::new(Mutex::new((Instant::now(), 0))),
            login_slots: Arc::new(tokio::sync::Semaphore::new(2)),
            rate: Arc::new(Mutex::new((Instant::now(), 0))),
        })
    }
    pub fn open_database(
        root: PathBuf,
        base_url: String,
        token: String,
        database_url: &str,
    ) -> Result<Self> {
        let database = Database::open(database_url)?;
        if root.join("catalog.sqlite3").exists() {
            let imported: bool = database.transaction(|c| {
                Ok(
                    c.query_one("SELECT EXISTS(SELECT 1 FROM saturn_imports)", &[])
                        .map_err(crate::platform::error::internal)?
                        .get(0),
                )
            })?;
            if !imported {
                return Err(crate::platform::error::bad(
                    "SQLite catalog detected. Stop the old server and run --import-sqlite before starting PostgreSQL.",
                ));
            }
        }
        Self::with_database(root, base_url, token, database)
    }
}
