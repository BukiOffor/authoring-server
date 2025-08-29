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
#[diesel(table_name = crate::schema::tasks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(task_id, topic_id))]
pub struct Tasks {
    pub task_id: String,
    pub subject_id: String,
    pub subject_name: String,
    pub topic_id: String,
    pub topic_name: String,
    pub num_of_questions: i32,
    pub subject_code: String,
    pub start_date: NaiveDateTime,
    pub due_date: NaiveDateTime,
}
