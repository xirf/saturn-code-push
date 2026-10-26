use super::error::{Result, bad};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;
pub(crate) fn now() -> String {
    Utc::now().to_rfc3339()
}

pub(crate) fn uid() -> String {
    Uuid::new_v4().to_string()
}

pub(crate) fn text(input: &Value, key: &str) -> Result<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .filter(|runtime| !runtime.trim().is_empty() && runtime.len() <= 256)
        .map(str::to_owned)
        .ok_or_else(|| bad(format!("Invalid {key}")))
}
