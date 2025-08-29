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
    Insertable,
    QueryableByName,
)]
#[diesel(table_name = crate::schema::topics)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Topic {
    pub id: String,
    pub subject_id: String,
    pub parent_topic_id: Option<String>,
    pub name: String,
    pub rubric: String,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub archived: bool,
}
