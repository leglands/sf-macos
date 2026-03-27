//! sf-db — Local SQLite store for Software Factory macOS app
//! rusqlite with bundled SQLite (no external lib dependency)

pub mod migrations;
pub mod models;
pub mod store;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("{0}")]
    Other(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Thread-safe connection (Mutex — single-writer, enough for local app)
pub type DbConn = Arc<Mutex<Connection>>;

/// Open (or create) the local SQLite database, run migrations, return connection.
pub fn open_db(path: impl AsRef<Path>) -> DbResult<DbConn> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrations::run_migrations(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}
