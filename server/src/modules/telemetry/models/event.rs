use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub(in crate::modules::telemetry) struct Event {
    pub received_at: String,
    pub event: Value,
}
