use crate::helpers::dto::MessageDto;
use crate::helpers::dto::user::{ResetPasswordDto, UpdateUserDto};
use crate::models::user::UserDto;
use crate::services;
use crate::{AppState, error::ModuleError};
use axum::Router;
use axum::extract::Path;
use axum::routing::{get, patch, post};
use axum::{Json, extract::State};
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SetSecretPayload {
    pub password: String,
    pub secret: String,
}

pub fn routes(state: Arc<AppState>) -> Router {
    let routes = get_routes(state.clone());
    let api = Router::new().nest("/user", routes);
    Router::new().merge(api)
}

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/update/{user_id}", patch(update_profile))
        .route("/fetch", get(fetch_user))
        .route("/secret/set", post(set_secret_password))
        .route("/password/reset", post(reset_password))
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

pub async fn set_secret_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetSecretPayload>,
) -> Result<Json<MessageDto>, ModuleError>{
    let response = services::user::set_secret_password(state.pool.clone(), payload.password, payload.secret).await?;
    Ok(Json(response))
}



pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordDto>,
) -> Result<Json<MessageDto>, ModuleError>{
    let response = services::user::reset_password(payload, state.pool.clone()).await?;
    Ok(Json(response))
}
