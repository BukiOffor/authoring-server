use crate::{AppState, error::ModuleError, helpers::dto::topic::TopicNode};
use axum::{Json, extract::State};
use axum::{Router, extract::Path, routing::get};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    let routes = get_routes(state.clone());
    let api = Router::new().nest("/topics", routes);
    Router::new().merge(api)
}

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/subject/{subject_id}", get(fetch_subject_topics))
        .route(
            "/subject/{subject_id}/topic/{topic_id}/subtopic",
            get(fetch_subtopics_under_topic),
        )
        .with_state(state)
}

pub async fn fetch_subject_topics(
    State(state): State<Arc<AppState>>,
    Path(subject_id): Path<String>,
) -> Result<Json<Vec<TopicNode>>, ModuleError> {
    let response = crate::services::topics::fetch_subject_topics(&subject_id, state.pool.clone())?;
    Ok(Json(response))
}

pub async fn fetch_subtopics_under_topic(
    State(state): State<Arc<AppState>>,
    Path((subject_id, topic_id)): Path<(String, String)>,
) -> Result<Json<Vec<TopicNode>>, ModuleError> {
    let response = crate::services::topics::fetch_subtopics_under_topic(
        &subject_id,
        &topic_id,
        state.pool.clone(),
    )?;
    Ok(Json(response))
}
