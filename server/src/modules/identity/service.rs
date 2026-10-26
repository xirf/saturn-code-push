use super::models::Credentials;
use crate::platform::error::{Result, internal};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub(crate) fn hash_password(password: &str) -> Result<String> {
    Argon2::default()
        .hash_password_with_salt(password.as_bytes(), Uuid::new_v4().as_bytes())
        .map(|hash| hash.to_string())
        .map_err(internal)
}

pub(super) fn verify(credentials: &Credentials, username: &str, password: &str) -> bool {
    let Ok(hash) = PasswordHash::new(&credentials.hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok()
        && username == credentials.username
}

pub(super) fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
