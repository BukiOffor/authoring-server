use std::sync::Arc;

use authoring_server::helpers::otp::OtpManager;
use authoring_server::{AppState, config, handlers};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = config::establish_connection();
    let state: Arc<AppState> = AppState {
        pool,
        otp_manager: OtpManager::new(5, 3),
    }
    .into();
    let app = handlers::get_routes(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());
    info!("Starting Web Server ............");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4756").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
