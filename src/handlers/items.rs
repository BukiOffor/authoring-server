use crate::helpers::dto::MessageDto;
use crate::helpers::dto::items::*;
use crate::{AppState, error::ModuleError};
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
