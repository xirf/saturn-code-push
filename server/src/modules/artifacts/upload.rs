use crate::platform::{
    database::Database,
    error::{Result, bad, internal},
};
use postgres::Transaction as Connection;
use sha2::{Digest, Sha256};
use std::{cell::Cell, path::PathBuf};
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;

/// The temporary file removes itself when a request fails or is cancelled.
pub(super) struct Upload {
    temporary: NamedTempFile,
    file: tokio::fs::File,
    digest: Sha256,
    received: u64,
    expected: u64,
}

impl Upload {
    pub fn new(directory: &std::path::Path, expected: u64) -> Result<Self> {
        let temporary = tempfile::Builder::new()
            .prefix("upload-")
            .suffix(".tmp")
            .tempfile_in(directory)
            .map_err(internal)?;
        let file = tokio::fs::File::from_std(temporary.reopen().map_err(internal)?);
        Ok(Self {
            temporary,
            file,
            digest: Sha256::new(),
            received: 0,
            expected,
        })
    }

    pub async fn chunk(&mut self, bytes: &[u8]) -> Result<()> {
        self.received = self
            .received
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| bad("Upload size overflow"))?;
        if self.received > self.expected {
            return Err(bad("Upload exceeds registered size"));
        }
        self.digest.update(bytes);
        self.file.write_all(bytes).await.map_err(internal)
    }

    pub async fn finish(self, expected_hash: Option<&str>) -> Result<NamedTempFile> {
        if self.received != self.expected {
            return Err(bad("Upload size does not match registration"));
        }
        if let Some(expected_hash) = expected_hash
            && hex::encode(self.digest.finalize()) != expected_hash
        {
            return Err(bad("Upload SHA-256 mismatch"));
        }
        self.file.sync_all().await.map_err(internal)?;
        drop(self.file);
        Ok(self.temporary)
    }
}

/// Publication and transaction cleanup finish on the blocking pool even if the request disconnects.
pub(super) async fn publish<F>(
    database: Database,
    temporary: NamedTempFile,
    destination: PathBuf,
    update: F,
) -> Result<()>
where
    F: FnOnce(&mut Connection<'_>) -> Result<()> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let published = Cell::new(false);
        database.transaction_with_cleanup(
            |connection| {
                // The transaction validates the one-use upload before publishing bytes.
                update(connection)?;
                // A crash before the database commit can leave a file for a pending row.
                // Replacing it is safe here: update rejected any already-published row.
                let file = temporary
                    .persist(&destination)
                    .map_err(|error| internal(error.error))?;
                published.set(true);
                file.sync_all().map_err(internal)?;
                #[cfg(unix)]
                if let Some(parent) = destination.parent() {
                    std::fs::File::open(parent)
                        .and_then(|directory| directory.sync_all())
                        .map_err(internal)?;
                }
                Ok(())
            },
            || {
                if published.get()
                    && let Err(error) = std::fs::remove_file(&destination)
                {
                    tracing::error!(%error, "Failed to remove uncommitted upload");
                }
            },
        )
    })
    .await
    .map_err(internal)?
}
