use crate::helpers::dto::subject::ItemReadyStats;
use crate::{AppState, error::ModuleError};
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
