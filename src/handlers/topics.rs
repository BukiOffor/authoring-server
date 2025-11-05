use crate::helpers::dto::items::{FetchDto};
use crate::helpers::dto::topic::{SubTopicWithMetadata, TopicMetaData};
use crate::{AppState, error::ModuleError, helpers::dto::topic::TopicNode};
use axum::extract::Query;
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
            "/subject/{subject_id}/topic/{topic_id}/item_count",
            get(fetch_subtopics_item_count),
        )
        .route(
            "/subject/{subject_id}/topic/{topic_id}/subtopic",
            get(fetch_subtopics_under_topic),
        )
        .route(
            "/subject/{subject_id}/topic/{topic_id}/metadata",
            get(fetch_subtopics_under_topic_with_metadata),
        )
        .with_state(state)
}

pub async fn fetch_subject_topics(
    State(state): State<Arc<AppState>>,
    Path(subject_id): Path<String>,
) -> Result<Json<Vec<TopicNode>>, ModuleError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let response = crate::services::topics::fetch_subject_topics(&subject_id, &mut conn)?;
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

pub async fn fetch_subtopics_item_count(
    State(state): State<Arc<AppState>>,
    Path((subject_id, topic_id)): Path<(String, String)>,
) -> Result<Json<TopicMetaData>, ModuleError> {
    let response = crate::services::topics::fetch_subtopics_item_stats(
        &subject_id,
        &topic_id,
        state.pool.clone(),
    )?;
    Ok(Json(response))
}

pub async fn fetch_subtopics_under_topic_with_metadata(
    State(state): State<Arc<AppState>>,
    Path((subject_id, topic_id)): Path<(String, String)>,
    Query(query): Query<FetchDto>,
) -> Result<Json<Vec<SubTopicWithMetadata>>, ModuleError> {
    let status = crate::models::item::ItemStatus::from_fetch(query.status);
    let response = crate::services::topics::fetch_subtopics_under_topic_with_metadata(
        &subject_id,
        &topic_id,
        status,
        state.pool.clone(),
    )?;
    Ok(Json(response))
}
