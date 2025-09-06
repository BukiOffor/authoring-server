pub mod auth;
pub mod items;
pub mod subject;
pub mod topics;
pub mod user;

use crate::{AppState, helpers::jwt::auth_middleware};
use axum::{Router, middleware};
use std::sync::Arc;
use tower::ServiceBuilder;

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(self::topics::routes(state.clone()))
        .merge(self::items::routes(state.clone()))
        .merge(self::subject::routes(state.clone()))
        .merge(self::user::routes(state.clone()))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(auth_middleware)))
        .merge(self::auth::routes(state.clone()))
}
