use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Artifact {
    pub id: i64,
    pub app_id: String,
    pub release_id: i64,
    pub patch_id: Option<i64>,
    pub arch: String,
    pub platform: String,
    pub hash: String,
    pub hash_signature: Option<String>,
    pub size: u64,
    pub can_sideload: bool,
    pub signing: Option<Value>,
    pub upload_key: String,
    pub download_key: String,
    pub uploaded: bool,
}
