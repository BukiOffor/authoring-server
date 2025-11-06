use std::{env, sync::Arc};

use crate::{
    DbPool, error::{ErrorMessage, ModuleError}, helpers::{
        self,
        dto::{MessageDto, user::{ResetPasswordDto, UpdateUserDto}},
    }, models::user::UserDto
};
use diesel::*;
use reqwest::Client;

pub fn update_user(
    user_id: String,
    payload: UpdateUserDto,
    pool: Arc<DbPool>,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let names: Vec<_> = payload.name.split(" ").collect();
    if names.len() < 2 {
        return Err(ModuleError::ParseError(
            "Expected full name to be more than 1 word".to_string(),
        ));
    }
    let last_name = names[0].to_string();
    let first_name = names[1..].join(" ");
    diesel::update(crate::schema::user::table.filter(crate::schema::user::id.eq(user_id)))
        .set((
            //crate::schema::user::first_name.eq(first_name),
            //crate::schema::user::last_name.eq(last_name),
            crate::schema::user::title.eq(payload.title),
            crate::schema::user::department.eq(payload.department),
            crate::schema::user::institution.eq(payload.institution),
            crate::schema::user::phone_number.eq(payload.phone_number),
            crate::schema::user::alt_phone_number.eq(payload.alt_phone_number),
        ))
        .execute(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    Ok("User updated successfully".into())
}

pub fn fetch_user(pool: Arc<DbPool>) -> Result<UserDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let user = crate::schema::user::table
        .select(crate::models::user::User::as_select())
        .first::<crate::models::user::User>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    Ok(user.into())
}

pub async fn set_secret_password(
    pool: Arc<DbPool>,
    password: String,
    secret: String,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let user = crate::schema::user::table
        .select(crate::models::user::User::as_select())
        .first::<crate::models::user::User>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let url = env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/auth", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;

    let json = serde_json::json!({
        "email": user.email,
        "password": password
    });
    let result = client
        .post(url)
        .json(&json)
        .send()
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    if !result.status().is_success() {
        let err_message = result.text().await.unwrap_or_default();
        let server_err: ErrorMessage = serde_json::from_str(&err_message).map_err(|_| {
            ModuleError::InternalError("Could not deserialize upstream error message".into())
        })?;
        return Err(ModuleError::Error(
            "Upstream Error | ".to_string() + &server_err.message,
        ));
    }

    let set_secret_url = env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/users/secret/set", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;
    let secret_payload = serde_json::json!({
        "secret": secret
    });

    let result = client
        .post(set_secret_url)
        .json(&secret_payload)
        .send()
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    if !result.status().is_success() {
        let err_message = result.text().await.unwrap_or_default();
        let server_err: ErrorMessage = serde_json::from_str(&err_message).map_err(|_| {
            ModuleError::InternalError("Could not deserialize upstream error message".into())
        })?;
        return Err(ModuleError::Error(
            "Upstream Error | ".to_string() + &server_err.message,
        ));
    }
    conn.transaction::<_, ModuleError, _>(
        |conn: &mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<SqliteConnection>,
        >| {
            let secret_hash = helpers::password_hasher(secret)?;

            diesel::update(
                crate::schema::user::table.filter(crate::schema::user::id.eq(user.id.clone())),
            )
            .set((
                crate::schema::user::secret.eq(secret_hash),
                crate::schema::user::updated_at.eq(chrono::Local::now().naive_local()),
            ))
            .execute(conn)
            .map_err(|e| ModuleError::InternalError(e.to_string()))?;
            Ok(())
        },
    )?;
    Ok("Secret Password Set Successfully".into())
}

pub async fn reset_password(payload: ResetPasswordDto, pool: Arc<DbPool>) -> Result<MessageDto, ModuleError>{
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let url = env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/users/password/reset", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;

    let result = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    if !result.status().is_success() {
        let err_message = result.text().await.unwrap_or_default();
        let server_err: ErrorMessage = serde_json::from_str(&err_message).map_err(|_| {
            ModuleError::InternalError("Could not deserialize upstream error message".into())
        })?;
        return Err(ModuleError::Error(
            "Upstream Error | ".to_string() + &server_err.message,
        ));
    }
    conn.transaction::<_, ModuleError, _>(
        |conn: &mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<SqliteConnection>,
        >| {
            let password_hash = helpers::password_hasher(payload.new_password)?;
            diesel::update(
                crate::schema::user::table.filter(crate::schema::user::id.eq(&payload.user_id)),
            )
            .set((
                crate::schema::user::password_hash.eq(password_hash),
                crate::schema::user::updated_at.eq(chrono::Local::now().naive_local()),
            ))
            .execute(conn)
            .map_err(|e| ModuleError::InternalError(e.to_string()))?;
            Ok(())
        },
    )?;
    Ok("Password Reset Set Successfully".into())
}
