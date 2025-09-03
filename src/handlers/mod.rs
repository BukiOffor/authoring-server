pub mod auth;
pub mod items;
pub mod subject;
pub mod topics;

use crate::{AppState, helpers::jwt::auth_middleware};
use axum::{Router, middleware};
use std::sync::Arc;
use tower::ServiceBuilder;

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        //.with_state(Arc::clone(&state))
        .merge(crate::handlers::topics::routes(state.clone()))
        .merge(crate::handlers::items::routes(state.clone()))
        .merge(crate::handlers::subject::routes(state.clone()))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(auth_middleware)))
        .merge(crate::handlers::auth::routes(state.clone()))
}
