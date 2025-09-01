pub mod dto;
pub mod jwt;
pub mod querys;

use crate::{DbConn, error::ModuleError, schema};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;

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

pub fn get_current_user_task(conn: &mut DbConn) -> Result<Option<String>, ModuleError> {
    use diesel::ExpressionMethods;
    use diesel::OptionalExtension;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;

    let now = Utc::now().naive_utc();
    let tasks = schema::tasks::table
        .filter(schema::tasks::start_date.le(now))
        .filter(schema::tasks::due_date.ge(now))
        .select(schema::tasks::task_id)
        .first(conn)
        .optional()
        .map_err(ModuleError::from)?;
    Ok(tasks)
}
