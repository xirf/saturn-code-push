use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct App {
    pub id: String,
    pub display_name: String,
    pub created_at: String,
}
