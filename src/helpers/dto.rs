use crate::models::item::{ItemType, Taxonomy};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageDto {
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemCreatedResponse {
    pub number_of_items_created: i64,
    pub topic_id: String,
    pub message: String,
}
impl From<&str> for MessageDto {
    fn from(value: &str) -> Self {
        MessageDto {
            message: value.into(),
        }
    }
}
pub mod auth {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthBodyDto {
        pub id: String,
        pub refresh_token: String,
        pub access_token: String,
        pub token_type: String,
    }

    impl AuthBodyDto {
        pub fn new(access_token: String, refresh_token: String, id: String) -> Self {
            Self {
                id,
                access_token,
                token_type: "Bearer Token".to_string(),
                refresh_token,
            }
        }
    }

    #[derive(Debug, Deserialize, Clone, Serialize)]
    pub struct AuthPayloadDto {
        pub email: String,
        pub password: String,
    }

    impl AuthPayloadDto {
        pub fn new(email: String, password: String) -> Self {
            Self { email, password }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct JwtPayloadDto {
        pub id: String,
    }

    impl JwtPayloadDto {
        pub fn new(id: String) -> Self {
            Self { id }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct LoginResponse {
        pub message: String,
        pub id: Uuid,
        pub roles: Vec<String>,
        pub permissions: Vec<String>,
        pub profile: UserLoginResponseDto,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct UserLoginResponseDto {
        pub email: String,
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Otp {
        pub code: String,
        pub secret: String,
        pub user_id: String,
    }
}

pub mod topics {

    use crate::models::topic::Topic;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct TopicDto {
        pub id: Uuid,
        pub subject_id: Uuid,
        pub parent_topic_id: Option<Uuid>,
        pub name: String,
        pub rubric: String,
        pub created_by: Uuid,
        pub created_at: NaiveDateTime,
        pub updated_at: NaiveDateTime,
        pub archived: bool,
    }

    impl From<TopicDto> for Topic {
        fn from(value: TopicDto) -> Self {
            Self {
                id: value.id.into(),
                subject_id: value.subject_id.into(),
                parent_topic_id: value.parent_topic_id.map(|pt| pt.into()),
                name: value.name,
                rubric: value.rubric,
                created_by: value.created_by.into(),
                created_at: value.created_at,
                updated_at: value.updated_at,
                archived: value.archived,
            }
        }
    }
}

pub mod tasks {
    use crate::models::{tasks::Tasks, tos::ToSDto};

    use super::*;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TaskMigrationDto {
        pub tasks: Vec<TaskDetails>,
        pub topics: Vec<super::topics::TopicDto>,
        pub topic_tos: Vec<ToSDto>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TaskDetails {
        pub task_id: Uuid,
        pub subject_id: Uuid,
        pub subject_name: String,
        pub topic_id: Uuid,
        pub topic_name: String,
        pub num_of_questions: i32,
        pub subject_code: String,
        pub start_date: NaiveDateTime,
        pub due_date: NaiveDateTime,
    }

    impl From<TaskDetails> for Tasks {
        fn from(value: TaskDetails) -> Self {
            Self {
                task_id: value.task_id.into(),
                subject_id: value.subject_id.into(),
                subject_name: value.subject_name,
                topic_id: value.topic_id.into(),
                topic_name: value.topic_name,
                num_of_questions: value.num_of_questions,
                subject_code: value.subject_code,
                start_date: value.start_date,
                due_date: value.due_date,
            }
        }
    }
}

pub mod topic {
    use crate::models::tos::ToS;

    use super::*;
    #[derive(Queryable, Debug)]
    pub struct FlatTopic {
        pub topic_id: String,
        pub topic_name: String,
        pub parent_topic_id: Option<String>,
        pub task_id: Option<String>,
        pub num_of_questions: Option<i32>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct TopicMetaData {
        pub id: String,
        pub name: String,
        pub num_of_questions: Option<i32>,
        pub expected_total_count: i64,
        pub task_id: Option<String>,
        pub item_type: ItemType,
        pub number_of_passages: i32,
        pub total_items_in_passage: i32,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct TopicNode {
        pub id: String,
        pub name: String,
        pub num_of_questions: Option<i32>,
        pub expected_total_count: i64,
        pub task_id: Option<String>,
        pub subtopics: Vec<TopicNode>,
    }

    impl TopicNode {
        pub fn find_subtopics(&self, topic_id: &str) -> Option<Vec<TopicNode>> {
            if self.id == topic_id {
                return Some(self.subtopics.clone());
            }
            for sub in &self.subtopics {
                if let Some(found) = sub.find_subtopics(topic_id) {
                    return Some(found);
                }
            }
            None
        }
    }
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct SubTopicWithMetadata {
        pub id: String,
        pub name: String,
        pub num_of_items_created: i32,
        pub expected_number_of_items: i64,
        pub task_id: Option<String>,
        pub item_type: ItemType,
        pub number_of_passages: i32,
        pub total_items_in_passage: i32,
    }

    impl SubTopicWithMetadata {
        pub fn from(value: TopicNode, tos: ToS, num_of_items_created: i32) -> Self {
            Self {
                id: value.id,
                name: value.name,
                num_of_items_created,
                expected_number_of_items: value.num_of_questions.unwrap_or(0) as i64,
                task_id: value.task_id,
                item_type: tos.item_type,
                number_of_passages: tos.number_of_passages,
                total_items_in_passage: tos.total_items_in_passage,
            }
        }
    }
}

pub mod items {

    use crate::{
        helpers::dto::items::display::OptionDto,
        models::{
            item::{Fetch, ItemStatus, Items},
            item_options::ItemOptions,
            passages::Passage,
        },
    };

    use super::*;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct UpdateItemStatus {
        pub status: ItemStatus,
    }

    #[derive(Debug, QueryableByName, Serialize, Deserialize)]
    pub struct ItemStats {
        #[diesel(sql_type = BigInt)]
        pub total_items: i64,
        #[diesel(sql_type = BigInt)]
        pub total_drafts: i64,
        #[diesel(sql_type = BigInt)]
        pub total_ready: i64,
        #[diesel(sql_type = BigInt)]
        pub total_published: i64,
    }

    #[derive(Debug, QueryableByName, Serialize, Deserialize)]
    pub struct ItemTotalStats {
        #[diesel(sql_type = Text)]
        pub topic_id: String,
        #[diesel(sql_type = Text)]
        pub topic_name: String,
        #[diesel(sql_type = Integer)]
        pub expected_items: i32,
        #[diesel(sql_type = BigInt)]
        pub items_in_draft: i64,
        #[diesel(sql_type = BigInt)]
        pub ready_items: i64,
        #[diesel(sql_type = BigInt)]
        pub submitted_items: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ItemWithOptions {
        pub item: Items,
        pub options: Vec<ItemOptions>,
    }

    impl ItemWithOptions {
        pub fn from(item: Items, options: Vec<ItemOptions>) -> Self {
            Self { item, options }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Options {
        pub id: String,
        pub position: i32,
        pub text: String,
        pub is_correct: bool,
    }

    /// Represents a passage, it's questions and options.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PassageDto {
        pub stem: String,
        pub rubric: String,
        pub items: Vec<CreateItemDto>,
        pub subject_id: Uuid,
        pub topic_id: Uuid,
    }

    impl PassageDto {
        /// converts the PassageDto into a PassageWithItems struct
        pub fn build(&self) -> PassageWithItems {
            let passage = self.build_passage();
            let mut items: Vec<ItemWithOptions> = self
                .items
                .clone()
                .into_iter()
                .map(|item| item.into())
                .collect();
            for item in &mut items {
                item.item.set_passage_id(passage.id.clone());
            }
            PassageWithItems { passage, items }
        }

        pub fn build_passage(&self) -> Passage {
            Passage {
                id: Uuid::now_v7().to_string(),
                rubric: Some(self.rubric.clone()),
                stem: self.stem.clone(),
                topic_id: self.topic_id.to_string(),
                subject_id: self.subject_id.to_string(),
                created_at: chrono::Utc::now().naive_local(),
                updated_at: chrono::Utc::now().naive_local(),
            }
        }
    }

    pub struct PassageWithItems {
        pub passage: Passage,
        pub items: Vec<ItemWithOptions>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateItemDto {
        pub question_type: ItemType,
        pub subject_id: Uuid,
        pub topic_id: Uuid,
        pub title: String,
        pub text: String,
        pub difficulty: i16,
        pub taxonomy: Taxonomy,
        pub options: Vec<CreateOptions>,
        pub passage_id: Option<Uuid>,
        pub submit: Option<bool>,
        pub task_id: Uuid,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateOptions {
        pub position: i32,
        pub text: String,
        pub is_correct: bool,
    }

    impl From<CreateOptions> for Options {
        fn from(value: CreateOptions) -> Self {
            Options {
                id: Uuid::now_v7().into(),
                position: value.position,
                text: value.text,
                is_correct: value.is_correct,
            }
        }
    }

    impl From<CreateItemDto> for ItemWithOptions {
        fn from(item: CreateItemDto) -> Self {
            let options = item.options.clone();
            let item = Items::from(item);
            let options = options
                .into_iter()
                .map(|option| ItemOptions::from(option.into(), item.id.clone()))
                .collect();
            ItemWithOptions::from(item, options)
        }
    }

    pub struct ViewItems {
        pub topic_id: String,
        pub subject_id: String,
    }

    pub struct Item {
        pub topic_id: String,
        pub topic_name: String,
        pub title: String,
        pub text: String,
        pub difficulty: i16,
        pub taxonomy: Taxonomy,
        pub passage_id: Option<Uuid>,
        pub task_id: Uuid,
        pub options: Vec<Options>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EditItem {
        pub item_id: String,
        pub title: String,
        pub text: String,
        pub difficulty: i16,
        pub taxonomy: Taxonomy,
        pub passage_id: Option<Uuid>,
        pub task_id: Uuid,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EditItemDto {
        pub item: EditItem,
        pub options: Vec<OptionDto>,
        pub publish: bool,
    }
    #[derive(Debug, serde::Deserialize)]
    pub struct StatusDto {
        pub status: ItemStatus,
    }
    #[derive(Debug, serde::Deserialize)]
    pub struct FetchDto {
        pub status: Fetch,
    }

    pub mod display {
        use super::*;
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct OptionDto {
            pub id: String,
            pub label: String,
            pub value: i64,
            pub is_answer: bool,
        }

        #[derive(Debug, Serialize, Clone)]

        pub struct ItemDto {
            pub id: String,
            pub title: String,
            pub text: String,
            pub question_type: String,
            pub difficulty: i16,
            pub status: String,
            pub date: NaiveDateTime,
            pub options: Vec<OptionDto>,
        }
        #[derive(Debug, Serialize, Clone)]

        pub struct PassageViewDto {
            pub id: String,
            pub rubric: String,
            pub stem: String,
            pub items: Vec<ItemDto>,
        }
        #[derive(Debug, Serialize, Clone)]

        pub struct ItemsAndPassages {
            pub passages: Vec<PassageViewDto>,
            pub items: Vec<ItemDto>,
        }
        #[derive(Debug, Serialize, Clone)]

        pub struct SubtopicDto {
            pub id: String,
            pub name: String,
            pub items: Vec<ItemsAndPassages>,
        }
        #[derive(Debug, Serialize, Clone)]

        pub struct TopicItemsDto {
            pub topic_id: String,
            pub task_id: String,
            pub items: ItemsAndPassages,
            pub subtopics: Vec<SubtopicDto>,
        }
    }
}

pub mod subject {
    use crate::helpers::dto::items::Options;

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ItemTransferDto {
        pub passage: Vec<PassageDto>,
        pub items: Vec<AcceptItemDto>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PassageDto {
        pub id: String,
        pub rubric: String,
        pub stem: String,
        pub items: Vec<AcceptItemDto>,
        pub subject_id: String,
        pub topic_id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AcceptItemDto {
        pub id: String,
        pub question_type: ItemType,
        pub subject_id: String,
        pub topic_id: String,
        pub title: String, // is the instruction for the item
        pub text: String,
        pub difficulty: i16,
        pub taxonomy: Taxonomy,
        pub options: Vec<Options>,
        pub passage_id: Option<String>,
        pub task_id: String,
    }

    #[derive(Debug, QueryableByName, Serialize, Deserialize)]
    pub struct ItemReadyStats {
        #[diesel(sql_type = Text)]
        pub topic_id: String,
        #[diesel(sql_type = Text)]
        pub topic_name: String,
        #[diesel(sql_type = Integer)]
        pub expected: i32,
        #[diesel(sql_type = BigInt)]
        pub ready: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SubjectDto {
        pub id: String,
        pub name: String,
        pub code: String,
    }
    #[derive(Debug, QueryableByName, Serialize, Deserialize)]
    pub struct SubjectDashboardDto {
        #[diesel(sql_type = Text)]
        pub id: String,
        #[diesel(sql_type = Text)]
        pub name: String,
        #[diesel(sql_type = Text)]
        pub code: String,
        #[diesel(sql_type = BigInt)]
        pub draft: i64,
        #[diesel(sql_type = BigInt)]
        pub submitted: i64,
        #[diesel(sql_type = BigInt)]
        pub total_items: i64,
    }
}

pub mod user {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateUserDto {
        pub title: String,
        pub name: String,
        pub email: String,
        pub institution: String,
        pub department: String,
        pub phone_number: String,
        pub alt_phone_number: String,
    }
}

pub mod pagination {
    use super::*;
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Pagination {
        #[serde(default = "default_page")]
        pub page: i32,
        #[serde(default = "default_page_size")]
        pub size: i32,
    }

    fn default_page() -> i32 {
        1
    }
    fn default_page_size() -> i32 {
        10
    }

    impl Pagination {
        pub fn offset(&self) -> i32 {
            (self.page - 1) * self.size
        }
    }

    #[derive(Serialize, Deserialize, Default)]
    pub struct PaginatedResult<T> {
        pub items: Vec<T>,
        pub metadata: Metadata,
    }

    #[derive(Serialize, Deserialize, Default)]
    pub struct Metadata {
        pub page: i32,
        pub size: i32,
        pub total_items: usize,
        pub num_pages: i32,
    }

    impl<T> PaginatedResult<T> {
        pub fn new(items: Vec<T>, total_items: usize, pagination: Pagination) -> Self {
            let metadata = Metadata {
                page: pagination.page,
                size: pagination.size,
                total_items,
                num_pages: (total_items as f64 / pagination.size as f64).ceil() as i32,
            };
            Self { items, metadata }
        }
    }
}
