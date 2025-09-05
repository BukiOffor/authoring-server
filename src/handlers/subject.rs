use crate::helpers::dto::MessageDto;
use crate::helpers::dto::auth::Otp;
use crate::helpers::dto::subject::ItemReadyStats;
use crate::helpers::jwt::Claims;
use crate::{AppState, error::ModuleError};
use axum::routing::post;
use axum::{Json, extract::State};
use axum::{Router, extract::Path, routing::get};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    let routes = get_routes(state.clone());
    let api = Router::new().nest("/subject", routes);
    Router::new().merge(api)
}

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/stats/subject_id/{subject_id}/task/{task_id}",
            get(get_item_count_for_publishing),
        )
        .route("/publish/otp/send/{subject_id}", post(send_otp))
        .route(
            "/publish/subject_id/{subject_id}/task/{task_id}",
            post(publish_items),
        )
        .with_state(state)
}

pub async fn get_item_count_for_publishing(
    State(state): State<Arc<AppState>>,
    Path((subject_id, task_id)): Path<(String, String)>,
) -> Result<Json<Vec<ItemReadyStats>>, ModuleError> {
    let response = crate::services::subject::get_item_count_for_publishing(
        &subject_id,
        &task_id,
        state.pool.clone(),
    )?;
    Ok(Json(response))
}

pub async fn publish_items(
    State(state): State<Arc<AppState>>,
    Path((subject_id, task_id)): Path<(String, String)>,
    Json(payload): Json<Otp>,
) -> Result<Json<MessageDto>, ModuleError> {
    let response = crate::services::subject::publish_items(
        &subject_id,
        &task_id,
        payload,
        state.pool.clone(),
        &state.otp_manager,
    )
    .await?;
    Ok(Json(response))
}

pub async fn send_otp(
    Claims { user_id, .. }: Claims,
    State(state): State<Arc<AppState>>,
    Path(subject_id): Path<String>,
) -> Result<Json<MessageDto>, ModuleError> {
    let otp_manager = &state.otp_manager;
    let pool = state.pool.clone();
    let response = crate::services::subject::send_otp(
        user_id,
        subject_id,
        "OTP Verification",
        otp_manager,
        pool,
    )
    .await?;
    Ok(Json(response))
}
