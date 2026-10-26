use super::error::{Result, internal};
use rusqlite::{Connection, TransactionBehavior};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

/// One connection serializes transactions. Never hold the lock across an await.
#[derive(Clone)]
pub(crate) struct Database(Arc<Mutex<Connection>>);

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path).map_err(internal)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(internal)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
            )
            .map_err(internal)?;
        crate::migration::run(&mut connection)?;
        Ok(Self(Arc::new(Mutex::new(connection))))
    }

    /// Transactions roll back on a use-case error, including cross-module operations.
    pub fn transaction<T>(&self, operation: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        self.transaction_with_cleanup(operation, || {})
    }

    /// Compensating filesystem cleanup must finish before another writer acquires the lock.
    pub fn transaction_with_cleanup<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T>,
        on_failure: impl FnOnce(),
    ) -> Result<T> {
        let mut connection = self.0.lock().map_err(internal)?;
        let result = (|| {
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(internal)?;
            let output = operation(&transaction)?;
            transaction.commit().map_err(internal)?;
            Ok(output)
        })();
        if result.is_err() {
            on_failure();
        }
        result
    }

    pub async fn call<T, F>(&self, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
    {
        let database = self.clone();
        tokio::task::spawn_blocking(move || database.transaction(operation))
            .await
            .map_err(internal)?
    }
}

pub(crate) fn next_id(connection: &Connection) -> Result<i64> {
    connection
        .query_row(
            "UPDATE saturn_sequence SET value=value+1 WHERE id=1 RETURNING value",
            [],
            |row| row.get(0),
        )
        .map_err(internal)
}
