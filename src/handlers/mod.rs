pub mod auth;

use crate::AppState;
use axum::Router;
use std::sync::Arc;

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .with_state(Arc::clone(&state))
        .merge(crate::handlers::auth::auth_routes(state.clone()))
}
