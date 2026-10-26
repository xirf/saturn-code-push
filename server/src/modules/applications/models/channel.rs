use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Channel {
    pub id: i64,
    pub app_id: String,
    pub name: String,
}
