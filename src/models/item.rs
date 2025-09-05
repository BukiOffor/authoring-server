use crate::helpers::dto::items::{CreateItemDto, Options};
use crate::helpers::dto::subject::AcceptItemDto;
use crate::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use uuid::Uuid;

// Store UUIDs as TEXT in SQLite-backed models; convert to/from `Uuid` at the edges.

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Eq)]
#[diesel(sql_type = Text)]
pub enum ItemType {
    MultipleChoice,
    Passage,
    Cloze,
}

impl<DB> ToSql<Text, DB> for ItemType
where
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match *self {
            Self::MultipleChoice => "MultipleChoice",
            Self::Passage => "Passage",
            Self::Cloze => "Cloze",
        };
        s.to_sql(out)
    }
}

// Implement FromSql for QuestionType
impl<DB> FromSql<Text, DB> for ItemType
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "MultipleChoice" => Ok(Self::MultipleChoice),
            "Passage" => Ok(Self::Passage),
            "Cloze" => Ok(Self::Cloze),
            s => Err(format!("Unknown question type: {}", s).into()),
        }
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, Default, PartialEq, Eq,
)]
#[diesel(sql_type = Text)]
pub enum ItemStatus {
    #[default]
    Draft,
    Submitted,
    Ready,
}

impl<DB> FromSql<Text, DB> for ItemStatus
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s).map_err(Into::into)
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for ItemStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        let s = serde_json::to_string(self)?;
        out.set_value(s);
        Ok(serialize::IsNull::No)
    }
}

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
#[diesel(table_name = crate::schema::items)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Items {
    pub id: String,
    pub subject_id: String,
    pub topic_id: String,
    pub question_type: ItemType,
    pub text: String,
    pub title: String,
    pub difficulty: i16,
    pub status: ItemStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub passage_id: Option<String>,
    pub taxonomy: Taxonomy,
    pub task_id: String,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, AsExpression, Default, FromSqlRow, PartialEq, Eq,
)]
#[diesel(sql_type = Text)]
pub enum Taxonomy {
    #[default]
    Understanding,
    Remembering,
    Applying,
    Analyzing,
}

impl<DB> ToSql<Text, DB> for Taxonomy
where
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match *self {
            Self::Understanding => "Understanding",
            Self::Remembering => "Remembering",
            Self::Applying => "Applying",
            Self::Analyzing => "Analyzing",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<Text, DB> for Taxonomy
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "Understanding" => Ok(Self::Understanding),
            "Remembering" => Ok(Self::Remembering),
            "Applying" => Ok(Self::Applying),
            "Analyzing" => Ok(Self::Analyzing),
            s => Err(format!("Unknown status: {}", s).into()),
        }
    }
}

impl Items {
    pub fn set_passage_id(&mut self, passage_id: String) {
        self.passage_id = Some(passage_id);
        self.updated_at = chrono::Utc::now().naive_local();
    }
}

impl From<CreateItemDto> for Items {
    fn from(item: CreateItemDto) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            subject_id: item.subject_id.to_string(),
            topic_id: item.topic_id.to_string(),
            question_type: item.question_type,
            text: item.text,
            title: item.title,
            difficulty: item.difficulty,
            status: item
                .submit
                .map(|s| {
                    if s {
                        ItemStatus::Ready
                    } else {
                        ItemStatus::Draft
                    }
                })
                .unwrap_or_default(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            passage_id: item.passage_id.map(|p| p.to_string()),
            taxonomy: item.taxonomy,
            task_id: item.task_id.to_string(),
        }
    }
}

impl AcceptItemDto {
    pub fn from(value: Items, options: Vec<Options>) -> Self {
        Self {
            id: value.id,
            question_type: value.question_type,
            subject_id: value.subject_id,
            topic_id: value.topic_id,
            title: value.title,
            text: value.text,
            difficulty: value.difficulty,
            taxonomy: value.taxonomy,
            options,
            passage_id: value.passage_id,
            task_id: value.task_id,
        }
    }
}
