use super::{App, repository};
use crate::platform::{database::Database, error::Result};
use std::sync::Arc;

pub(super) async fn reconcile(database: &Database, app: &App) -> Result<()> {
    let candidates = database.call_read(repository::candidates).await?;
    // Reuse only in-memory, short-lived tokens within this batch.
    let mut tokens = std::collections::HashMap::new();
    for item in candidates {
        let token_key = (item.installation, item.repository.clone());
        let token = if let Some(token) = tokens.get(&token_key) {
            Ok(String::clone(token))
        } else {
            match app.token(item.installation, &item.repository).await {
                Ok(token) => {
                    tokens.insert(token_key, token.clone());
                    Ok(token)
                }
                Err(error) => Err(error),
            }
        };
        let result = match token {
            Ok(token) => {
                app.run(&token, &item.repository, item.run, item.attempt)
                    .await
            }
            Err(error) => Err(error),
        };
        let failed = result.is_err();
        database
            .call(move |c| match result {
                Ok(run) => repository::save(c, &item, &run),
                Err(error) => repository::error(c, &item, &error.1),
            })
            .await?;
        if failed {
            break;
        }
    }
    Ok(())
}
pub(crate) fn start(database: Database, app: &Arc<App>) -> Result<()> {
    let weak = Arc::downgrade(app);
    let handle = tokio::runtime::Handle::try_current().map_err(crate::platform::error::internal)?;
    handle.spawn(async move {
        loop {
            let Some(app)=weak.upgrade() else {break};
            if let Err(error)=reconcile(&database,&app).await {tracing::warn!(status=%error.0,"GitHub reconciliation failed; retrying later")}
            tokio::select!{_=app.notify.notified()=>{},_=tokio::time::sleep(std::time::Duration::from_secs(60))=>{}}
            if Arc::strong_count(&app)==1 {break}
        }
    });
    Ok(())
}
