mod client;
mod http;
mod repository;
mod sync;
pub(crate) use client::App;
pub(crate) use http::routes;
pub(crate) use sync::start;

#[cfg(test)]
pub(crate) async fn reconcile_for_test(
    runtime: &crate::bootstrap::Runtime,
) -> crate::platform::error::Result<()> {
    sync::reconcile(
        &runtime.database,
        runtime.github.as_ref().expect("test app"),
    )
    .await
}
