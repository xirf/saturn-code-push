mod device;
mod patches;
mod releases;

use crate::{bootstrap::Runtime, http::policy};
use axum::Router;

pub(crate) fn routes(runtime: &Runtime) -> Router<Runtime> {
    let management = Router::new().nest(
        "/api/v1/apps/{app}",
        releases::routes().merge(patches::routes()),
    );
    policy::management(runtime, management).merge(device::routes(runtime))
}
