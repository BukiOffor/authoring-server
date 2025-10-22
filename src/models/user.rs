use crate::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use diesel::prelude::*;

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
    pub phone_number: Option<String>,
    pub alt_phone_number: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDto {
    pub id: String,
    pub username: Option<String>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub session_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub department: String,
    pub institution: String,
    pub phone_number: Option<String>,
    pub alt_phone_number: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserWithTask {
    pub user: UserDto,
    pub task_id: String,
}

impl User {
    pub fn from_dto(dto: UserDto, password: String) -> User {
        Self {
            id: dto.id,
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
            bearer_token: String::new(),
            secret: None,
            phone_number: dto.phone_number,
            alt_phone_number: dto.alt_phone_number,
        }
    }
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.into(),
            username: Some(user.username),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_active: user.is_active,
            session_id: user.session_id.map(|s| s.into()),
            created_at: user.created_at,
            updated_at: user.updated_at,
            title: user.title,
            department: user.department,
            institution: user.institution,
            phone_number: user.phone_number,
            alt_phone_number: user.alt_phone_number,
        }
    }
}
