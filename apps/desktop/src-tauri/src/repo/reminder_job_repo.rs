use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::reminder_job::{ReminderJob, ReminderJobStatus};
use crate::repo::RepoError;

/// Persistence port for reminder jobs.
pub trait ReminderJobRepository: Send + Sync {
    fn insert(&self, job: &ReminderJob) -> Result<(), RepoError>;
    fn get(&self, id: Uuid) -> Result<Option<ReminderJob>, RepoError>;
    /// List jobs for a task filtered by status.
    fn list_by_task(
        &self,
        task_id: Uuid,
        statuses: &[ReminderJobStatus],
    ) -> Result<Vec<ReminderJob>, RepoError>;
    /// List active jobs whose `fire_at` is at or before `before`, ordered by
    /// `fire_at` ascending. Used by the polling loop.
    fn list_due(&self, before: DateTime<Utc>, limit: i64) -> Result<Vec<ReminderJob>, RepoError>;
    /// Transition a job to a new status.
    fn update_status(
        &self,
        id: Uuid,
        status: ReminderJobStatus,
        last_error: Option<&str>,
    ) -> Result<(), RepoError>;
    /// Cancel all active (PENDING/SCHEDULED) jobs for a task.
    fn cancel_active_by_task(&self, task_id: Uuid) -> Result<usize, RepoError>;
}
