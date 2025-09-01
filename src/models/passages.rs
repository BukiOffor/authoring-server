use diesel::{AsChangeset, Identifiable, Insertable, Queryable, QueryableByName, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

// impl Passage {
//     pub fn from(passage: PassageDto, creator: Uuid) -> Self {
//         Self {
//             id: Uuid::now_v7(),
//             stem: passage.stem,
//             topic_id: passage.topic_id,
//             subject_id: passage.subject_id,
//             created_by: creator,
//             created_at: chrono::Utc::now().naive_local(),
//             updated_at: chrono::Utc::now().naive_local(),
//             mg_id: None,
//             mg_passage_id: None,
//         }
//     }
// }
