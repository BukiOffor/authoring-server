use crate::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::Uuid;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    QueryableByName,
)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(User))]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub session_id: Option<String>, // this does not take care of the sujects
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub department: String,
    pub institution: String,
    pub bearer_token: String,
    pub secret: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub session_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub department: String,
    pub institution: String,
}

impl User {
    pub fn from_dto(dto: UserDto, password: String, token: String) -> User {
        Self {
            id: dto.id.into(),
            username: dto.username.unwrap_or_default(),
            email: dto.email,
            password_hash: password,
            last_name: dto.last_name,
            is_active: dto.is_active,
            session_id: dto.session_id.map(|s| s.into()),
            first_name: dto.first_name,
            created_at: dto.created_at,
            updated_at: dto.created_at,
            title: dto.title,
            department: dto.department,
            institution: dto.institution,
            bearer_token: token,
            secret: None,
        }
    }
}
