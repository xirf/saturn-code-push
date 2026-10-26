use crate::{
    modules::identity::api::Credentials,
    platform::{database::Database, error::Result, objects::ObjectStore},
};
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
    pub token_digest: [u8; 32],
    pub console_dir: PathBuf,
    pub dashboard: Option<Arc<Credentials>>,
    pub login_rate: Arc<Mutex<(Instant, u32)>>,
    pub login_slots: Arc<tokio::sync::Semaphore>,
    pub rate: Arc<Mutex<(Instant, u32)>>,
}

impl Runtime {
    pub fn open(root: PathBuf, base_url: String, token: String) -> Result<Self> {
        let objects = ObjectStore::open(&root)?;
        let database = Database::open(&root.join("catalog.sqlite3"))?;
        Ok(Self {
            database,
            objects,
            base_url,
            token_digest: Sha256::digest(token.as_bytes()).into(),
            console_dir: PathBuf::from("console/.output/public"),
            dashboard: None,
            login_rate: Arc::new(Mutex::new((Instant::now(), 0))),
            login_slots: Arc::new(tokio::sync::Semaphore::new(2)),
            rate: Arc::new(Mutex::new((Instant::now(), 0))),
        })
    }
}
