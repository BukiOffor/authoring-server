use std::sync::Arc;

use authoring_server::helpers::otp::OtpManager;
use authoring_server::{AppState, config, handlers};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use reqwest::Method;
use reqwest::header::*;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
const CONFIG_BYTES: &[u8] = include_bytes!("../.env");

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
    write_config().unwrap_or_else(|e| {
        tracing::error!("Could not write config file: {}", e.to_string());
        std::process::exit(1);
    });
    let state: Arc<AppState> = AppState {
        pool,
        otp_manager: OtpManager::new(10, 3),
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
        .allow_headers([
            CONTENT_TYPE,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
            AUTHORIZATION,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_HEADERS,
        ])
        .allow_credentials(true)
        .allow_origin([
            "http://192.168.1.177:4200".parse::<HeaderValue>().unwrap(),
            "http://192.168.1.177:4300".parse::<HeaderValue>().unwrap(),
            "http://localhost:4200".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:1430".parse::<HeaderValue>().unwrap(),
            "http://localhost:1430".parse::<HeaderValue>().unwrap(),
            "tauri://localhost".parse().unwrap(), 
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

fn write_config() -> std::io::Result<()> {
    use std::fs;
    use std::io::Write;
    let current_dir = std::env::current_dir()?;
    tracing::info!("Current directory: {:?}", current_dir);
    let file_path = current_dir.join(".env");
    // Only write if file doesn’t already exist
    if !file_path.exists() {
        let mut file = fs::File::create(&file_path)?;
        file.write_all(CONFIG_BYTES)?;
        println!("✅ Wrote config file to {:?}", file_path);
    } else {
        println!("ℹ️ Config file already exists at {:?}", file_path);
    }
    Ok(())
}

// fn write_config() -> std::io::Result<()> {
//     use std::fs;
//     use std::io::Write;

//     // Get home dir
//     let home_dir = std::env::home_dir().expect("Failed to find home directory");

//     let sib_dir = home_dir.join(".sib");
//     fs::create_dir_all(&sib_dir)?; // ensure directory exists

//     let current_dir = std::env::current_dir()?;
//     tracing::info!("Current directory: {:?}", current_dir);

//     // Target file path
//     let file_path = sib_dir.join("config.json");

//     // Only write if file doesn’t already exist
//     if !file_path.exists() {
//         let mut file = fs::File::create(&file_path)?;
//         file.write_all(CONFIG_BYTES)?;
//         println!("✅ Wrote config file to {:?}", file_path);
//     } else {
//         println!("ℹ️ Config file already exists at {:?}", file_path);
//     }

//     Ok(())
// }
