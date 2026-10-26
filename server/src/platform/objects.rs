use super::error::{Result, internal};
use std::path::{Path, PathBuf};

/// Object names come from numeric database IDs, never user-supplied paths.
#[derive(Clone)]
pub(crate) struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.join("objects");
        std::fs::create_dir_all(&root).map_err(internal)?;
        Ok(Self { root })
    }
    pub fn binary(&self, id: i64) -> PathBuf {
        self.root.join(id.to_string())
    }
    pub fn bundle(&self, id: i64) -> PathBuf {
        self.root.join(format!("asset-{id}"))
    }
    pub fn directory(&self) -> &Path {
        &self.root
    }
}
