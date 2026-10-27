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

pub(super) fn verify_hash(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}
pub(super) fn dummy_hash() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| hash_password("invalid-user-dummy-password").expect("Argon2 hashing"))
}

pub(super) fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
