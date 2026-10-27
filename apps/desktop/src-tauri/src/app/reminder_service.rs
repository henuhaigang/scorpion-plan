use std::sync::Arc;

use uuid::Uuid;

use crate::domain::reminder_calc::compute_fire_at;
use crate::domain::reminder_job::{ReminderJob, ReminderJobStatus};
use crate::domain::TaskStatus;
use crate::repo::reminder_job_repo::ReminderJobRepository;
use crate::repo::{RepoError, SettingsRepository, TaskRepository};
use crate::scheduler::NotificationSender;
use crate::time::Clock;

use super::AppError;

/// Application service that reconciles reminder jobs with task state and
/// fires due notifications.
///
/// Reconciliation is idempotent: calling it multiple times for the same task
/// state produces the same job set.
pub struct ReminderService<
    R: TaskRepository,
    S: SettingsRepository,
    J: ReminderJobRepository,
    N: NotificationSender,
    C: Clock,
> {
    tasks: Arc<R>,
    settings: Arc<S>,
    jobs: Arc<J>,
    notifier: Arc<N>,
    clock: Arc<C>,
}

impl<
        R: TaskRepository,
        S: SettingsRepository,
        J: ReminderJobRepository,
        N: NotificationSender,
        C: Clock,
    > ReminderService<R, S, J, N, C>
{
    pub fn new(
        tasks: Arc<R>,
        settings: Arc<S>,
        jobs: Arc<J>,
        notifier: Arc<N>,
        clock: Arc<C>,
    ) -> Self {
        Self {
            tasks,
            settings,
            jobs,
            notifier,
            clock,
        }
    }

    /// Reconcile the reminder job for a single task.
    ///
    /// - If the task warrants a future reminder → ensure exactly one active job
    ///   exists with the correct `fire_at`.
    /// - If the task no longer warrants a reminder → cancel all active jobs.
    pub fn reconcile_task(&self, task_id: Uuid) -> Result<(), AppError> {
        let task = self
            .tasks
            .get(task_id)?
            .ok_or_else(|| RepoError::TaskNotFound(task_id))?;
        let global = self.settings.get_global_reminder_default()?;
        let now = self.clock.now_utc();
        let zone = self.clock.zone();

        let desired_fire_at = compute_fire_at(&task, &global, now, zone);
        let active_statuses = [ReminderJobStatus::Pending, ReminderJobStatus::Scheduled];
        let active_jobs = self.jobs.list_by_task(task_id, &active_statuses)?;

        match desired_fire_at {
            Some(fire_at) => {
                // Check if we already have a job with this exact fire_at.
                let already_correct = active_jobs.iter().any(|j| j.fire_at == fire_at);
                if already_correct {
                    return Ok(()); // Idempotent: nothing to do.
                }
                // Cancel existing jobs (wrong fire_at) and create a new one.
                self.jobs.cancel_active_by_task(task_id)?;
                let job = ReminderJob::new(task_id, fire_at, now);
                self.jobs.insert(&job)?;
            }
            None => {
                // No reminder warranted: cancel all active jobs.
                self.jobs.cancel_active_by_task(task_id)?;
            }
        }

        Ok(())
    }

    /// Reconcile all active TODO tasks. Called on app startup.
    pub fn reconcile_all(&self) -> Result<usize, AppError> {
        let query = crate::repo::TaskQuery {
            statuses: vec![TaskStatus::Todo],
            ..Default::default()
        };
        let tasks = self.tasks.list(&query)?;
        let mut count = 0;
        for task in &tasks {
            self.reconcile_task(task.id)?;
            count += 1;
        }
        Ok(count)
    }

    /// Poll for due jobs and fire them. Returns the number of notifications
    /// successfully delivered.
    pub fn poll_due(&self) -> Result<usize, AppError> {
        let now = self.clock.now_utc();
        let due_jobs = self.jobs.list_due(now, 100)?;
        let mut fired = 0;

        for job in due_jobs {
            // Re-check the task is still valid.
            let task = match self.tasks.get(job.task_id)? {
                Some(t) => t,
                None => {
                    self.jobs.update_status(
                        job.id,
                        ReminderJobStatus::Canceled,
                        Some("task not found"),
                    )?;
                    continue;
                }
            };

            if task.status != TaskStatus::Todo || task.deleted_at.is_some() {
                self.jobs
                    .update_status(job.id, ReminderJobStatus::Canceled, None)?;
                continue;
            }

            match self.notifier.send(&task.title, "DayFlow 任务提醒") {
                Ok(()) => {
                    self.jobs
                        .update_status(job.id, ReminderJobStatus::Fired, None)?;
                    fired += 1;
                }
                Err(e) => {
                    self.jobs.update_status(
                        job.id,
                        ReminderJobStatus::Failed,
                        Some(&e.to_string()),
                    )?;
                }
            }
        }

        Ok(fired)
    }

    /// Cancel all active reminder jobs for a task. Called on complete, cancel,
    /// restore, or soft-delete.
    pub fn cancel_for_task(&self, task_id: Uuid) -> Result<usize, AppError> {
        Ok(self.jobs.cancel_active_by_task(task_id)?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use chrono::{DateTime, TimeZone, Utc};

    use super::*;
    use crate::domain::reminder::{GlobalReminderDefault, ReminderPreference};
    use crate::domain::{Task, TaskPriority, TaskSchedule};
    use crate::repo::TaskQuery;
    use crate::scheduler::FakeNotificationSender;
    use crate::time::FixedClock;

    // --- Fake repositories ---

    struct FakeTaskRepo {
        tasks: Mutex<Vec<Task>>,
    }

    impl FakeTaskRepo {
        fn new() -> Self {
            Self {
                tasks: Mutex::new(Vec::new()),
            }
        }
    }

    impl TaskRepository for FakeTaskRepo {
        fn insert(&self, task: &Task) -> Result<(), RepoError> {
            self.tasks.lock().unwrap().push(task.clone());
            Ok(())
        }

        fn get(&self, id: Uuid) -> Result<Option<Task>, RepoError> {
            Ok(self
                .tasks
                .lock()
                .unwrap()
                .iter()
                .find(|t| t.id == id)
                .cloned())
        }

        fn list(&self, query: &TaskQuery) -> Result<Vec<Task>, RepoError> {
            Ok(self
                .tasks
                .lock()
                .unwrap()
                .iter()
                .filter(|task| {
                    if !query.statuses.is_empty() && !query.statuses.contains(&task.status) {
                        return false;
                    }
                    if !query.include_deleted && task.deleted_at.is_some() {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect())
        }

        fn update(&self, task: &Task, expected_version: i64) -> Result<Task, RepoError> {
            let mut tasks = self.tasks.lock().unwrap();
            match tasks.iter().position(|t| t.id == task.id) {
                None => Err(RepoError::TaskNotFound(task.id)),
                Some(index) => {
                    if tasks[index].version != expected_version {
                        return Err(RepoError::VersionConflict {
                            id: task.id,
                            expected: expected_version,
                        });
                    }
                    tasks[index] = task.clone();
                    Ok(task.clone())
                }
            }
        }

        fn count(&self) -> Result<i64, RepoError> {
            Ok(self.tasks.lock().unwrap().len() as i64)
        }
    }

    struct FakeJobRepo {
        jobs: Mutex<Vec<ReminderJob>>,
    }

    impl FakeJobRepo {
        fn new() -> Self {
            Self {
                jobs: Mutex::new(Vec::new()),
            }
        }

        fn active_for(&self, task_id: Uuid) -> Vec<ReminderJob> {
            self.jobs
                .lock()
                .unwrap()
                .iter()
                .filter(|j| j.task_id == task_id && j.status.is_active())
                .cloned()
                .collect()
        }
    }

    impl ReminderJobRepository for FakeJobRepo {
        fn insert(&self, job: &ReminderJob) -> Result<(), RepoError> {
            self.jobs.lock().unwrap().push(job.clone());
            Ok(())
        }

        fn get(&self, id: Uuid) -> Result<Option<ReminderJob>, RepoError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .find(|j| j.id == id)
                .cloned())
        }

        fn list_by_task(
            &self,
            task_id: Uuid,
            statuses: &[ReminderJobStatus],
        ) -> Result<Vec<ReminderJob>, RepoError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .filter(|j| j.task_id == task_id && statuses.contains(&j.status))
                .cloned()
                .collect())
        }

        fn list_due(
            &self,
            before: DateTime<Utc>,
            limit: i64,
        ) -> Result<Vec<ReminderJob>, RepoError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .filter(|j| j.status.is_active() && j.fire_at <= before)
                .take(limit as usize)
                .cloned()
                .collect())
        }

        fn update_status(
            &self,
            id: Uuid,
            status: ReminderJobStatus,
            last_error: Option<&str>,
        ) -> Result<(), RepoError> {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                job.status = status;
                job.last_error = last_error.map(String::from);
                job.updated_at = Utc::now();
            }
            Ok(())
        }

        fn cancel_active_by_task(&self, task_id: Uuid) -> Result<usize, RepoError> {
            let mut jobs = self.jobs.lock().unwrap();
            let count = jobs
                .iter_mut()
                .filter(|j| j.task_id == task_id && j.status.is_active())
                .count();
            for job in jobs.iter_mut() {
                if job.task_id == task_id && job.status.is_active() {
                    job.status = ReminderJobStatus::Canceled;
                    job.updated_at = Utc::now();
                }
            }
            Ok(count)
        }
    }

    struct FakeSettings {
        default: Mutex<GlobalReminderDefault>,
        strings: Mutex<HashMap<String, String>>,
    }

    impl FakeSettings {
        fn new() -> Self {
            Self {
                default: Mutex::new(GlobalReminderDefault::default()),
                strings: Mutex::new(HashMap::new()),
            }
        }
    }

    impl SettingsRepository for FakeSettings {
        fn get_global_reminder_default(&self) -> Result<GlobalReminderDefault, RepoError> {
            Ok(*self.default.lock().unwrap())
        }

        fn set_global_reminder_default(
            &self,
            value: &GlobalReminderDefault,
        ) -> Result<(), RepoError> {
            *self.default.lock().unwrap() = *value;
            Ok(())
        }

        fn get_string(&self, key: &str) -> Result<Option<String>, RepoError> {
            Ok(self.strings.lock().unwrap().get(key).cloned())
        }

        fn set_string(&self, key: &str, value: &str) -> Result<(), RepoError> {
            self.strings
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }
    }

    type TestService = ReminderService<
        FakeTaskRepo,
        FakeSettings,
        FakeJobRepo,
        FakeNotificationSender,
        FixedClock,
    >;

    fn setup() -> (
        TestService,
        Arc<FakeTaskRepo>,
        Arc<FakeSettings>,
        Arc<FakeJobRepo>,
        Arc<FakeNotificationSender>,
    ) {
        let tasks = Arc::new(FakeTaskRepo::new());
        let settings = Arc::new(FakeSettings::new());
        let jobs = Arc::new(FakeJobRepo::new());
        let notifier = Arc::new(FakeNotificationSender::new());
        // 2026-10-26 00:00 UTC, zone UTC for simplicity.
        let instant = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let zone: chrono_tz::Tz = "UTC".parse().unwrap();
        let clock = Arc::new(FixedClock::new(instant, zone));
        let service = TestService::new(
            tasks.clone(),
            settings.clone(),
            jobs.clone(),
            notifier.clone(),
            clock,
        );
        (service, tasks, settings, jobs, notifier)
    }

    fn make_timed_task(id: Uuid, date: &str, time: &str) -> Task {
        let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let t = chrono::NaiveTime::parse_from_str(time, "%H:%M").unwrap();
        let schedule = TaskSchedule {
            date: d,
            time: Some(t),
            all_day: false,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        Task::new(
            id,
            "test task".to_string(),
            schedule,
            TaskPriority::Normal,
            None,
            ReminderPreference::InheritDefault,
            now,
        )
        .unwrap()
    }

    #[test]
    fn reconcile_creates_job_for_timed_task() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();

        let active = jobs.active_for(id);
        assert_eq!(active.len(), 1);
        // 14:00 UTC - 10 min (default) = 13:50 UTC
        assert_eq!(
            active[0].fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 50, 0).unwrap()
        );
    }

    #[test]
    fn reconcile_is_idempotent() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        service.reconcile_task(id).unwrap();
        service.reconcile_task(id).unwrap();

        let active = jobs.active_for(id);
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn reconcile_cancels_job_when_task_completed() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 1);

        // Complete the task.
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let mut completed = task.clone();
        completed.complete(now).unwrap();
        tasks.update(&completed, task.version).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 0);
    }

    #[test]
    fn reconcile_cancels_job_when_task_canceled() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 1);

        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let mut canceled = task.clone();
        canceled.cancel(now).unwrap();
        tasks.update(&canceled, task.version).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 0);
    }

    #[test]
    fn reconcile_cancels_job_when_task_deleted() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 1);

        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let mut deleted = task.clone();
        deleted.soft_delete(now).unwrap();
        tasks.update(&deleted, task.version).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 0);
    }

    #[test]
    fn reconcile_no_job_for_untimed_task() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let d = chrono::NaiveDate::parse_from_str("2026-10-26", "%Y-%m-%d").unwrap();
        let schedule = TaskSchedule {
            date: d,
            time: None,
            all_day: false,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let task = Task::new(
            id,
            "untimed".to_string(),
            schedule,
            TaskPriority::Normal,
            None,
            ReminderPreference::InheritDefault,
            now,
        )
        .unwrap();
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 0);
    }

    #[test]
    fn reconcile_no_job_for_disabled_preference() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let d = chrono::NaiveDate::parse_from_str("2026-10-26", "%Y-%m-%d").unwrap();
        let t = chrono::NaiveTime::parse_from_str("14:00", "%H:%M").unwrap();
        let schedule = TaskSchedule {
            date: d,
            time: Some(t),
            all_day: false,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let task = Task::new(
            id,
            "disabled".to_string(),
            schedule,
            TaskPriority::Normal,
            None,
            ReminderPreference::Disabled,
            now,
        )
        .unwrap();
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        assert_eq!(jobs.active_for(id).len(), 0);
    }

    #[test]
    fn reconcile_updates_job_when_rescheduled() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        let before = jobs.active_for(id);
        assert_eq!(before.len(), 1);

        // Reschedule to a different time.
        let new_date = chrono::NaiveDate::parse_from_str("2026-10-27", "%Y-%m-%d").unwrap();
        let mut rescheduled = task.clone();
        rescheduled.reschedule(new_date, None, Utc::now()).unwrap();
        tasks.update(&rescheduled, task.version).unwrap();

        service.reconcile_task(id).unwrap();
        let after = jobs.active_for(id);
        assert_eq!(after.len(), 1);
        // New fire_at should be different (next day).
        assert_ne!(before[0].fire_at, after[0].fire_at);
    }

    #[test]
    fn poll_due_fires_notifications() {
        let (service, tasks, _settings, jobs, notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        // Create a job that is already due.
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let job = ReminderJob::new(id, now - chrono::Duration::minutes(1), now);
        jobs.insert(&job).unwrap();

        let fired = service.poll_due().unwrap();
        assert_eq!(fired, 1);
        assert_eq!(notifier.sent_count(), 1);

        let loaded = jobs.get(job.id).unwrap().unwrap();
        assert_eq!(loaded.status, ReminderJobStatus::Fired);
    }

    #[test]
    fn poll_due_skips_completed_tasks() {
        let (service, tasks, _settings, jobs, notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        // Complete the task.
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let mut completed = task.clone();
        completed.complete(now).unwrap();
        tasks.update(&completed, task.version).unwrap();

        // Create a due job.
        let job = ReminderJob::new(id, now - chrono::Duration::minutes(1), now);
        jobs.insert(&job).unwrap();

        let fired = service.poll_due().unwrap();
        assert_eq!(fired, 0);
        assert_eq!(notifier.sent_count(), 0);

        let loaded = jobs.get(job.id).unwrap().unwrap();
        assert_eq!(loaded.status, ReminderJobStatus::Canceled);
    }

    #[test]
    fn poll_due_marks_failed_on_sender_error() {
        let (service, tasks, _settings, jobs, notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        notifier.set_fail(true);

        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let job = ReminderJob::new(id, now - chrono::Duration::minutes(1), now);
        jobs.insert(&job).unwrap();

        let fired = service.poll_due().unwrap();
        assert_eq!(fired, 0);

        let loaded = jobs.get(job.id).unwrap().unwrap();
        assert_eq!(loaded.status, ReminderJobStatus::Failed);
        assert!(loaded.last_error.is_some());
    }

    #[test]
    fn reconcile_all_processes_all_todo_tasks() {
        let (service, tasks, _settings, jobs, _notifier) = setup();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let task1 = make_timed_task(id1, "2026-10-26", "10:00");
        let task2 = make_timed_task(id2, "2026-10-26", "15:00");
        tasks.insert(&task1).unwrap();
        tasks.insert(&task2).unwrap();

        let count = service.reconcile_all().unwrap();
        assert_eq!(count, 2);
        assert_eq!(jobs.active_for(id1).len(), 1);
        assert_eq!(jobs.active_for(id2).len(), 1);
    }

    #[test]
    fn global_default_change_affects_inherit_tasks() {
        let (service, tasks, settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let task = make_timed_task(id, "2026-10-26", "14:00");
        tasks.insert(&task).unwrap();

        // Default is 10 min → fire at 13:50.
        service.reconcile_task(id).unwrap();
        let before = jobs.active_for(id);
        assert_eq!(
            before[0].fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 50, 0).unwrap()
        );

        // Change global default to 30 min.
        settings
            .set_global_reminder_default(&GlobalReminderDefault {
                enabled: true,
                minutes_before: 30,
            })
            .unwrap();

        service.reconcile_task(id).unwrap();
        let after = jobs.active_for(id);
        assert_eq!(after.len(), 1);
        assert_eq!(
            after[0].fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 30, 0).unwrap()
        );
    }

    #[test]
    fn custom_preference_unaffected_by_global_change() {
        let (service, tasks, settings, jobs, _notifier) = setup();
        let id = Uuid::new_v4();
        let d = chrono::NaiveDate::parse_from_str("2026-10-26", "%Y-%m-%d").unwrap();
        let t = chrono::NaiveTime::parse_from_str("14:00", "%H:%M").unwrap();
        let schedule = TaskSchedule {
            date: d,
            time: Some(t),
            all_day: false,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let task = Task::new(
            id,
            "custom".to_string(),
            schedule,
            TaskPriority::Normal,
            None,
            ReminderPreference::Custom { minutes_before: 15 },
            now,
        )
        .unwrap();
        tasks.insert(&task).unwrap();

        service.reconcile_task(id).unwrap();
        let before = jobs.active_for(id);
        assert_eq!(
            before[0].fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 45, 0).unwrap()
        );

        // Change global default — should not affect custom.
        settings
            .set_global_reminder_default(&GlobalReminderDefault {
                enabled: true,
                minutes_before: 99,
            })
            .unwrap();

        service.reconcile_task(id).unwrap();
        let after = jobs.active_for(id);
        assert_eq!(after.len(), 1);
        assert_eq!(
            after[0].fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 45, 0).unwrap()
        );
    }
}
