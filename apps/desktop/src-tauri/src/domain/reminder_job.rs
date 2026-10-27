use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Lifecycle status of a reminder job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReminderJobStatus {
    /// Created but not yet confirmed by the OS scheduler.
    Pending,
    /// Confirmed by the OS scheduler; will fire at `fire_at`.
    Scheduled,
    /// Successfully delivered.
    Fired,
    /// Cancelled before firing (task completed, canceled, deleted, rescheduled).
    Canceled,
    /// Delivery attempt failed; will be retried on next reconciliation.
    Failed,
}

impl ReminderJobStatus {
    /// Whether this job is still "active" (could still fire).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Scheduled)
    }
}

/// A single reminder delivery job. The single source of truth for what
/// *should* fire, independent of OS scheduler state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReminderJob {
    pub id: Uuid,
    pub task_id: Uuid,
    /// The UTC instant at which the notification should be delivered.
    pub fire_at: DateTime<Utc>,
    pub status: ReminderJobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attempts: i32,
    pub last_error: Option<String>,
}

impl ReminderJob {
    pub fn new(task_id: Uuid, fire_at: DateTime<Utc>, now: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            fire_at,
            status: ReminderJobStatus::Pending,
            created_at: now,
            updated_at: now,
            attempts: 0,
            last_error: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_job_starts_pending() {
        let now = Utc::now();
        let job = ReminderJob::new(Uuid::new_v4(), now + chrono::Duration::minutes(10), now);
        assert_eq!(job.status, ReminderJobStatus::Pending);
        assert!(job.is_active());
        assert_eq!(job.attempts, 0);
        assert!(job.last_error.is_none());
    }

    #[test]
    fn status_serialization_round_trips() {
        for status in [
            ReminderJobStatus::Pending,
            ReminderJobStatus::Scheduled,
            ReminderJobStatus::Fired,
            ReminderJobStatus::Canceled,
            ReminderJobStatus::Failed,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: ReminderJobStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn job_serialization_round_trips() {
        let now = Utc::now();
        let job = ReminderJob::new(Uuid::new_v4(), now + chrono::Duration::minutes(5), now);
        let json = serde_json::to_string(&job).unwrap();
        let back: ReminderJob = serde_json::from_str(&json).unwrap();
        assert_eq!(job, back);
    }
}
