use crate::helpers::dto::MessageDto;
use crate::helpers::dto::user::UpdateUserDto;
use crate::models::user::UserDto;
use crate::services;
use crate::{AppState, error::ModuleError};
use axum::Router;
use axum::extract::Path;
use axum::routing::{get, patch, post};
use axum::{Json, extract::State};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    let routes = get_routes(state.clone());
    let api = Router::new().nest("/user", routes);
    Router::new().merge(api)
}

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/update/{user_id}", patch(update_profile))
        .route("/fetch", get(fetch_user))
        .route("/set_secret_password", post(set_secret_password))
        .route("/reset_password", post(reset_password))
        .with_state(state)
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<MessageDto>, ModuleError> {
    let response = services::user::update_user(user_id, payload, state.pool.clone())?;
    Ok(Json(response))
}

pub async fn fetch_user(State(state): State<Arc<AppState>>) -> Result<Json<UserDto>, ModuleError> {
    let response = services::user::fetch_user(state.pool.clone())?;
    Ok(Json(response))
}

pub async fn set_secret_password() {
    unimplemented!()
}

pub async fn reset_password() {
    unimplemented!()
}
