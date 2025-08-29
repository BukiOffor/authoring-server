use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod auth {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthBodyDto {
        pub id: Uuid,
        pub refresh_token: String,
        pub access_token: String,
        pub token_type: String,
    }

    impl AuthBodyDto {
        pub fn new(access_token: String, refresh_token: String, id: Uuid) -> Self {
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

    #[derive(Debug, Deserialize)]
    pub struct JwtPayloadDto {
        pub id: Uuid,
    }

    impl JwtPayloadDto {
        pub fn new(id: Uuid) -> Self {
            Self { id }
        }
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
                subject_id: value.id.into(),
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
    use crate::models::tasks::Tasks;

    use super::*;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TaskMigrationDto {
        pub tasks: Vec<TaskDetails>,
        pub topics: Vec<super::topics::TopicDto>,
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
