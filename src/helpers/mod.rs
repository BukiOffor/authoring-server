pub mod dto;
pub mod jwt;
pub mod querys;

use crate::error::ModuleError;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub fn password_verfier(password: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(hash).unwrap();
    argon2.verify_password(password.as_bytes(), &hash).is_ok()
}

pub fn password_hasher(password: String) -> Result<String, ModuleError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    Ok(password_hash.to_string())
}
