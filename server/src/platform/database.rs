use super::error::{Result, internal};
use postgres::Transaction;
use postgres_native_tls::MakeTlsConnector;
use r2d2_postgres::PostgresConnectionManager;

pub(crate) type Connection<'a> = Transaction<'a>;
#[derive(Clone)]
pub(crate) struct AuditContext {
    pub actor: crate::modules::identity::api::Actor,
    pub action: String,
    pub resource: String,
    pub app_id: Option<String>,
    pub write: bool,
}
tokio::task_local! { pub(crate) static AUDIT_CONTEXT: AuditContext; }

struct Manager(PostgresConnectionManager<MakeTlsConnector>);
struct ManagedConnection(Option<postgres::Client>);
impl std::ops::Deref for ManagedConnection {
    type Target = postgres::Client;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("live connection")
    }
}
impl std::ops::DerefMut for ManagedConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("live connection")
    }
}
impl r2d2::ManageConnection for Manager {
    type Connection = ManagedConnection;
    type Error = postgres::Error;
    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        r2d2::ManageConnection::connect(&self.0).map(|client| ManagedConnection(Some(client)))
    }
    fn is_valid(&self, connection: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        r2d2::ManageConnection::is_valid(&self.0, connection)
    }
    fn has_broken(&self, connection: &mut Self::Connection) -> bool {
        connection
            .0
            .as_ref()
            .is_none_or(postgres::Client::is_closed)
    }
}

/// Parameters remain bound values, including optional values; never interpolate SQL.
#[macro_export]
macro_rules! sql_params {
    ($($value:expr),* $(,)?) => { &[$(&$value as &(dyn postgres::types::ToSql + Sync)),*] };
}

#[derive(Clone)]
pub(crate) struct Database(std::sync::Arc<PoolHolder>);
struct PoolHolder {
    pool: Option<r2d2::Pool<Manager>>,
}
impl Drop for PoolHolder {
    fn drop(&mut self) {
        // postgres::Client owns a synchronous Tokio runtime, including during close.
        // Closing a pool on an async executor thread would panic.
        if let Some(pool) = self.pool.take() {
            let _ = std::thread::spawn(move || drop(pool)).join();
        }
    }
}

/// Keep the publication lock through rollback and filesystem cleanup, not just COMMIT.
struct LockedConnection(Option<r2d2::PooledConnection<Manager>>);
impl LockedConnection {
    fn acquire(pool: &r2d2::Pool<Manager>) -> Result<Self> {
        let mut connection = pool.get().map_err(internal)?;
        connection
            .query_one(
                "SELECT pg_advisory_lock(hashtext(current_schema()),1396790350)",
                &[],
            )
            .map_err(internal)?;
        Ok(Self(Some(connection)))
    }
}
impl Drop for LockedConnection {
    fn drop(&mut self) {
        if let Some(mut connection) = self.0.take()
            && connection
                .query_one(
                    "SELECT pg_advisory_unlock(hashtext(current_schema()),1396790350)",
                    &[],
                )
                .is_err()
        {
            // A connection with uncertain lock state must never return to the pool.
            // Taking the actual client marks this wrapper broken for r2d2.
            drop(connection.0.take());
        }
    }
}

impl Database {
    pub fn open(url: &str) -> Result<Self> {
        let config = url
            .parse::<postgres::Config>()
            .map_err(|_| internal("Invalid DATABASE_URL"))?;
        Self::connect(config)
    }
    fn connect(config: postgres::Config) -> Result<Self> {
        let tls = native_tls::TlsConnector::builder()
            .build()
            .map_err(internal)?;
        let manager = Manager(PostgresConnectionManager::new(
            config,
            MakeTlsConnector::new(tls),
        ));
        let pool = r2d2::Pool::builder()
            .max_size(8)
            .min_idle(Some(1))
            .connection_timeout(std::time::Duration::from_secs(10))
            .build(manager)
            .map_err(internal)?;
        let database = Self(std::sync::Arc::new(PoolHolder { pool: Some(pool) }));
        database.transaction(crate::migration::run)?;
        Ok(database)
    }
    #[cfg(test)]
    pub fn test(root: &std::path::Path) -> Result<Self> {
        use sha2::{Digest, Sha256};
        let url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must point to a disposable PostgreSQL database");
        let mut config = url.parse::<postgres::Config>().map_err(internal)?;
        let name = format!(
            "test_{}",
            hex::encode(Sha256::digest(root.to_string_lossy().as_bytes()))
        );
        let tls = native_tls::TlsConnector::new().map_err(internal)?;
        let mut client = config
            .connect(MakeTlsConnector::new(tls))
            .map_err(internal)?;
        client
            .batch_execute(&format!("CREATE SCHEMA IF NOT EXISTS {name}"))
            .map_err(internal)?;
        config.options(&format!("-c search_path={name}"));
        Self::connect(config)
    }
    pub fn transaction<T>(
        &self,
        operation: impl FnOnce(&mut Connection<'_>) -> Result<T>,
    ) -> Result<T> {
        tokio::task::block_in_place(|| self.transaction_with_cleanup(operation, || {}))
    }
    pub fn transaction_with_cleanup<T>(
        &self,
        operation: impl FnOnce(&mut Connection<'_>) -> Result<T>,
        on_failure: impl FnOnce(),
    ) -> Result<T> {
        let mut locked = LockedConnection::acquire(self.0.pool.as_ref().expect("live pool"))?;
        let client = locked.0.as_mut().expect("locked connection");
        let mut cleanup_allowed = true;
        let result = (|| {
            let mut transaction = client.transaction().map_err(internal)?;
            let output = operation(&mut transaction)?;
            match transaction.commit() {
                Ok(()) => Ok(output),
                Err(error) => {
                    // A transport error can hide a successful COMMIT. Keep the file:
                    // a pending row can retry it, and a committed row still has its bytes.
                    cleanup_allowed = error.as_db_error().is_some();
                    Err(internal(error))
                }
            }
        })();
        if result.is_err() && cleanup_allowed {
            on_failure();
        }
        result
    }
    pub async fn call_read<T, F>(&self, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection<'_>) -> Result<T> + Send + 'static,
    {
        if let Ok(mut context) = AUDIT_CONTEXT.try_with(Clone::clone) {
            context.write = false;
            AUDIT_CONTEXT.scope(context, self.call(operation)).await
        } else {
            self.call(operation).await
        }
    }
    pub async fn call<T, F>(&self, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection<'_>) -> Result<T> + Send + 'static,
    {
        let database = self.clone();
        let audit = AUDIT_CONTEXT.try_with(Clone::clone).ok();
        tokio::task::spawn_blocking(move || {
            database.transaction(|c| {
                // Recheck permission under the transaction lock. Disable/revoke takes effect
                // even when authentication finished before a concurrent administrative change.
                if let Some(context) = &audit {
                    crate::modules::identity::api::validate_actor(c, context)?;
                }
                let output = operation(c)?;
                if let Some(context) = audit.filter(|a| a.write) {
                    crate::modules::identity::api::append_audit(c, &context)?;
                }
                Ok(output)
            })
        })
        .await
        .map_err(internal)?
    }
}
pub(crate) fn next_id(connection: &mut Connection<'_>) -> Result<i64> {
    connection
        .query_one(
            "UPDATE saturn_sequence SET value=value+1 WHERE id=1 RETURNING value",
            &[],
        )
        .map_err(internal)?
        .try_get(0)
        .map_err(internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_cleanup_keeps_the_cross_process_publication_lock() {
        let directory = tempfile::tempdir().unwrap();
        let database = Database::test(directory.path()).unwrap();
        let result: Result<()> = database.transaction_with_cleanup(
            |_| Err(crate::platform::error::bad("Simulate failed publication")),
            || {
                let mut other = database.0.pool.as_ref().unwrap().get().unwrap();
                let available: bool = other
                    .query_one(
                        "SELECT pg_try_advisory_lock(hashtext(current_schema()),1396790350)",
                        &[],
                    )
                    .unwrap()
                    .get(0);
                if available {
                    other
                        .query_one(
                            "SELECT pg_advisory_unlock(hashtext(current_schema()),1396790350)",
                            &[],
                        )
                        .unwrap();
                }
                assert!(
                    !available,
                    "A second publisher acquired the lock before file cleanup finished"
                );
            },
        );
        assert!(result.is_err());
        // Cleanup released the session lock and the connection is reusable.
        database.transaction(|_| Ok(())).unwrap();
    }
}
