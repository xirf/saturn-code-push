use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Check {
    pub app_id: String,
    pub channel: String,
    pub release_version: String,
    pub platform: String,
    pub arch: String,
    pub current_patch_number: Option<i64>,
    pub patch_number: Option<i64>,
}
