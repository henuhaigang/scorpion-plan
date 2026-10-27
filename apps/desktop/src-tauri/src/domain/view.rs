use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};

use super::task::{Task, TaskStatus};

/// Derived view buckets. Never persisted as task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskBucket {
    Today,
    Future,
    Legacy,
    Completed,
    Canceled,
    Deleted,
}

/// A task annotated with its derived view state.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskView {
    pub task: Task,
    pub bucket: TaskBucket,
    pub is_overdue: bool,
}

/// Pure derived-view rules. No I/O, no system time: `today` and `now_local`
/// are injected so tests are deterministic.
pub struct TaskViewPolicy;

impl TaskViewPolicy {
    /// Bucket for any task (including terminal states).
    pub fn bucket(task: &Task, today: NaiveDate) -> TaskBucket {
        if task.deleted_at.is_some() {
            return TaskBucket::Deleted;
        }
        match task.status {
            TaskStatus::Completed => TaskBucket::Completed,
            TaskStatus::Canceled => TaskBucket::Canceled,
            TaskStatus::Todo => {
                let d = task.current_schedule.date;
                if d < today {
                    TaskBucket::Legacy
                } else if d > today {
                    TaskBucket::Future
                } else {
                    TaskBucket::Today
                }
            }
        }
    }

    /// A timed task is overdue only if scheduled today and its time is earlier
    /// than the current local time. Untimed tasks are never overdue.
    /// Completed/canceled/deleted tasks are never overdue.
    pub fn is_overdue(task: &Task, now_local: DateTime<FixedOffset>) -> bool {
        if task.status != TaskStatus::Todo || task.deleted_at.is_some() {
            return false;
        }
        let today = now_local.date_naive();
        if task.current_schedule.date != today {
            return false;
        }
        match task.current_schedule.time {
            Some(t) => t < now_local.time(),
            None => false,
        }
    }

    /// Annotate a task with bucket + overdue for a given "now".
    pub fn view(task: &Task, now_local: DateTime<FixedOffset>) -> TaskView {
        let today = now_local.date_naive();
        TaskView {
            task: task.clone(),
            bucket: Self::bucket(task, today),
            is_overdue: Self::is_overdue(task, now_local),
        }
    }

    /// Sort key for timed tasks: scheduled time, then createdAt, then id.
    pub fn timed_sort_key(task: &Task) -> (NaiveTime, chrono::DateTime<chrono::Utc>, uuid::Uuid) {
        (
            task.current_schedule
                .time
                .unwrap_or(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap()),
            task.created_at,
            task.id,
        )
    }

    /// Sort key for untimed tasks: createdAt, then id.
    pub fn untimed_sort_key(task: &Task) -> (chrono::DateTime<chrono::Utc>, uuid::Uuid) {
        (task.created_at, task.id)
    }

    /// Split active TODO tasks into (timed, untimed) for the today view,
    /// each sorted per the stable tie-breaker rules.
    pub fn split_today(
        tasks: &[Task],
        now_local: DateTime<FixedOffset>,
    ) -> (Vec<TaskView>, Vec<TaskView>) {
        let today = now_local.date_naive();
        let mut timed: Vec<&Task> = tasks
            .iter()
            .filter(|t| {
                Self::bucket(t, today) == TaskBucket::Today && t.current_schedule.time.is_some()
            })
            .collect();
        let mut untimed: Vec<&Task> = tasks
            .iter()
            .filter(|t| {
                Self::bucket(t, today) == TaskBucket::Today && t.current_schedule.time.is_none()
            })
            .collect();

        timed.sort_by_key(|t| Self::timed_sort_key(t));
        untimed.sort_by_key(|t| Self::untimed_sort_key(t));

        (
            timed
                .into_iter()
                .map(|t| Self::view(t, now_local))
                .collect(),
            untimed
                .into_iter()
                .map(|t| Self::view(t, now_local))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::reminder::ReminderPreference;
    use crate::domain::schedule::TaskSchedule;
    use crate::domain::task::TaskPriority;
    use chrono::Utc;
    use uuid::Uuid;

    fn at(s: &str) -> DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn make_task(date: &str, time: Option<&str>, created: &str, id: Uuid) -> Task {
        let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let t = time.map(|s| NaiveTime::parse_from_str(s, "%H:%M").unwrap());
        Task::new(
            id,
            "t".into(),
            TaskSchedule {
                date: d,
                time: t,
                all_day: false,
            },
            TaskPriority::Normal,
            None,
            ReminderPreference::InheritDefault,
            at(created),
        )
        .unwrap()
    }

    /// now = 2026-10-26 14:30 local (UTC+8)
    fn now_local() -> DateTime<FixedOffset> {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        chrono::DateTime::parse_from_rfc3339("2026-10-26T14:30:00+08:00")
            .unwrap()
            .with_timezone(&offset)
    }

    #[test]
    fn buckets_by_date_relative_to_today() {
        let now = now_local();
        let today = now.date_naive();
        let past = make_task(
            "2026-10-20",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        let today_t = make_task(
            "2026-10-26",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        let future = make_task(
            "2026-11-02",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );

        assert_eq!(TaskViewPolicy::bucket(&past, today), TaskBucket::Legacy);
        assert_eq!(TaskViewPolicy::bucket(&today_t, today), TaskBucket::Today);
        assert_eq!(TaskViewPolicy::bucket(&future, today), TaskBucket::Future);
    }

    #[test]
    fn overdue_only_for_timed_today_tasks_before_now() {
        let now = now_local();
        // 09:00 today < 14:30 => overdue
        let early = make_task(
            "2026-10-26",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        // 15:00 today > 14:30 => not overdue
        let late = make_task(
            "2026-10-26",
            Some("15:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        // untimed today => never overdue
        let untimed = make_task("2026-10-26", None, "2026-10-01T00:00:00Z", Uuid::new_v4());
        // timed but yesterday => legacy, not overdue
        let yesterday = make_task(
            "2026-10-25",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );

        assert!(TaskViewPolicy::is_overdue(&early, now));
        assert!(!TaskViewPolicy::is_overdue(&late, now));
        assert!(!TaskViewPolicy::is_overdue(&untimed, now));
        assert!(!TaskViewPolicy::is_overdue(&yesterday, now));
    }

    #[test]
    fn terminal_states_are_never_overdue() {
        let now = now_local();
        let mut completed = make_task(
            "2026-10-26",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        completed.complete(at("2026-10-26T01:00:00Z")).unwrap();
        assert!(!TaskViewPolicy::is_overdue(&completed, now));

        let mut canceled = make_task(
            "2026-10-26",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        canceled.cancel(at("2026-10-26T01:00:00Z")).unwrap();
        assert!(!TaskViewPolicy::is_overdue(&canceled, now));

        let mut deleted = make_task(
            "2026-10-26",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        deleted.soft_delete(at("2026-10-26T01:00:00Z")).unwrap();
        assert!(!TaskViewPolicy::is_overdue(&deleted, now));
    }

    #[test]
    fn timed_tasks_sort_by_time_then_created_then_id() {
        let now = now_local();
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let id_c = Uuid::new_v4();
        // Same time 09:00, different created_at
        let a = make_task("2026-10-26", Some("09:00"), "2026-10-01T00:00:00Z", id_a);
        let b = make_task("2026-10-26", Some("09:00"), "2026-10-02T00:00:00Z", id_b);
        // Earlier time 08:00
        let c = make_task("2026-10-26", Some("08:00"), "2026-10-03T00:00:00Z", id_c);

        let (timed, _) = TaskViewPolicy::split_today(&[a, b, c], now);
        let ids: Vec<Uuid> = timed.iter().map(|v| v.task.id).collect();
        assert_eq!(ids, vec![id_c, id_a, id_b]);
    }

    #[test]
    fn untimed_tasks_go_to_unscheduled_group() {
        let now = now_local();
        let untimed = make_task("2026-10-26", None, "2026-10-01T00:00:00Z", Uuid::new_v4());
        let timed = make_task(
            "2026-10-26",
            Some("09:00"),
            "2026-10-01T00:00:00Z",
            Uuid::new_v4(),
        );
        let (timed_v, untimed_v) = TaskViewPolicy::split_today(&[untimed, timed], now);
        assert_eq!(timed_v.len(), 1);
        assert_eq!(untimed_v.len(), 1);
        assert!(untimed_v[0].task.current_schedule.time.is_none());
    }

    #[test]
    fn legacy_tasks_are_not_mixed_into_today() {
        let now = now_local();
        let legacy = make_task(
            "2026-10-01",
            Some("09:00"),
            "2026-09-01T00:00:00Z",
            Uuid::new_v4(),
        );
        let (timed, untimed) = TaskViewPolicy::split_today(&[legacy], now);
        assert!(timed.is_empty());
        assert!(untimed.is_empty());
    }
}
