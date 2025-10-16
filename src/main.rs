use std::sync::Arc;

use authoring_server::helpers::otp::OtpManager;
use authoring_server::{AppState, config, handlers};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use reqwest::header::*;
use reqwest::Method;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool = config::establish_connection();

    run_migration(pool.clone()).unwrap_or_else(|e| {
        tracing::error!("Could not run migration: {}", e.to_string());
        std::process::exit(1)
    });
    let state: Arc<AppState> = AppState {
        pool,
        otp_manager: OtpManager::new(5, 3),
    }
    .into();
  let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_headers([CONTENT_TYPE, ACCESS_CONTROL_ALLOW_CREDENTIALS])
        .allow_credentials(true)
        .allow_origin([
            "http://192.168.1.177:4200".parse::<HeaderValue>().unwrap(),
            "http://192.168.1.177:4300".parse::<HeaderValue>().unwrap(),
            "http://localhost:4200".parse::<HeaderValue>().unwrap(),
            "http://localhost:4500".parse::<HeaderValue>().unwrap(),
            "https://02856ee0334e.ngrok-free.app"
                .parse::<HeaderValue>()
                .unwrap(),
        ]);
    info!("Starting Web Server ............");
    let app = handlers::get_routes(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors);
    info!("Starting Web Server ............");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4756").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

pub fn run_migration(
    pool: Arc<authoring_server::DbPool>,
) -> Result<(), Box<dyn std::error::Error>> {
    pool.get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| {
            tracing::error!("{}", e.to_string());
            std::process::exit(1)
        });
    Ok(())
}
