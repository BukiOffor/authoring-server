//use crate::schema::users;
use crate::{Deserialize, Serialize, models::item::ItemType};
//use diesel::backend::Backend;
//use diesel::deserialize::{FromSql, FromSqlRow};
//use diesel::expression::AsExpression;
use diesel::prelude::*;
//use diesel::serialize::{self, Output, ToSql};
//use diesel::sql_types::Text;

#[derive(
    Debug,
    Clone,
    Queryable,
    Identifiable,
    Serialize,
    Selectable,
    Deserialize,
    Insertable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::tos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(id))]
pub struct ToS {
    pub id: String,
    pub subject_id: String,
    pub num_of_questions: i32,
    pub start_range: i32,
    pub end_range: i32,
    pub item_type: ItemType,
    pub number_of_passages: i32,
    pub total_items_in_passage: i32,
    pub topic_id: String,
    pub sub_topic_id: String,
}

// #[derive(
//     Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Eq, Default,
// )]
// #[diesel(sql_type = Text)]
// pub enum ItemType {
//     #[default]
//     MCQ,
//     CLOZE,
//     PASSAGE,
// }

// impl<DB> ToSql<Text, DB> for ItemType
// where
//     DB: Backend,
//     str: ToSql<Text, DB>,
// {
//     fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
//         let s = match *self {
//             ItemType::MCQ => "MCQ",
//             ItemType::CLOZE => "CLOZE",
//             ItemType::PASSAGE => "PASSAGE",
//         };
//         s.to_sql(out)
//     }
// }

// impl<DB> FromSql<Text, DB> for ItemType
// where
//     DB: Backend,
//     String: FromSql<Text, DB>,
// {
//     fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
//         match String::from_sql(bytes)?.as_str() {
//             "MCQ" => Ok(ItemType::MCQ),
//             "CLOZE" => Ok(ItemType::CLOZE),
//             "PASSAGE" => Ok(ItemType::PASSAGE),
//             s => Err(format!("Unknown role: {}", s).into()),
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::tos)]
#[diesel(belongs_to(ToS))]
pub struct ToSDto {
    pub id: String,
    pub subject_id: String,
    pub num_of_questions: i32,
    pub start_range: i32,
    pub end_range: i32,
    pub item_type: ItemType,
    pub number_of_passages: i32,
    pub total_items_in_passage: i32,
    pub topic_id: String,
    pub sub_topic_id: String,
}

impl From<ToS> for ToSDto {
    fn from(tos: ToS) -> Self {
        Self {
            id: tos.id,
            subject_id: tos.subject_id,
            num_of_questions: tos.num_of_questions,
            start_range: tos.start_range,
            end_range: tos.end_range,
            item_type: tos.item_type,
            number_of_passages: tos.number_of_passages,
            total_items_in_passage: tos.total_items_in_passage,
            topic_id: tos.topic_id,
            sub_topic_id: tos.sub_topic_id,
        }
    }
}
