pub mod handlers;
pub mod helpers;
pub mod mailer;
pub mod models;
pub mod services;
use std::sync::Arc;

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

pub mod config {
    use super::*;
    use dotenvy::dotenv;
    use std::{
        env::{self, home_dir},
        path,
        sync::Arc,
    };
    pub fn ensure_sqlite_db_exists() {
        dotenv().ok();
        let url = env::var("URL").unwrap_or(".sib/app.sqlite3".into());
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
        let url = env::var("URL").unwrap_or(".sib/app.sqlite3".into());
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
