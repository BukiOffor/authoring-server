use std::sync::Arc;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{
    DbPool,
    error::ModuleError,
    helpers::dto::{MessageDto, user::UpdateUserDto},
};

pub fn update_user(
    user_id: String,
    payload: UpdateUserDto,
    pool: Arc<DbPool>,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let names: Vec<_>= payload.name.split(" ").collect();
    if names.len() < 2 {
        return Err(ModuleError::ParseError("Expected full name to be more than 1 word".to_string()));
    }
    let last_name = names[0].to_string();
    let first_name = names[1..].join(" ");
    diesel::update(crate::schema::user::table.filter(crate::schema::user::id.eq(user_id)))
        .set((
            crate::schema::user::first_name.eq(first_name),
            crate::schema::user::last_name.eq(last_name),
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


pub fn set_secret_password(){
    unimplemented!()
}

pub fn reset_password(){
    unimplemented!()
}