use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AssetBundle {
    pub id: i64,
    pub app_id: String,
    pub release_id: i64,
    pub patch_id: i64,
    pub platform: String,
    pub sha256: String,
    pub size: u64,
    pub upload_key: String,
    pub download_key: String,
    pub uploaded: bool,
}
