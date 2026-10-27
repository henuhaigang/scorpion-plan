use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::DomainError;
use super::reminder::ReminderPreference;
use super::schedule::TaskSchedule;
use super::title::normalize_title;

/// The only persisted task business statuses.
/// TODAY/FUTURE/LEGACY/OVERDUE are derived view states, never stored here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Todo,
    Completed,
    Canceled,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskPriority {
    Low,
    #[default]
    Normal,
    High,
}

/// A task entity. Identified by a stable UUID from creation.
///
/// `original_schedule` is the plan at creation; `current_schedule` is what the
/// task is currently scheduled for. Rescheduling only touches `current_schedule`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub original_schedule: TaskSchedule,
    pub current_schedule: TaskSchedule,
    pub priority: TaskPriority,
    pub category_id: Option<Uuid>,
    pub reminder_preference: ReminderPreference,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub version: i64,
}

impl Task {
    pub fn new(
        id: Uuid,
        title: String,
        schedule: TaskSchedule,
        priority: TaskPriority,
        category_id: Option<Uuid>,
        reminder_preference: ReminderPreference,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        schedule.validate()?;
        reminder_preference.validate()?;
        let title = normalize_title(&title)?;
        Ok(Self {
            id,
            title,
            status: TaskStatus::Todo,
            original_schedule: schedule,
            current_schedule: schedule,
            priority,
            category_id,
            reminder_preference,
            created_at: now,
            updated_at: now,
            completed_at: None,
            deleted_at: None,
            version: 1,
        })
    }

    fn require_active(&self) -> Result<(), DomainError> {
        if self.deleted_at.is_some() {
            return Err(DomainError::Deleted);
        }
        Ok(())
    }

    fn transition(&mut self, to: TaskStatus, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.require_active()?;
        let from = self.status;
        let allowed = matches!(
            (from, to),
            (TaskStatus::Todo, TaskStatus::Completed)
                | (TaskStatus::Completed, TaskStatus::Todo)
                | (TaskStatus::Todo, TaskStatus::Canceled)
                | (TaskStatus::Canceled, TaskStatus::Todo)
        );
        if !allowed {
            return Err(DomainError::InvalidTransition { from, to });
        }
        self.status = to;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    /// TODO -> COMPLETED. Sets `completed_at`; keeps the schedule.
    pub fn complete(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.transition(TaskStatus::Completed, now)?;
        self.completed_at = Some(now);
        Ok(())
    }

    /// COMPLETED -> TODO. Clears `completed_at`; caller recalculates reminders.
    pub fn undo_completion(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.transition(TaskStatus::Todo, now)?;
        self.completed_at = None;
        Ok(())
    }

    /// TODO -> CANCELED. Keeps the schedule; caller cancels pending reminders.
    pub fn cancel(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.transition(TaskStatus::Canceled, now)?;
        Ok(())
    }

    /// CANCELED -> TODO (explicit restore).
    pub fn restore(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.transition(TaskStatus::Todo, now)?;
        Ok(())
    }

    /// Soft delete (tombstone). Distinct from cancellation; never physically removes.
    pub fn soft_delete(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        self.require_active()?;
        if self.deleted_at.is_some() {
            return Err(DomainError::Deleted);
        }
        self.deleted_at = Some(now);
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    /// Reschedule: changes `current_schedule` only, never `original_schedule`.
    ///
    /// `new_time`:
    /// - `None`          => keep the task's current time (untimed stays untimed).
    /// - `Some(Some(t))` => set a specific time.
    /// - `Some(None)`    => explicitly clear the time (task becomes untimed).
    pub fn reschedule(
        &mut self,
        date: chrono::NaiveDate,
        new_time: Option<Option<chrono::NaiveTime>>,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        self.require_active()?;
        if self.status != TaskStatus::Todo {
            return Err(DomainError::RescheduleNotAllowed(self.status));
        }
        let time = match new_time {
            Some(t) => t,
            None => self.current_schedule.time,
        };
        let all_day = if time.is_some() {
            false
        } else {
            self.current_schedule.all_day
        };
        let schedule = TaskSchedule {
            date,
            time,
            all_day,
        };
        schedule.validate()?;
        self.current_schedule = schedule;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    /// Update mutable fields (title, priority, category, reminder preference).
    pub fn update_fields(
        &mut self,
        title: Option<String>,
        priority: Option<TaskPriority>,
        category_id: Option<Option<Uuid>>,
        reminder_preference: Option<ReminderPreference>,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        self.require_active()?;
        if let Some(t) = title {
            self.title = normalize_title(&t)?;
        }
        if let Some(p) = priority {
            self.priority = p;
        }
        if let Some(c) = category_id {
            self.category_id = c;
        }
        if let Some(r) = reminder_preference {
            r.validate()?;
            self.reminder_preference = r;
        }
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
    use uuid::Uuid;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 26, 12, 0, 0).unwrap()
    }

    fn task() -> Task {
        Task::new(
            Uuid::new_v4(),
            "写周报".into(),
            TaskSchedule::timed(
                NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(),
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            ),
            TaskPriority::Normal,
            None,
            ReminderPreference::InheritDefault,
            now(),
        )
        .unwrap()
    }

    #[test]
    fn new_task_starts_at_version_one_with_matching_schedules() {
        let t = task();
        assert_eq!(t.version, 1);
        assert_eq!(t.original_schedule, t.current_schedule);
        assert_eq!(t.status, TaskStatus::Todo);
        assert!(t.completed_at.is_none());
        assert!(t.deleted_at.is_none());
    }

    #[test]
    fn complete_sets_completed_at_and_bumps_version() {
        let mut t = task();
        t.complete(now()).unwrap();
        assert_eq!(t.status, TaskStatus::Completed);
        assert_eq!(t.completed_at, Some(now()));
        assert_eq!(t.version, 2);
    }

    #[test]
    fn undo_completion_clears_completed_at() {
        let mut t = task();
        t.complete(now()).unwrap();
        t.undo_completion(now()).unwrap();
        assert_eq!(t.status, TaskStatus::Todo);
        assert!(t.completed_at.is_none());
        assert_eq!(t.version, 3);
    }

    #[test]
    fn cancel_and_restore_round_trip() {
        let mut t = task();
        t.cancel(now()).unwrap();
        assert_eq!(t.status, TaskStatus::Canceled);
        t.restore(now()).unwrap();
        assert_eq!(t.status, TaskStatus::Todo);
        assert_eq!(t.version, 3);
    }

    #[test]
    fn illegal_transitions_are_rejected() {
        let mut t = task();
        t.complete(now()).unwrap();
        assert_eq!(
            t.cancel(now()),
            Err(DomainError::InvalidTransition {
                from: TaskStatus::Completed,
                to: TaskStatus::Canceled
            })
        );
        let mut t = task();
        t.cancel(now()).unwrap();
        assert_eq!(
            t.complete(now()),
            Err(DomainError::InvalidTransition {
                from: TaskStatus::Canceled,
                to: TaskStatus::Completed
            })
        );
    }

    #[test]
    fn soft_delete_is_a_tombstone_not_a_status_change() {
        let mut t = task();
        t.soft_delete(now()).unwrap();
        assert!(t.is_deleted());
        assert_eq!(t.status, TaskStatus::Todo);
        assert_eq!(t.version, 2);
        // A second delete is rejected.
        assert_eq!(t.soft_delete(now()), Err(DomainError::Deleted));
    }

    #[test]
    fn reschedule_keeps_original_schedule() {
        let mut t = task();
        let new_date = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        t.reschedule(new_date, None, now()).unwrap();
        assert_eq!(t.current_schedule.date, new_date);
        // Time preserved.
        assert_eq!(
            t.current_schedule.time,
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
        );
        // Original untouched.
        assert_eq!(
            t.original_schedule.date,
            NaiveDate::from_ymd_opt(2026, 10, 26).unwrap()
        );
    }

    #[test]
    fn reschedule_can_explicitly_clear_time() {
        let mut t = task();
        t.reschedule(
            NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
            Some(None),
            now(),
        )
        .unwrap();
        assert!(t.current_schedule.time.is_none());
        assert!(!t.current_schedule.all_day);
    }

    #[test]
    fn reschedule_disallowed_for_completed_and_canceled() {
        let mut t = task();
        t.complete(now()).unwrap();
        assert_eq!(
            t.reschedule(NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(), None, now()),
            Err(DomainError::RescheduleNotAllowed(TaskStatus::Completed))
        );
        let mut t = task();
        t.cancel(now()).unwrap();
        assert_eq!(
            t.reschedule(NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(), None, now()),
            Err(DomainError::RescheduleNotAllowed(TaskStatus::Canceled))
        );
    }

    #[test]
    fn every_mutation_bumps_version_and_updated_at() {
        let mut t = task();
        let base_updated = t.updated_at;
        t.update_fields(Some("新标题".into()), None, None, None, now())
            .unwrap();
        assert_eq!(t.version, 2);
        assert_eq!(t.updated_at, base_updated);
        assert_eq!(t.title, "新标题");
    }
}
