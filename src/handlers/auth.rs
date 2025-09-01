use crate::helpers::jwt::generate_token;
use crate::services;
use crate::{
    AppState,
    error::ModuleError,
    helpers::dto::auth::{AuthBodyDto, AuthPayloadDto},
};
use axum::Router;
use axum::routing::post;
use axum::{Json, extract::State};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new().route("/auth", post(auth)).with_state(state)
}

pub async fn auth(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayloadDto>,
) -> Result<Json<AuthBodyDto>, ModuleError> {
    let jwt = services::auth::authenticate_user(payload, state.pool.clone()).await?;
    let token = generate_token(jwt).await?;
    Ok(token)
}
