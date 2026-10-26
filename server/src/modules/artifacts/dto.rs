use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum PreviewKind {
    Text,
    Image,
    None,
}

#[derive(Serialize)]
pub(super) struct AssetFile {
    pub index: usize,
    pub path: String,
    pub size: u64,
    pub compressed_size: u64,
    pub preview: PreviewKind,
}

#[derive(Serialize)]
pub(super) struct AssetInspection {
    pub files: Vec<AssetFile>,
    pub expanded_size: u64,
}

#[derive(Serialize)]
pub(super) struct AssetPreview {
    pub kind: PreviewKind,
    pub content: String,
}

#[derive(Deserialize)]
pub(crate) struct RegisterBundle {
    pub sha256: String,
    pub size: u64,
    pub platform: String,
}

#[derive(Deserialize)]
pub(crate) struct AssetCheck {
    pub app_id: String,
    pub release_version: String,
    pub channel: String,
    pub platform: String,
    pub patch_number: i64,
}
