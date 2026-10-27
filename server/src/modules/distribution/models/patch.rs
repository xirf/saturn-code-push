use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Patch {
    pub id: i64,
    pub release_id: i64,
    pub number: i64,
    pub channel_ids: Vec<i64>,
    #[serde(default)]
    pub rollout_percentages: std::collections::HashMap<String, u8>,
    pub is_rolled_back: bool,
    pub created_at: String,
}
