use crate::error::ErrorMessage;
use crate::helpers::dto::MessageDto;
use crate::helpers::dto::auth::Otp;
use crate::helpers::dto::items::{ItemTotalStats, Options};
use crate::helpers::dto::pagination::{PaginatedResult, Pagination};
use crate::helpers::otp::OtpManager;
use crate::helpers::{querys, read_config};
use crate::models::item::{ItemStatus, Items};
use crate::models::item_options::ItemOptions;
use crate::models::passages::Passage;
use crate::models::user::User;
use crate::schema::user;
use crate::{DbConn, DbPool, error::ModuleError, helpers::dto::subject::*, schema::*};
use crate::{fetch, helpers};
use diesel::prelude::*;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub fn get_item_count_for_publishing(
    subject_id: &str,
    pool: Arc<DbPool>,
) -> Result<Vec<ItemReadyStats>, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let task_id = helpers::get_current_user_task(&mut conn)?;

    if let Some(task_id) = task_id {
        let query = diesel::sql_query(crate::helpers::querys::GET_SUBJECT_ITEM_READY_DETAILS);
        let stats: Vec<ItemReadyStats> = query
            .bind::<diesel::sql_types::Text, _>(subject_id)
            .bind::<diesel::sql_types::Text, _>(task_id)
            .load(&mut conn)
            .map_err(|e| ModuleError::InternalError(e.to_string()))?;

        Ok(stats)
    } else {
        Err(ModuleError::Error(
            "Task Id could not be fetched".to_string(),
        ))
    }
}
#[allow(unused)]
pub async fn publish_items(
    subject_id: &str,
    payload: Otp,
    pool: Arc<DbPool>,
    otp_manager: &OtpManager,
) -> Result<MessageDto, ModuleError> {
    use super::auth::try_login;

    if payload.code.ne("123456") {
        return Err(ModuleError::Error("Invalid OTP".into()));
    }
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let current_task = helpers::get_current_user_task(&mut conn)?;
    if current_task.is_none() {
        return Err(ModuleError::Error("No active task is running".to_string()));
    }

    let task_id = current_task.unwrap();
    let user: User = fetch!(user::table, user::id, payload.user_id.clone(), User, conn);

    if let Some(secret) = user.secret {
        if !helpers::password_verfier(&payload.secret, &secret) {
            return Err(ModuleError::Error(
                "secret provided was incorrect".to_string(),
            ));
        }
    } else {
        return Err(ModuleError::Error(
            "User has not set secret password".to_string(),
        ));
    }
    // let is_verified = otp_manager.verify_otp(&format!("publish_{}", subject_id), &payload.code);
    // if !is_verified.0 {
    //     return Err(ModuleError::InvalidOtp(is_verified.1));
    // }
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    // let auth_payload = AuthPayloadDto::new(user.email, payload.secret.clone());

    // if let Err(e) = try_login(auth_payload, &client).await {
    //     return Err(e);
    // }

    // let url = std::env::var("UPSTREAM_SERVER")
    //     .map(|server| format!("{}/author/accept", server))
    //     .map_err(|e| ModuleError::Error(e.to_string()))?;
    
    let upstream = read_config()?.upstream_server;
    let url = format!("{}/author/accept", upstream);

    let items_to_send = build_items_for_publishing(subject_id, &task_id, user.id, payload.secret, &mut conn)?;

    let server_response = client
        .post(url)
        .json(&items_to_send)
        .send()
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    tracing::info!("Upload items status: {}", server_response.status());
    if !server_response.status().is_success() {
        let body = server_response.text().await.unwrap_or_default();
        tracing::info!("{}", body);
        let message: ErrorMessage = serde_json::from_str(&body).unwrap_or_default();
        return Err(ModuleError::InternalError(message.message));
    }
    let mut items_published = Vec::new();
    items_published.extend_from_slice(&items_to_send.items);
    items_to_send.passage.into_iter().for_each(|p| {
        items_published.extend_from_slice(&p.items);
    });
    let items = items_published
        .into_iter()
        .map(|i| i.id)
        .collect::<Vec<String>>();

    diesel::update(items::table.filter(items::id.eq_any(&items)))
        .set(items::status.eq(ItemStatus::Submitted))
        .execute(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    Ok("Items published successfully".into())
}

pub fn build_items_for_publishing(
    subject_id: &str,
    task_id: &str,
    user: String,
    secret: String,
    conn: &mut DbConn,
) -> Result<ItemTransferDto, ModuleError> {
    let mut response = ItemTransferDto {
        user,
        secret,
        passage: Vec::new(),
        items: Vec::new(),
    };

    let single_items_to_publish = items::table
        .filter(items::subject_id.eq(subject_id))
        .filter(items::task_id.eq(task_id))
        .filter(items::passage_id.is_null())
        .filter(items::status.eq(ItemStatus::Ready))
        .select(Items::as_select())
        .load::<Items>(conn)?;

    for item in single_items_to_publish {
        let options = item_options::table
            .filter(item_options::item_id.eq(&item.id))
            .select(ItemOptions::as_select())
            .load::<ItemOptions>(conn)?
            .into_iter()
            .map(|op| op.into())
            .collect::<Vec<Options>>();
        response.items.push(AcceptItemDto::from(item, options))
    }

    let passage_items_to_publish = items::table
        .filter(items::subject_id.eq(subject_id))
        .filter(items::task_id.eq(task_id))
        .filter(items::passage_id.is_not_null())
        .filter(items::status.eq(ItemStatus::Ready))
        .select(Items::as_select())
        .load::<Items>(conn)?;

    // validate existing passages items to make sure all items are in the same state
    {
        let passages_id_to_validate: HashSet<String> = passage_items_to_publish
            .iter()
            .map(|i| i.passage_id.clone().unwrap_or_default())
            .collect();
        let passage_items_to_validate = items::table
            .filter(items::passage_id.eq_any(&passages_id_to_validate))
            .select(Items::as_select())
            .load::<Items>(conn)?;
        for item in passage_items_to_validate {
            if item.status.ne(&ItemStatus::Ready) {
                return Err(ModuleError::Error("Item in passage has a state that is not ready, please complete changes before publishing".into()));
            }
        }
    }

    let mut passage_item_collections: HashMap<String, Vec<String>> = HashMap::new();
    for item in passage_items_to_publish {
        let passage_id = item.passage_id.unwrap();
        passage_item_collections
            .entry(passage_id)
            .or_default()
            .push(item.id);
    }
    let passage_item_collections: Vec<(String, Vec<String>)> = passage_item_collections
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();

    for (passage_id, item_ids) in passage_item_collections {
        let passage = passages::table
            .filter(passages::id.eq(&passage_id))
            .select(Passage::as_select())
            .first::<Passage>(conn)?;

        let items = items::table
            .filter(items::id.eq_any(&item_ids))
            .select(Items::as_select())
            .load::<Items>(conn)?;

        let options = item_options::table
            .filter(item_options::item_id.eq_any(&item_ids))
            .select(ItemOptions::as_select())
            .load::<ItemOptions>(conn)?;

        struct ItemsWithOptions {
            pub item: Items,
            pub options: Vec<ItemOptions>,
        }
        // items with there options
        let mut itwo = Vec::new();

        for item in items {
            let item_options = options
                .clone()
                .into_iter()
                .filter(|op| op.item_id == item.id)
                .collect();
            itwo.push(ItemsWithOptions {
                item,
                options: item_options,
            });
        }
        let mut passage_items = Vec::new();
        for item in itwo {
            let options = item
                .options
                .into_iter()
                .map(|op| op.into())
                .collect::<Vec<Options>>();
            let item_dto = AcceptItemDto::from(item.item, options);
            passage_items.push(item_dto);
        }

        let passage_with_items = PassageDto {
            id: passage.id,
            rubric: passage.rubric.unwrap_or_default(),
            stem: passage.stem,
            topic_id: passage.topic_id,
            subject_id: passage.subject_id,
            items: passage_items,
        };
        response.passage.push(passage_with_items);
    }

    Ok(response)
}

pub async fn send_otp(
    user_id: String,
    subject_id: String,
    email_subject: &str,
    otp_manager: &OtpManager,
    pool: Arc<DbPool>,
) -> Result<MessageDto, ModuleError> {
    use crate::schema::user;

    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let user: User = fetch!(user::table, user::id, user_id.clone(), User, conn);
    let identifier = format!("publish_{}", subject_id);
    let otp = otp_manager.generate_otp(&identifier);
    crate::mailer::send_otp(user.first_name, otp, user.email, email_subject).await?;
    Ok("OTP sent successfully".into())
}

// fn update_item_status(items: Vec<String>, conn: &mut DbConn) -> Result<(), ModuleError> {
//     diesel::update(items::table.filter(items::id.eq_any(&items)))
//         .set(items::status.eq(ItemStatus::Submitted))
//         .execute(conn)
//         .map_err(|e| ModuleError::InternalError(e.to_string()))?;
//     Ok(())
// }

pub fn get_item_stats_for_subject(
    subject_id: String,
    pagination: Pagination,
    pool: Arc<DbPool>,
) -> Result<PaginatedResult<ItemTotalStats>, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let items = diesel::sql_query(querys::GET_SUBJECT_ITEM_TOTAL_STATS.to_string())
        .bind::<diesel::sql_types::Text, _>(subject_id.clone())
        .bind::<diesel::sql_types::Integer, _>(pagination.size)
        .bind::<diesel::sql_types::Integer, _>(pagination.offset())
        .load::<ItemTotalStats>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let count = diesel::sql_query(querys::GET_SUBJECT_ITEM_TOTAL_STATS.to_string())
        .bind::<diesel::sql_types::Text, _>(subject_id)
        .bind::<diesel::sql_types::Integer, _>(i32::MAX)
        .bind::<diesel::sql_types::Integer, _>(0)
        .load::<ItemTotalStats>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let result = PaginatedResult::new(items, count.len(), pagination);

    Ok(result)
}

pub fn get_subjects(pool: Arc<DbPool>) -> Result<Vec<SubjectDto>, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let subjects = tasks::table
        .select((tasks::subject_id, tasks::subject_name, tasks::subject_code))
        .distinct()
        .load::<(String, String, String)>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?
        .into_iter()
        .map(|(id, name, code)| SubjectDto { id, name, code })
        .collect();

    Ok(subjects)
}

pub fn get_subjects_dashboard(
    pool: Arc<DbPool>,
    id: String,
) -> Result<SubjectDashboardDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let query = r#"
        SELECT t.subject_id AS id,
               t.subject_name AS name,
               t.subject_code AS code,
               COALESCE(SUM(CASE WHEN i.status = '"Draft"' THEN 1 ELSE 0 END), 0) AS draft,
               COALESCE(SUM(CASE WHEN i.status = '"Submitted"' THEN 1 ELSE 0 END), 0) AS submitted,
               COALESCE(COUNT(i.id), 0) AS total_items
        FROM (
            SELECT DISTINCT subject_id, subject_name, subject_code
            FROM tasks
        ) t
        LEFT JOIN items i ON t.subject_id = i.subject_id
        WHERE t.subject_id = ?
        GROUP BY t.subject_id, t.subject_name, t.subject_code
    "#;
    let subjects_dashboard = diesel::sql_query(query)
        .bind::<diesel::sql_types::Text, _>(id)
        .get_result::<SubjectDashboardDto>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    Ok(subjects_dashboard)
}
