use crate::helpers::dto::items::Options;
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
#[diesel(table_name = crate::schema::item_options)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ItemOptions {
    pub id: String,
    pub item_id: String, // foreign key to the item
    pub label: String,   // actual option text
    pub value: i64,      // position of the option
    pub is_answer: bool,
}

impl ItemOptions {
    pub fn from(option: Options, item_id: String) -> Self {
        ItemOptions {
            id: option.id,
            item_id,
            label: option.text,
            value: option.position as i64,
            is_answer: option.is_correct,
        }
    }
}

impl From<ItemOptions> for Options {
    fn from(value: ItemOptions) -> Self {
        Self {
            id: value.id,
            position: value.value as i32,
            text: value.label,
            is_correct: value.is_answer,
        }
    }
}
