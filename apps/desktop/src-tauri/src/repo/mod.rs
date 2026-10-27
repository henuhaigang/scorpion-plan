use uuid::Uuid;

use crate::domain::{GlobalReminderDefault, Task, TaskStatus};

pub mod sqlite;

pub use sqlite::{
    migrate, SqliteCategoryRepository, SqliteSettingsRepository, SqliteTaskRepository,
};

/// Repository-level failures. `NotFound` and `VersionConflict` are distinct so
/// callers can react differently (404 vs 409).
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("task {0} not found")]
    TaskNotFound(Uuid),
    #[error("task {0} is soft-deleted")]
    TaskDeleted(Uuid),
    #[error("version conflict on task {id}: expected {expected}, store has a different version")]
    VersionConflict { id: Uuid, expected: i64 },
    #[error("category {0} not found")]
    CategoryNotFound(Uuid),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
}

/// Query filter for listing tasks. All fields are optional; `None` = no constraint.
#[derive(Debug, Clone, Default)]
pub struct TaskQuery {
    /// Only these statuses (empty = all non-deleted).
    pub statuses: Vec<TaskStatus>,
    /// Include soft-deleted rows.
    pub include_deleted: bool,
    /// scheduled_date strictly before this date.
    pub date_before: Option<chrono::NaiveDate>,
    /// scheduled_date strictly after this date.
    pub date_after: Option<chrono::NaiveDate>,
    /// scheduled_date equal to this date.
    pub date_eq: Option<chrono::NaiveDate>,
}

/// Persistence port for tasks. Implementations must be safe for shared use.
pub trait TaskRepository: Send + Sync {
    fn insert(&self, task: &Task) -> Result<(), RepoError>;
    fn get(&self, id: Uuid) -> Result<Option<Task>, RepoError>;
    fn list(&self, query: &TaskQuery) -> Result<Vec<Task>, RepoError>;
    /// Optimistic-locking update.
    ///
    /// Applies `task` (which must carry `version = expected_version + 1`) only
    /// if the stored row still has `expected_version` and is not deleted.
    /// Returns the stored row after the update.
    fn update(&self, task: &Task, expected_version: i64) -> Result<Task, RepoError>;
    fn count(&self) -> Result<i64, RepoError>;
}

/// Persistence port for categories.
pub trait CategoryRepository: Send + Sync {
    fn insert(
        &self,
        id: Uuid,
        name: &str,
        sort_order: i64,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepoError>;
    fn list_active(&self) -> Result<Vec<Category>, RepoError>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub sort_order: i64,
}

/// Typed settings access. Domain code never reads raw strings.
pub trait SettingsRepository: Send + Sync {
    fn get_global_reminder_default(&self) -> Result<GlobalReminderDefault, RepoError>;
    fn set_global_reminder_default(&self, value: &GlobalReminderDefault) -> Result<(), RepoError>;
    fn get_string(&self, key: &str) -> Result<Option<String>, RepoError>;
    fn set_string(&self, key: &str, value: &str) -> Result<(), RepoError>;
}
