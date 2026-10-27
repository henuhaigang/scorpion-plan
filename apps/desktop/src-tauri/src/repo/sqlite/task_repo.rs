use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use super::SqliteConnection;

use crate::domain::{
    GlobalReminderDefault, ReminderPreference, Task, TaskPriority, TaskSchedule, TaskStatus,
};
use crate::repo::{
    Category, CategoryRepository, RepoError, SettingsRepository, TaskQuery, TaskRepository,
};

fn date_str(d: chrono::NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn time_str(t: chrono::NaiveTime) -> String {
    t.format("%H:%M:%S").to_string()
}

fn parse_date(s: &str) -> Result<chrono::NaiveDate, RepoError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(db_error)
}

fn parse_time(s: &str) -> Result<chrono::NaiveTime, RepoError> {
    chrono::NaiveTime::parse_from_str(s, "%H:%M:%S")
        .or_else(|_| chrono::NaiveTime::parse_from_str(s, "%H:%M"))
        .map_err(db_error)
}

fn parse_instant(s: &str) -> Result<chrono::DateTime<chrono::Utc>, RepoError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(db_error)
}

fn instant_str(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.to_rfc3339()
}

fn status_str(s: TaskStatus) -> &'static str {
    match s {
        TaskStatus::Todo => "TODO",
        TaskStatus::Completed => "COMPLETED",
        TaskStatus::Canceled => "CANCELED",
    }
}

fn parse_status(s: &str) -> Result<TaskStatus, RepoError> {
    match s {
        "TODO" => Ok(TaskStatus::Todo),
        "COMPLETED" => Ok(TaskStatus::Completed),
        "CANCELED" => Ok(TaskStatus::Canceled),
        other => Err(db_error(format!("bad status {other}"))),
    }
}

fn priority_str(p: TaskPriority) -> &'static str {
    match p {
        TaskPriority::Low => "LOW",
        TaskPriority::Normal => "NORMAL",
        TaskPriority::High => "HIGH",
    }
}

fn parse_priority(s: &str) -> Result<TaskPriority, RepoError> {
    match s {
        "LOW" => Ok(TaskPriority::Low),
        "NORMAL" => Ok(TaskPriority::Normal),
        "HIGH" => Ok(TaskPriority::High),
        other => Err(db_error(format!("bad priority {other}"))),
    }
}

fn reminder_mode_str(p: &ReminderPreference) -> &'static str {
    match p {
        ReminderPreference::InheritDefault => "INHERIT_DEFAULT",
        ReminderPreference::Custom { .. } => "CUSTOM",
        ReminderPreference::Disabled => "DISABLED",
    }
}

fn parse_reminder(mode: &str, minutes: Option<i64>) -> Result<ReminderPreference, RepoError> {
    match mode {
        "INHERIT_DEFAULT" => Ok(ReminderPreference::InheritDefault),
        "CUSTOM" => {
            let m = minutes.ok_or_else(|| db_error("CUSTOM requires minutes"))?;
            Ok(ReminderPreference::Custom {
                minutes_before: m as u32,
            })
        }
        "DISABLED" => Ok(ReminderPreference::Disabled),
        other => Err(db_error(format!("bad reminder mode {other}"))),
    }
}

fn row_to_task(row: &Row) -> rusqlite::Result<Task> {
    let id: String = row.get(0)?;
    let title: String = row.get(1)?;
    let status = parse_status(&row.get::<_, String>(2)?).map_err(to_sqlite)?;
    let original_date = parse_date(&row.get::<_, String>(3)?).map_err(to_sqlite)?;
    let original_time: Option<String> = row.get(4)?;
    let original_all_day: i64 = row.get(5)?;
    let scheduled_date = parse_date(&row.get::<_, String>(6)?).map_err(to_sqlite)?;
    let scheduled_time: Option<String> = row.get(7)?;
    let is_all_day: i64 = row.get(8)?;
    let reminder_mode: String = row.get(9)?;
    let reminder_minutes: Option<i64> = row.get(10)?;
    let priority = parse_priority(&row.get::<_, String>(11)?).map_err(to_sqlite)?;
    let category_id: Option<String> = row.get(12)?;
    let created_at = parse_instant(&row.get::<_, String>(13)?).map_err(to_sqlite)?;
    let updated_at = parse_instant(&row.get::<_, String>(14)?).map_err(to_sqlite)?;
    let completed_at: Option<String> = row.get(15)?;
    let deleted_at: Option<String> = row.get(16)?;
    let version: i64 = row.get(17)?;

    let original_schedule = TaskSchedule {
        date: original_date,
        time: original_time
            .as_deref()
            .map(parse_time)
            .transpose()
            .map_err(to_sqlite)?,
        all_day: original_all_day != 0,
    };
    let current_schedule = TaskSchedule {
        date: scheduled_date,
        time: scheduled_time
            .as_deref()
            .map(parse_time)
            .transpose()
            .map_err(to_sqlite)?,
        all_day: is_all_day != 0,
    };

    Ok(Task {
        id: Uuid::parse_str(&id).map_err(to_sqlite)?,
        title,
        status,
        original_schedule,
        current_schedule,
        priority,
        category_id: category_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(to_sqlite)?,
        reminder_preference: parse_reminder(&reminder_mode, reminder_minutes).map_err(to_sqlite)?,
        created_at,
        updated_at,
        completed_at: completed_at
            .as_deref()
            .map(parse_instant)
            .transpose()
            .map_err(to_sqlite)?,
        deleted_at: deleted_at
            .as_deref()
            .map(parse_instant)
            .transpose()
            .map_err(to_sqlite)?,
        version,
    })
}

fn to_sqlite(e: impl std::fmt::Display) -> rusqlite::Error {
    rusqlite::Error::InvalidPath(e.to_string().into())
}

fn db_error(msg: impl std::fmt::Display) -> RepoError {
    RepoError::Db(rusqlite::Error::InvalidPath(msg.to_string().into()))
}

const TASK_COLUMNS: &str = "id, title, status, original_date, original_time, original_is_all_day, \
     scheduled_date, scheduled_time, is_all_day, reminder_mode, reminder_minutes, priority, \
     category_id, created_at, updated_at, completed_at, deleted_at, version";

/// SQLite-backed task repository.
pub struct SqliteTaskRepository {
    conn: SqliteConnection,
}

impl SqliteTaskRepository {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }
}

impl TaskRepository for SqliteTaskRepository {
    fn insert(&self, task: &Task) -> Result<(), RepoError> {
        self.conn.lock().execute(
            "INSERT INTO tasks (id, title, status, original_date, original_time, original_is_all_day,
                scheduled_date, scheduled_time, is_all_day, reminder_mode, reminder_minutes,
                priority, category_id, created_at, updated_at, completed_at, deleted_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                task.id.to_string(),
                task.title,
                status_str(task.status),
                date_str(task.original_schedule.date),
                task.original_schedule.time.map(time_str),
                task.original_schedule.all_day as i64,
                date_str(task.current_schedule.date),
                task.current_schedule.time.map(time_str),
                task.current_schedule.all_day as i64,
                reminder_mode_str(&task.reminder_preference),
                match task.reminder_preference {
                    ReminderPreference::Custom { minutes_before } => Some(minutes_before as i64),
                    _ => None,
                },
                priority_str(task.priority),
                task.category_id.map(|c| c.to_string()),
                instant_str(task.created_at),
                instant_str(task.updated_at),
                task.completed_at.map(instant_str),
                task.deleted_at.map(instant_str),
                task.version,
            ],
        )?;
        Ok(())
    }

    fn get(&self, id: Uuid) -> Result<Option<Task>, RepoError> {
        let sql = format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?1");
        let task = self
            .conn
            .lock()
            .query_row(sql.as_str(), params![id.to_string()], row_to_task)
            .optional()?;
        Ok(task)
    }

    fn list(&self, query: &TaskQuery) -> Result<Vec<Task>, RepoError> {
        let mut sql = format!("SELECT {TASK_COLUMNS} FROM tasks WHERE 1=1");
        let mut param_values: Vec<String> = Vec::new();

        if !query.statuses.is_empty() {
            let placeholders: Vec<String> = (1..=query.statuses.len())
                .map(|i| format!("?{}", param_values.len() + i))
                .collect();
            sql.push_str(&format!(" AND status IN ({})", placeholders.join(", ")));
            for s in &query.statuses {
                param_values.push(status_str(*s).to_string());
            }
        }
        if !query.include_deleted {
            sql.push_str(" AND deleted_at IS NULL");
        }
        if let Some(d) = query.date_before {
            param_values.push(date_str(d));
            sql.push_str(&format!(" AND scheduled_date < ?{}", param_values.len()));
        }
        if let Some(d) = query.date_after {
            param_values.push(date_str(d));
            sql.push_str(&format!(" AND scheduled_date > ?{}", param_values.len()));
        }
        if let Some(d) = query.date_eq {
            param_values.push(date_str(d));
            sql.push_str(&format!(" AND scheduled_date = ?{}", param_values.len()));
        }
        sql.push_str(" ORDER BY scheduled_date, scheduled_time, created_at, id");

        let conn = self.conn.lock();
        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let tasks = stmt
            .query_map(params_ref.as_slice(), row_to_task)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(tasks)
    }

    fn update(&self, task: &Task, expected_version: i64) -> Result<Task, RepoError> {
        let updated = self.conn.lock().execute(
            "UPDATE tasks SET
                title = ?2, status = ?3,
                original_date = ?4, original_time = ?5, original_is_all_day = ?6,
                scheduled_date = ?7, scheduled_time = ?8, is_all_day = ?9,
                reminder_mode = ?10, reminder_minutes = ?11,
                priority = ?12, category_id = ?13,
                created_at = ?14, updated_at = ?15, completed_at = ?16, deleted_at = ?17,
                version = ?18
             WHERE id = ?1 AND version = ?19 AND deleted_at IS NULL",
            params![
                task.id.to_string(),
                task.title,
                status_str(task.status),
                date_str(task.original_schedule.date),
                task.original_schedule.time.map(time_str),
                task.original_schedule.all_day as i64,
                date_str(task.current_schedule.date),
                task.current_schedule.time.map(time_str),
                task.current_schedule.all_day as i64,
                reminder_mode_str(&task.reminder_preference),
                match task.reminder_preference {
                    ReminderPreference::Custom { minutes_before } => Some(minutes_before as i64),
                    _ => None,
                },
                priority_str(task.priority),
                task.category_id.map(|c| c.to_string()),
                instant_str(task.created_at),
                instant_str(task.updated_at),
                task.completed_at.map(instant_str),
                task.deleted_at.map(instant_str),
                task.version,
                expected_version,
            ],
        )?;

        if updated == 0 {
            // Distinguish not-found/deleted from a genuine version conflict.
            let existing = self.get(task.id)?;
            match existing {
                None => return Err(RepoError::TaskNotFound(task.id)),
                Some(row) if row.deleted_at.is_some() => {
                    return Err(RepoError::TaskDeleted(task.id))
                }
                Some(_) => {
                    return Err(RepoError::VersionConflict {
                        id: task.id,
                        expected: expected_version,
                    })
                }
            }
        }

        self.get(task.id)?
            .ok_or_else(|| RepoError::TaskNotFound(task.id))
    }

    fn count(&self) -> Result<i64, RepoError> {
        let n: i64 = self
            .conn
            .lock()
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))?;
        Ok(n)
    }
}

/// SQLite-backed category repository.
pub struct SqliteCategoryRepository {
    conn: SqliteConnection,
}

impl SqliteCategoryRepository {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }
}

impl CategoryRepository for SqliteCategoryRepository {
    fn insert(
        &self,
        id: Uuid,
        name: &str,
        sort_order: i64,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepoError> {
        self.conn.lock().execute(
            "INSERT INTO categories (id, name, sort_order, created_at, updated_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![
                id.to_string(),
                name,
                sort_order,
                instant_str(now),
                instant_str(now)
            ],
        )?;
        Ok(())
    }

    fn list_active(&self) -> Result<Vec<Category>, RepoError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, sort_order FROM categories WHERE deleted_at IS NULL ORDER BY sort_order, name")?;
        let cats = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(to_sqlite)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(cats)
    }
}

/// SQLite-backed settings repository (typed accessors over key/value).
pub struct SqliteSettingsRepository {
    conn: SqliteConnection,
}

const KEY_REMINDER_DEFAULT: &str = "reminder.default";

impl SqliteSettingsRepository {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn get_global_reminder_default(&self) -> Result<GlobalReminderDefault, RepoError> {
        let value: Option<String> = self
            .conn
            .lock()
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![KEY_REMINDER_DEFAULT],
                |r| r.get(0),
            )
            .optional()?;
        match value {
            Some(v) => {
                serde_json::from_str(&v).map_err(|e| db_error(format!("bad reminder default: {e}")))
            }
            None => Ok(GlobalReminderDefault::default()),
        }
    }

    fn set_global_reminder_default(&self, value: &GlobalReminderDefault) -> Result<(), RepoError> {
        let json = serde_json::to_string(value).map_err(db_error)?;
        self.conn.lock().execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
              ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![KEY_REMINDER_DEFAULT, json, instant_str(chrono::Utc::now())],
        )?;
        Ok(())
    }

    fn get_string(&self, key: &str) -> Result<Option<String>, RepoError> {
        let v: Option<String> = self
            .conn
            .lock()
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?;
        Ok(v)
    }

    fn set_string(&self, key: &str, value: &str) -> Result<(), RepoError> {
        self.conn.lock().execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
              ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, instant_str(chrono::Utc::now())],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{normalize_title, TaskSchedule};
    use crate::repo::sqlite::migrate::migrate;
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};

    fn open() -> SqliteConnection {
        let conn = SqliteConnection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn sample_task(title: &str) -> Task {
        let d = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
        Task::new(
            Uuid::new_v4(),
            normalize_title(title).unwrap(),
            TaskSchedule::timed(d, NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            TaskPriority::Normal,
            None,
            ReminderPreference::InheritDefault,
            Utc.with_ymd_and_hms(2026, 10, 26, 12, 0, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn insert_then_get_round_trips() {
        let conn = open();
        let repo = SqliteTaskRepository::new(conn);
        let t = sample_task("写周报");
        let id = t.id;
        repo.insert(&t).unwrap();
        let loaded = repo.get(id).unwrap().unwrap();
        assert_eq!(loaded, t);
    }

    #[test]
    fn list_filters_by_status_and_date() {
        let conn = open();
        let repo = SqliteTaskRepository::new(conn);
        let today = sample_task("today");
        let mut past = sample_task("past");
        past.current_schedule.date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        past.original_schedule.date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        repo.insert(&today).unwrap();
        repo.insert(&past).unwrap();

        let q = TaskQuery {
            statuses: vec![TaskStatus::Todo],
            date_eq: Some(NaiveDate::from_ymd_opt(2026, 10, 26).unwrap()),
            ..Default::default()
        };
        let list = repo.list(&q).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "today");
    }

    #[test]
    fn update_with_correct_version_succeeds() {
        let conn = open();
        let repo = SqliteTaskRepository::new(conn);
        let mut t = sample_task("v1");
        repo.insert(&t).unwrap();
        let expected = t.version;
        t.title = "v2".into();
        t.version += 1;
        let updated = repo.update(&t, expected).unwrap();
        assert_eq!(updated.title, "v2");
        assert_eq!(updated.version, 2);
    }

    #[test]
    fn update_with_stale_version_conflicts() {
        let conn = open();
        let repo = SqliteTaskRepository::new(conn);
        let mut t = sample_task("v1");
        repo.insert(&t).unwrap();
        // Simulate a newer write.
        t.title = "newer".into();
        t.version += 1;
        repo.update(&t, 1).unwrap();
        // Now a stale writer with expected_version = 1 must conflict.
        let mut stale = sample_task("stale");
        stale.id = t.id;
        stale.version = 2;
        let err = repo.update(&stale, 1).unwrap_err();
        assert!(matches!(err, RepoError::VersionConflict { .. }));
    }

    #[test]
    fn update_missing_task_is_not_found() {
        let conn = open();
        let repo = SqliteTaskRepository::new(conn);
        let mut t = sample_task("ghost");
        t.version = 2;
        let err = repo.update(&t, 1).unwrap_err();
        assert!(matches!(err, RepoError::TaskNotFound(_)));
    }

    #[test]
    fn soft_deleted_task_is_excluded_and_update_rejected() {
        let conn = open();
        let repo = SqliteTaskRepository::new(conn);
        let mut t = sample_task("doomed");
        repo.insert(&t).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 13, 0, 0).unwrap();
        t.soft_delete(now).unwrap();
        repo.update(&t, 1).unwrap();

        // Excluded from default list.
        let q = TaskQuery::default();
        assert!(repo.list(&q).unwrap().is_empty());
        // Still present when explicitly included (tombstone preserved).
        let q = TaskQuery {
            include_deleted: true,
            ..Default::default()
        };
        assert_eq!(repo.list(&q).unwrap().len(), 1);
        // Further updates rejected as deleted.
        let mut again = repo.get(t.id).unwrap().unwrap();
        again.title = "zombie".into();
        again.version += 1;
        let err = repo.update(&again, 2).unwrap_err();
        assert!(matches!(err, RepoError::TaskDeleted(_)));
    }

    #[test]
    fn settings_round_trip_and_default() {
        let conn = open();
        let repo = SqliteSettingsRepository::new(conn);
        // Default before any write.
        let d = repo.get_global_reminder_default().unwrap();
        assert!(d.enabled);
        // Write a custom value.
        let v = GlobalReminderDefault {
            enabled: false,
            minutes_before: 0,
        };
        repo.set_global_reminder_default(&v).unwrap();
        assert_eq!(repo.get_global_reminder_default().unwrap(), v);
    }

    #[test]
    fn persistence_survives_reopen() {
        let path =
            std::env::temp_dir().join(format!("dayflow-persistence-{}.sqlite", Uuid::new_v4()));

        let task_id;
        let reminder_default = GlobalReminderDefault {
            enabled: false,
            minutes_before: 0,
        };
        {
            let conn = SqliteConnection::open(&path).unwrap();
            migrate(&conn).unwrap();
            let tasks = SqliteTaskRepository::new(conn.clone());
            let settings = SqliteSettingsRepository::new(conn);

            let task = sample_task("重启后仍在");
            task_id = task.id;
            tasks.insert(&task).unwrap();
            settings
                .set_global_reminder_default(&reminder_default)
                .unwrap();
        }

        let conn = SqliteConnection::open(&path).unwrap();
        migrate(&conn).unwrap();
        let tasks = SqliteTaskRepository::new(conn.clone());
        let settings = SqliteSettingsRepository::new(conn);

        let loaded = tasks.get(task_id).unwrap().unwrap();
        assert_eq!(loaded.title, "重启后仍在");
        assert_eq!(
            settings.get_global_reminder_default().unwrap(),
            reminder_default
        );

        let _ = std::fs::remove_file(&path);
    }
}
