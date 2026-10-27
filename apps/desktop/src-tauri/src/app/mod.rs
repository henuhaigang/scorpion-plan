use std::sync::Arc;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

use crate::domain::{
    DomainError, GlobalReminderDefault, ReminderPreference, Task, TaskPriority, TaskSchedule,
    TaskStatus, TaskView, TaskViewPolicy,
};
use crate::repo::sqlite::{SqliteSettingsRepository, SqliteTaskRepository};
use crate::repo::{RepoError, SettingsRepository, TaskQuery, TaskRepository};
use crate::time::{Clock, SystemClock};

/// Application-layer failures. Domain and repository errors are preserved so
/// callers can distinguish validation problems from persistence problems.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("repository error: {0}")]
    Repo(#[from] RepoError),
}

#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub id: Uuid,
    pub title: String,
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub all_day: bool,
    pub priority: TaskPriority,
    pub category_id: Option<Uuid>,
    pub reminder_preference: ReminderPreference,
}

impl CreateTaskCommand {
    /// Quick-add defaults: today, no time, normal priority, no category,
    /// reminder inherits the global default.
    pub fn quick_add(id: Uuid, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            date: None,
            time: None,
            all_day: false,
            priority: TaskPriority::Normal,
            category_id: None,
            reminder_preference: ReminderPreference::InheritDefault,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTaskCommand {
    pub title: Option<String>,
    pub priority: Option<TaskPriority>,
    pub category_id: Option<Option<Uuid>>,
    pub reminder_preference: Option<ReminderPreference>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TodayView {
    pub timed: Vec<TaskView>,
    pub untimed: Vec<TaskView>,
}

/// Application use cases for the local task core.
///
/// The service owns no business rules itself: it validates input, applies
/// domain behavior with an injected clock, and persists through repository
/// ports. UI code calls this boundary instead of touching SQL or domain
/// invariants directly.
pub struct TaskService<R: TaskRepository, S: SettingsRepository, C: Clock> {
    tasks: Arc<R>,
    settings: Arc<S>,
    clock: Arc<C>,
}

pub type DesktopTaskService =
    TaskService<SqliteTaskRepository, SqliteSettingsRepository, SystemClock>;

pub const DRAFT_KEY: &str = "draft.quick_add";

impl<R: TaskRepository, S: SettingsRepository, C: Clock> TaskService<R, S, C> {
    pub fn new(tasks: Arc<R>, settings: Arc<S>, clock: Arc<C>) -> Self {
        Self {
            tasks,
            settings,
            clock,
        }
    }

    pub fn get_task(&self, id: Uuid) -> Result<Task, AppError> {
        Ok(self
            .tasks
            .get(id)?
            .ok_or_else(|| RepoError::TaskNotFound(id))?)
    }

    pub fn create_task(&self, cmd: CreateTaskCommand) -> Result<Task, AppError> {
        if let Some(existing) = self.tasks.get(cmd.id)? {
            return Ok(existing);
        }

        let now = self.clock.now_utc();
        let date = cmd.date.unwrap_or_else(|| self.clock.today_local());
        let schedule = TaskSchedule {
            date,
            time: cmd.time,
            all_day: cmd.all_day,
        };

        let task = Task::new(
            cmd.id,
            cmd.title,
            schedule,
            cmd.priority,
            cmd.category_id,
            cmd.reminder_preference,
            now,
        )?;
        self.tasks.insert(&task)?;
        Ok(task)
    }

    pub fn update_task(
        &self,
        id: Uuid,
        expected_version: i64,
        cmd: UpdateTaskCommand,
    ) -> Result<Task, AppError> {
        let has_changes = cmd.title.is_some()
            || cmd.priority.is_some()
            || cmd.category_id.is_some()
            || cmd.reminder_preference.is_some();
        if !has_changes {
            return self.get_task(id);
        }

        self.mutate(id, expected_version, |task, now| {
            task.update_fields(
                cmd.title,
                cmd.priority,
                cmd.category_id,
                cmd.reminder_preference,
                now,
            )
        })
    }

    pub fn reschedule_task(
        &self,
        id: Uuid,
        expected_version: i64,
        date: NaiveDate,
        time: Option<Option<NaiveTime>>,
    ) -> Result<Task, AppError> {
        self.mutate(id, expected_version, |task, now| {
            task.reschedule(date, time, now)
        })
    }

    pub fn complete_task(&self, id: Uuid, expected_version: i64) -> Result<Task, AppError> {
        self.mutate(id, expected_version, |task, now| task.complete(now))
    }

    pub fn undo_completion(&self, id: Uuid, expected_version: i64) -> Result<Task, AppError> {
        self.mutate(id, expected_version, |task, now| task.undo_completion(now))
    }

    pub fn cancel_task(&self, id: Uuid, expected_version: i64) -> Result<Task, AppError> {
        self.mutate(id, expected_version, |task, now| task.cancel(now))
    }

    pub fn restore_canceled_task(&self, id: Uuid, expected_version: i64) -> Result<Task, AppError> {
        self.mutate(id, expected_version, |task, now| task.restore(now))
    }

    pub fn soft_delete_task(&self, id: Uuid, expected_version: i64) -> Result<Task, AppError> {
        self.mutate(id, expected_version, |task, now| task.soft_delete(now))
    }

    pub fn list_today(&self) -> Result<TodayView, AppError> {
        let today = self.clock.today_local();
        let now_local = self.clock.now_local();
        let query = TaskQuery {
            statuses: vec![TaskStatus::Todo],
            date_eq: Some(today),
            ..Default::default()
        };
        let tasks = self.tasks.list(&query)?;
        let (timed, untimed) = TaskViewPolicy::split_today(&tasks, now_local);
        Ok(TodayView { timed, untimed })
    }

    pub fn list_legacy(&self) -> Result<Vec<TaskView>, AppError> {
        let today = self.clock.today_local();
        let now_local = self.clock.now_local();
        let query = TaskQuery {
            statuses: vec![TaskStatus::Todo],
            date_before: Some(today),
            ..Default::default()
        };
        let tasks = self.tasks.list(&query)?;
        Ok(Self::sorted_views(tasks, now_local))
    }

    pub fn list_future(&self) -> Result<Vec<TaskView>, AppError> {
        let today = self.clock.today_local();
        let now_local = self.clock.now_local();
        let query = TaskQuery {
            statuses: vec![TaskStatus::Todo],
            date_after: Some(today),
            ..Default::default()
        };
        let tasks = self.tasks.list(&query)?;
        Ok(Self::sorted_views(tasks, now_local))
    }

    pub fn global_reminder_default(&self) -> Result<GlobalReminderDefault, AppError> {
        Ok(self.settings.get_global_reminder_default()?)
    }

    pub fn set_global_reminder_default(
        &self,
        value: &GlobalReminderDefault,
    ) -> Result<(), AppError> {
        self.settings.set_global_reminder_default(value)?;
        Ok(())
    }

    pub fn get_draft(&self) -> Result<String, AppError> {
        Ok(self.settings.get_string(DRAFT_KEY)?.unwrap_or_default())
    }

    pub fn set_draft(&self, value: &str) -> Result<(), AppError> {
        self.settings.set_string(DRAFT_KEY, value)?;
        Ok(())
    }

    fn mutate<F>(&self, id: Uuid, expected_version: i64, f: F) -> Result<Task, AppError>
    where
        F: FnOnce(&mut Task, DateTime<Utc>) -> Result<(), DomainError>,
    {
        let current = self.get_task(id)?;
        if current.deleted_at.is_some() {
            return Err(RepoError::TaskDeleted(id).into());
        }

        let mut next = current;
        f(&mut next, self.clock.now_utc())?;
        Ok(self.tasks.update(&next, expected_version)?)
    }

    fn sorted_views(
        tasks: Vec<Task>,
        now_local: chrono::DateTime<chrono::FixedOffset>,
    ) -> Vec<TaskView> {
        let mut views = tasks
            .into_iter()
            .map(|task| TaskViewPolicy::view(&task, now_local))
            .collect::<Vec<_>>();
        views.sort_by_key(|a| schedule_sort_key(&a.task));
        views
    }
}

fn schedule_sort_key(task: &Task) -> (NaiveDate, NaiveTime, DateTime<Utc>, Uuid) {
    (
        task.current_schedule.date,
        task.current_schedule
            .time
            .unwrap_or(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap()),
        task.created_at,
        task.id,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use chrono::TimeZone;

    use super::*;
    use crate::domain::TaskBucket;
    use crate::time::FixedClock;

    struct FakeRepo {
        tasks: Mutex<Vec<Task>>,
        inserts: Mutex<usize>,
    }

    impl FakeRepo {
        fn new() -> Self {
            Self {
                tasks: Mutex::new(Vec::new()),
                inserts: Mutex::new(0),
            }
        }

        fn insert_count(&self) -> usize {
            *self.inserts.lock().unwrap()
        }
    }

    impl TaskRepository for FakeRepo {
        fn insert(&self, task: &Task) -> Result<(), RepoError> {
            self.tasks.lock().unwrap().push(task.clone());
            *self.inserts.lock().unwrap() += 1;
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
                    if let Some(date) = query.date_before {
                        if task.current_schedule.date >= date {
                            return false;
                        }
                    }
                    if let Some(date) = query.date_after {
                        if task.current_schedule.date <= date {
                            return false;
                        }
                    }
                    if let Some(date) = query.date_eq {
                        if task.current_schedule.date != date {
                            return false;
                        }
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
                    let stored = tasks[index].clone();
                    if stored.deleted_at.is_some() {
                        return Err(RepoError::TaskDeleted(task.id));
                    }
                    if stored.version != expected_version {
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

    type TestService = TaskService<FakeRepo, FakeSettings, FixedClock>;

    fn service_at(instant: DateTime<Utc>) -> (TestService, Arc<FakeRepo>, Arc<FakeSettings>) {
        let repo = Arc::new(FakeRepo::new());
        let settings = Arc::new(FakeSettings::new());
        let zone: chrono_tz::Tz = "Asia/Shanghai".parse().unwrap();
        let clock = Arc::new(FixedClock::new(instant, zone));
        let service = TestService::new(repo.clone(), settings.clone(), clock);
        (service, repo, settings)
    }

    fn now() -> DateTime<Utc> {
        // 12:00 local time in Asia/Shanghai.
        Utc.with_ymd_and_hms(2026, 10, 26, 4, 0, 0).unwrap()
    }

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn time(s: &str) -> NaiveTime {
        NaiveTime::parse_from_str(s, "%H:%M").unwrap()
    }

    fn quick_add(id: Uuid, title: &str) -> CreateTaskCommand {
        CreateTaskCommand::quick_add(id, title)
    }

    #[test]
    fn create_task_uses_quick_add_defaults() {
        let (service, repo, _settings) = service_at(now());
        let id = Uuid::new_v4();

        let task = service.create_task(quick_add(id, "  写周报 ")).unwrap();

        assert_eq!(task.id, id);
        assert_eq!(task.title, "写周报");
        assert_eq!(task.current_schedule.date, date("2026-10-26"));
        assert_eq!(task.current_schedule.time, None);
        assert!(!task.current_schedule.all_day);
        assert_eq!(task.priority, TaskPriority::Normal);
        assert_eq!(task.reminder_preference, ReminderPreference::InheritDefault);
        assert_eq!(task.original_schedule, task.current_schedule);
        assert_eq!(repo.insert_count(), 1);
    }

    #[test]
    fn create_task_rejects_empty_and_oversized_titles() {
        let (service, _repo, _settings) = service_at(now());

        assert!(matches!(
            service.create_task(quick_add(Uuid::new_v4(), "   ")),
            Err(AppError::Domain(DomainError::EmptyTitle))
        ));

        let long: String = "中".repeat(201);
        assert!(matches!(
            service.create_task(quick_add(Uuid::new_v4(), &long)),
            Err(AppError::Domain(DomainError::TitleTooLong))
        ));
    }

    #[test]
    fn duplicate_create_with_same_id_is_idempotent() {
        let (service, repo, _settings) = service_at(now());
        let id = Uuid::new_v4();
        let command = quick_add(id, "只创建一次");

        let first = service.create_task(command.clone()).unwrap();
        let second = service.create_task(command).unwrap();

        assert_eq!(first, second);
        assert_eq!(repo.insert_count(), 1);
    }

    #[test]
    fn complete_and_undo_completion_update_lifecycle() {
        let (service, _repo, _settings) = service_at(now());
        let task = service
            .create_task(quick_add(Uuid::new_v4(), "完成"))
            .unwrap();

        let completed = service.complete_task(task.id, task.version).unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
        assert!(completed.completed_at.is_some());
        assert_eq!(completed.version, 2);

        let restored = service
            .undo_completion(completed.id, completed.version)
            .unwrap();
        assert_eq!(restored.status, TaskStatus::Todo);
        assert!(restored.completed_at.is_none());
        assert_eq!(restored.version, 3);
    }

    #[test]
    fn cancel_and_restore_canceled_task() {
        let (service, _repo, _settings) = service_at(now());
        let task = service
            .create_task(quick_add(Uuid::new_v4(), "取消"))
            .unwrap();

        let canceled = service.cancel_task(task.id, task.version).unwrap();
        assert_eq!(canceled.status, TaskStatus::Canceled);

        let restored = service
            .restore_canceled_task(canceled.id, canceled.version)
            .unwrap();
        assert_eq!(restored.status, TaskStatus::Todo);
    }

    #[test]
    fn stale_expected_version_returns_conflict() {
        let (service, _repo, _settings) = service_at(now());
        let task = service
            .create_task(quick_add(Uuid::new_v4(), "冲突"))
            .unwrap();

        let err = service.complete_task(task.id, 99).unwrap_err();
        assert!(matches!(
            err,
            AppError::Repo(RepoError::VersionConflict { .. })
        ));
    }

    #[test]
    fn soft_delete_preserves_tombstone_and_hides_task_from_today() {
        let (service, _repo, _settings) = service_at(now());
        let task = service
            .create_task(quick_add(Uuid::new_v4(), "删除"))
            .unwrap();

        let deleted = service.soft_delete_task(task.id, task.version).unwrap();
        assert!(deleted.deleted_at.is_some());

        let loaded = service.get_task(task.id).unwrap();
        assert!(loaded.deleted_at.is_some());

        assert!(service.list_today().unwrap().timed.is_empty());
        assert!(service.list_today().unwrap().untimed.is_empty());
    }

    #[test]
    fn update_task_trims_title_and_bumps_version() {
        let (service, _repo, _settings) = service_at(now());
        let task = service
            .create_task(quick_add(Uuid::new_v4(), "旧标题"))
            .unwrap();

        let updated = service
            .update_task(
                task.id,
                task.version,
                UpdateTaskCommand {
                    title: Some("  新标题  ".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "新标题");
        assert_eq!(updated.version, 2);
    }

    #[test]
    fn reschedule_changes_current_schedule_only() {
        let (service, _repo, _settings) = service_at(now());
        let mut cmd = quick_add(Uuid::new_v4(), "改期");
        cmd.time = Some(time("09:00"));
        let task = service.create_task(cmd).unwrap();

        let rescheduled = service
            .reschedule_task(task.id, task.version, date("2026-11-02"), None)
            .unwrap();

        assert_eq!(rescheduled.current_schedule.date, date("2026-11-02"));
        assert_eq!(rescheduled.current_schedule.time, Some(time("09:00")));
        assert_eq!(rescheduled.original_schedule.date, date("2026-10-26"));
        assert_eq!(rescheduled.original_schedule.time, Some(time("09:00")));
    }

    #[test]
    fn list_today_splits_timed_and_untimed_tasks() {
        let (service, _repo, _settings) = service_at(now());

        let early = Uuid::new_v4();
        let mut early_cmd = quick_add(early, "早");
        early_cmd.time = Some(time("09:00"));
        service.create_task(early_cmd).unwrap();

        let late = Uuid::new_v4();
        let mut late_cmd = quick_add(late, "晚");
        late_cmd.time = Some(time("15:00"));
        service.create_task(late_cmd).unwrap();

        let untimed = Uuid::new_v4();
        service.create_task(quick_add(untimed, "待安排")).unwrap();

        let view = service.list_today().unwrap();

        assert_eq!(view.timed.len(), 2);
        assert_eq!(view.timed[0].task.id, early);
        assert!(view.timed[0].is_overdue);
        assert_eq!(view.timed[1].task.id, late);
        assert!(!view.timed[1].is_overdue);
        assert_eq!(view.untimed.len(), 1);
        assert_eq!(view.untimed[0].task.id, untimed);
        assert!(!view.untimed[0].is_overdue);
    }

    #[test]
    fn list_today_excludes_legacy_future_and_terminal_tasks() {
        let (service, _repo, _settings) = service_at(now());

        let legacy = Uuid::new_v4();
        let mut legacy_cmd = quick_add(legacy, "之前");
        legacy_cmd.date = Some(date("2026-10-01"));
        service.create_task(legacy_cmd).unwrap();

        let future = Uuid::new_v4();
        let mut future_cmd = quick_add(future, "未来");
        future_cmd.date = Some(date("2026-11-02"));
        service.create_task(future_cmd).unwrap();

        let completed = Uuid::new_v4();
        let completed_task = service.create_task(quick_add(completed, "已完成")).unwrap();
        service
            .complete_task(completed, completed_task.version)
            .unwrap();

        let today = service.list_today().unwrap();
        assert!(today.timed.is_empty());
        assert!(today.untimed.is_empty());
    }

    #[test]
    fn list_legacy_contains_only_past_todo_tasks() {
        let (service, _repo, _settings) = service_at(now());

        let legacy = Uuid::new_v4();
        let mut legacy_cmd = quick_add(legacy, "之前未完成");
        legacy_cmd.date = Some(date("2026-10-01"));
        service.create_task(legacy_cmd).unwrap();

        let today = Uuid::new_v4();
        service.create_task(quick_add(today, "今天")).unwrap();

        let legacy_tasks = service.list_legacy().unwrap();
        assert_eq!(legacy_tasks.len(), 1);
        assert_eq!(legacy_tasks[0].task.id, legacy);
        assert_eq!(legacy_tasks[0].bucket, TaskBucket::Legacy);
    }

    #[test]
    fn draft_round_trips_through_settings() {
        let (service, _repo, settings) = service_at(now());

        service.set_draft("写周报").unwrap();

        assert_eq!(service.get_draft().unwrap(), "写周报");
        assert_eq!(
            settings
                .strings
                .lock()
                .unwrap()
                .get(DRAFT_KEY)
                .map(String::as_str),
            Some("写周报")
        );
    }

    #[test]
    fn global_reminder_default_round_trips_through_settings() {
        let (service, _repo, settings) = service_at(now());
        let value = GlobalReminderDefault {
            enabled: false,
            minutes_before: 0,
        };

        service.set_global_reminder_default(&value).unwrap();
        assert_eq!(service.global_reminder_default().unwrap(), value);
        assert_eq!(*settings.default.lock().unwrap(), value);
    }
}
