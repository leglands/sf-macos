//! Application state — shared between AppKit main thread and Tokio async tasks
//! via Arc<Mutex<AppState>> + dispatch_async for UI updates

use std::sync::{Arc, Mutex};
use sf_db::DbConn;
use sf_remote::instance::RemoteInstance;

#[derive(Debug)]
pub struct AppState {
    pub db: Arc<DbConn>,
    pub instances: Vec<RemoteInstance>,
    pub active_instance: Option<String>,  // instance id
}

pub type SharedState = Arc<Mutex<AppState>>;

impl AppState {
    pub fn new(db: Arc<DbConn>) -> SharedState {
        Arc::new(Mutex::new(Self {
            db,
            instances: vec![],
            active_instance: None,
        }))
    }
}
