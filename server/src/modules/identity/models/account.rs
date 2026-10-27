use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Account {
    pub id: String,
    pub username: String,
    pub role: String,
    pub disabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Actor {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub role: String,
    pub app_id: Option<String>,
    pub permission: Option<String>,
}
impl Actor {
    pub fn account(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            name: account.username.clone(),
            kind: "account".into(),
            role: account.role.clone(),
            app_id: None,
            permission: None,
        }
    }
}
