pub mod handlers;
pub mod helpers;
pub mod mailer;
pub mod models;
pub mod services;
use std::{path::PathBuf, sync::Arc};

use diesel::SqliteConnection;
pub use serde::{Deserialize, Serialize};
pub mod schema;
pub use uuid::Uuid;
pub mod error;
pub mod macros;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub struct AppState {
    pub pool: Arc<DbPool>,
    pub otp_manager: helpers::otp::OtpManager,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub upstream_server: String,
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            upstream_server: "http://127.0.0.1:8000".to_string(),
            url: ".sib/app.sqlite3".to_string(),
        }
    }
}



pub mod config {
    use crate::helpers::read_config;

    use super::*;
    use dotenvy::dotenv;
    use std::{
        env::{home_dir},
        path,
        sync::Arc,
    };
    pub fn ensure_sqlite_db_exists() {
        dotenv().ok();
        let url = read_config().unwrap_or_default().url;
        let database_path = home_dir().unwrap().join(path::Path::new(&url));
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)
                .expect("Failed to create directories for SQLite database path");
        }
        if !database_path.exists() {
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&database_path)
                .expect("Failed to create SQLite database file");
        }
    }

    pub fn establish_connection() -> Arc<DbPool> {
        dotenv().ok();
        ensure_sqlite_db_exists();
        let url = read_config().unwrap_or_default().url;
        let database_url = home_dir().unwrap().join(path::Path::new(&url));
        let database_url = database_url.to_str().unwrap();

        let manager =
            diesel::r2d2::ConnectionManager::<SqliteConnection>::new(database_url.to_string());
        diesel::r2d2::Pool::builder()
            .max_size(2)
            .build(manager)
            .expect("Failed to create r2d2 pool for SQLite")
            .into()
    }
}

pub fn create_app_data_dir(app_name: &str) -> anyhow::Result<std::path::PathBuf> {
    let mut dir = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
            })
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .map(|h| h.join("Library").join("Application Support"))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    } else {
        // Linux and others
        std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
    };
    dir = dir.join(app_name);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}