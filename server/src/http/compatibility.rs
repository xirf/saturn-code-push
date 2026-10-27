//! Self-hosted organization responses required by the supported CLI protocol.
use crate::bootstrap::Runtime;
use axum::{Extension, Json, Router, routing::get};
use serde_json::{Value, json};

pub(super) fn routes(runtime: &Runtime) -> Router<Runtime> {
    let routes = Router::new()
        .route("/api/v1/users/me", get(current_user))
        .route("/api/v1/plan", get(plan))
        .route("/api/v1/organizations", get(organizations));
    super::policy::management(runtime, routes)
}

async fn current_user(
    Extension(actor): Extension<crate::modules::identity::api::Actor>,
) -> Json<Value> {
    Json(json!({
        "id": 1,
        "email": "operator@localhost",
        "display_name": actor.name,
        "has_active_subscription": true,
        "jwt_issuer": "saturn",
    }))
}

async fn plan() -> Json<Value> {
    Json(json!({"level": "self-hosted"}))
}

async fn organizations() -> Json<Value> {
    Json(json!({
        "organizations": [{
            "organization": {
                "id": 1,
                "name": "Saturn",
                "organization_type": "team",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            },
            "role": "owner",
        }],
    }))
}
