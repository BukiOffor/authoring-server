use diesel::{AsChangeset, Identifiable, Insertable, Queryable, QueryableByName, Selectable};
use serde::{Deserialize, Serialize};

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
#[diesel(table_name = crate::schema::passages)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Passage {
    pub id: String,
    pub stem: String, //text of the passage
    pub topic_id: String,
    pub subject_id: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
