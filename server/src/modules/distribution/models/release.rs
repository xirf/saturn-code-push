use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Release {
    pub id: i64,
    pub app_id: String,
    pub version: String,
    pub flutter_revision: String,
    pub flutter_version: Option<String>,
    pub platforms: HashMap<String, Value>,
    pub created_at: String,
    pub updated_at: String,
}

impl Release {
    pub fn is_active(&self, platform: &str) -> bool {
        self.platforms
            .get(platform)
            .and_then(|configuration| configuration["status"].as_str())
            == Some("active")
    }
}
