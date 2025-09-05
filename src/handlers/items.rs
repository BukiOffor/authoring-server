use crate::helpers::dto::MessageDto;
use crate::helpers::dto::items::display::TopicItemsDto;
use crate::helpers::dto::items::*;
use crate::{AppState, error::ModuleError, services};
use axum::routing::delete;
use axum::routing::patch;
use axum::routing::post;
use axum::{Json, extract::State};
use axum::{Router, extract::Path, routing::get};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    let routes = get_routes(state.clone());
    let api = Router::new().nest("/items", routes);
    Router::new().merge(api)
}

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/stats/{subject_id}", get(fetch_item_stats))
        .route("/create/single", post(create_item))
        .route("/create/passage", post(create_passage_and_items))
        .route("/get/{topic_id}/{task_id}", get(fetch_items_under_topic))
        .route("/delete/{item_id}", delete(delete_item))
        .route("/update/{item_id}", patch(update_item))
        .route("/update/status/{item_id}", patch(update_item_status))
        .with_state(state)
}

pub async fn fetch_item_stats(
    State(state): State<Arc<AppState>>,
    Path(subject_id): Path<String>,
) -> Result<Json<ItemStats>, ModuleError> {
    let response = crate::services::items::fetch_item_stats(&subject_id, state.pool.clone())?;
    Ok(Json(response))
}

pub async fn create_item(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateItemDto>,
) -> Result<Json<MessageDto>, ModuleError> {
    let item: ItemWithOptions = payload.into();
    let response =
        crate::services::items::create_item(state.pool.clone(), item.item, item.options)?;
    Ok(Json(response))
}

pub async fn create_passage_and_items(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PassageDto>,
) -> Result<Json<MessageDto>, ModuleError> {
    let response =
        crate::services::items::create_passage_and_items(state.pool.clone(), payload.build())?;
    Ok(Json(response))
}

pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> Result<Json<MessageDto>, ModuleError> {
    let response = services::items::delete_item(item_id, state.pool.clone())?;
    Ok(Json(response))
}

pub async fn fetch_items_under_topic(
    State(state): State<Arc<AppState>>,
    Path((topic_id, task_id)): Path<(String, String)>,
) -> Result<Json<TopicItemsDto>, ModuleError> {
    let response =
        services::items::fetch_topic_items_with_subtopics(&topic_id, &task_id, state.pool.clone())?;
    Ok(Json(response))
}

pub async fn update_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(dto): Json<EditItemDto>,
) -> Result<Json<MessageDto>, ModuleError> {
    let publish = dto.publish.clone();
    let response = services::items::update_item(item_id, dto, state.pool.clone(), publish)?;
    Ok(Json(response))
}

pub async fn update_item_status(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(dto): Json<UpdateItemStatus>,
) -> Result<Json<MessageDto>, ModuleError> {
    let response = services::items::update_item_status(item_id, dto.status, state.pool.clone())?;
    Ok(Json(response))
}
