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
    Submitted,
    ModeratedLevel1Review,
    ModeratedLevel2Review,
    Approved,
    Rejected(String),
    Archived,
    Edited,
    EditedWithDifficulty,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum EditLevel {
    Full,
    Partial,
    None,
    #[default]
    Unknown,
}

impl<DB> ToSql<Text, DB> for EditLevel
where
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match *self {
            Self::Full => "Full",
            Self::Partial => "Partial",
            Self::None => "None",
            Self::Unknown => "Unknown",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<Text, DB> for EditLevel
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "Full" => Ok(Self::Full),
            "Partial" => Ok(Self::Partial),
            "None" => Ok(Self::None),
            "Unknown" => Ok(Self::Unknown),
            s => Err(format!("Unknown edit level: {}", s).into()),
        }
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
    pub stem: String, // text of question
    pub rubric: String,
    pub difficulty: i16, // 1-10 scale
    pub status: ItemStatus,
    pub created_by: String, // User.id of author
    pub reviewer_id_one: Option<String>,
    pub reviewer_id_two: Option<String>, // User.id of moderator
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub passage_id: Option<String>,
    pub used: Used,
    pub edit_level: EditLevel,
    pub mg_id: Option<String>,
    pub mg_passage_id: Option<String>,
    pub taxonomy: Taxonomy,
    pub task_id: String,
    pub count: i64,
}

//

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

#[derive(
    Debug, Clone, Serialize, Deserialize, AsExpression, Default, FromSqlRow, PartialEq, Eq,
)]
#[diesel(sql_type = Text)]
pub enum Used {
    Yes,
    #[default]
    No,
}

impl<DB> FromSql<Text, DB> for Used
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s).map_err(Into::into)
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for Used {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        let s = serde_json::to_string(self)?;
        out.set_value(s);
        Ok(serialize::IsNull::No)
    }
}

impl Items {
    pub fn set_passage_id(&mut self, passage_id: Uuid) {
        self.passage_id = Some(passage_id.to_string());
        self.updated_at = chrono::Utc::now().naive_local();
    }
}
