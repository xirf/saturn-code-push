use crate::platform::error::{Result, bad};
use argon2::PasswordHash;

pub(crate) struct Credentials {
    pub username: String,
    pub(in crate::modules::identity) hash: String,
}

impl Credentials {
    pub fn new(username: String, hash: String) -> Result<Self> {
        if username.is_empty()
            || username.len() > 128
            || username.trim() != username
            || username.chars().any(char::is_control)
        {
            return Err(bad("Invalid dashboard username"));
        }
        let parsed = PasswordHash::new(&hash).map_err(|_| bad("Invalid password hash"))?;
        if parsed.algorithm.as_str() != "argon2id" {
            return Err(bad("Use an Argon2id password hash"));
        }
        Ok(Self { username, hash })
    }
}
