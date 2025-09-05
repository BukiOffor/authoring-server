use crate::error::ErrorMessage;
use crate::helpers::dto::MessageDto;
use crate::helpers::dto::auth::Otp;
use crate::helpers::dto::items::{ItemTotalStats, Options};
use crate::helpers::otp::OtpManager;
use crate::helpers::querys;
use crate::models::item::{ItemStatus, Items};
use crate::models::item_options::ItemOptions;
use crate::models::passages::Passage;
use crate::models::user::User;
use crate::schema::user;
use crate::{DbConn, DbPool, error::ModuleError, helpers::dto::subject::*, schema::*};
use crate::{fetch, helpers};
use diesel::prelude::*;
use helpers::dto::auth::AuthPayloadDto;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

pub fn get_item_count_for_publishing(
    subject_id: &str,
    task_id: &str,
    pool: Arc<DbPool>,
) -> Result<Vec<ItemReadyStats>, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let query = diesel::sql_query(crate::helpers::querys::GET_SUBJECT_ITEM_READY_DETAILS);
    let stats: Vec<ItemReadyStats> = query
        .bind::<diesel::sql_types::Text, _>(subject_id)
        .bind::<diesel::sql_types::Text, _>(task_id)
        .load(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    Ok(stats)
}

pub async fn publish_items(
    subject_id: &str,
    task_id: &str,
    payload: Otp,
    pool: Arc<DbPool>,
    otp_manager: &OtpManager,
) -> Result<MessageDto, ModuleError> {
    use super::auth::try_login;

    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let user: User = fetch!(user::table, user::id, payload.user_id.clone(), User, conn);

    if !helpers::password_verfier(&payload.secret, &user.password_hash) {
        return Err(ModuleError::Error(
            "secret provided was incorrect".to_string(),
        ));
    }
    let is_verified = otp_manager.verify_otp(&format!("publish_{}", subject_id), &payload.code);
    if !is_verified {
        return Err(ModuleError::InvalidOtp);
    }
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let auth_payload = AuthPayloadDto::new(user.email, payload.secret);

    if let Err(e) = try_login(auth_payload, &client).await {
        return Err(e);
    }

    let url = std::env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/author/accept", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;

    let items_to_send = build_items_for_publishing(subject_id, task_id, &mut conn)?;

    let server_response = client
        .post(url)
        .json(&items_to_send)
        .send()
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    tracing::info!("Upload items status: {}", server_response.status());
    if !server_response.status().is_success() {
        let body = server_response.text().await.unwrap_or_default();
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

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with cookie store enabled
    let client = Client::builder()
        .cookie_store(true) // This is key - enables cookie persistence
        .build()?;

    // 1. Login and receive authentication cookie
    let login_response = client
        .post("http://localhost:8000/auth")
        .json(&json!({
            "email": "ebuka2264@yahoo.com",
            "password": "password"
        }))
        .send()
        .await?;

    println!("Login status: {}", login_response.status());

    // The cookie is automatically stored by reqwest!
    // You can inspect the cookies if needed
    for cookie in login_response.cookies() {
        println!("Received cookie: {}={}", cookie.name(), cookie.value());
    }

    // 2. Make authenticated request - cookies are sent automatically
    let protected_response = client.get("http://localhost:8000/ping").send().await?;

    println!("Protected status: {}", protected_response.status());
    println!("Protected response: {}", protected_response.text().await?);

    Ok(())
}

pub fn build_items_for_publishing(
    subject_id: &str,
    task_id: &str,
    conn: &mut DbConn,
) -> Result<ItemTransferDto, ModuleError> {
    let mut response = ItemTransferDto {
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
    pool: Arc<DbPool>,
) -> Result<Vec<ItemTotalStats>, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let stats = diesel::sql_query(querys::GET_SUBJECT_ITEM_TOTAL_STATS.to_string())
        .bind::<diesel::sql_types::Text, _>(subject_id)
        .load::<ItemTotalStats>(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    Ok(stats)
}
