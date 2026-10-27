pub mod migrate;
pub mod reminder_job_repo;
pub mod task_repo;

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

pub use migrate::migrate;
pub use reminder_job_repo::SqliteReminderJobRepository;
pub use task_repo::{SqliteCategoryRepository, SqliteSettingsRepository, SqliteTaskRepository};

/// A shareable SQLite connection.
///
/// `rusqlite::Connection` is `Send` but not `Sync`; wrapping it in a `Mutex`
/// behind an `Arc` lets multiple repository types share one connection safely.
#[derive(Debug, Clone)]
pub struct SqliteConnection {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteConnection {
    pub fn new(conn: Connection) -> Self {
        let _ = conn.pragma_update(None, "foreign_keys", "ON");
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, rusqlite::Error> {
        Ok(Self::new(Connection::open(path)?))
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        Ok(Self::new(Connection::open_in_memory()?))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("SQLite connection mutex poisoned")
    }
}
