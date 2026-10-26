use crate::platform::error::{Result, bad};
use argon2::PasswordHash;
use sha2::{Digest, Sha256};

pub(crate) struct Credentials {
    pub username: String,
    pub(in crate::modules::identity) hash: String,
    pub identity: String,
}

impl Credentials {
    pub fn new(username: String, hash: String) -> Result<Self> {
        if username.is_empty() || username.len() > 128 {
            return Err(bad("Invalid dashboard username"));
        }
        let parsed = PasswordHash::new(&hash).map_err(|_| bad("Invalid password hash"))?;
        if parsed.algorithm.as_str() != "argon2id" {
            return Err(bad("Use an Argon2id password hash"));
        }
        let identity = hex::encode(Sha256::digest(format!("{username}\0{hash}")));
        Ok(Self {
            username,
            hash,
            identity,
        })
    }
}
