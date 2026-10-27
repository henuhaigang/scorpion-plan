use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use super::SqliteConnection;
use crate::domain::reminder_job::{ReminderJob, ReminderJobStatus};
use crate::repo::reminder_job_repo::ReminderJobRepository;
use crate::repo::RepoError;

pub struct SqliteReminderJobRepository {
    conn: SqliteConnection,
}

impl SqliteReminderJobRepository {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }
}

fn conv_err(msg: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, msg.into())
}

fn row_to_job(row: &Row) -> rusqlite::Result<ReminderJob> {
    let id_str: String = row.get(0)?;
    let task_id_str: String = row.get(1)?;
    let fire_at_str: String = row.get(2)?;
    let status_str: String = row.get(3)?;
    let created_str: String = row.get(4)?;
    let updated_str: String = row.get(5)?;
    let attempts: i32 = row.get(6)?;
    let last_error: Option<String> = row.get(7)?;

    Ok(ReminderJob {
        id: Uuid::parse_str(&id_str).map_err(|e| conv_err(e.to_string()))?,
        task_id: Uuid::parse_str(&task_id_str).map_err(|e| conv_err(e.to_string()))?,
        fire_at: fire_at_str
            .parse()
            .map_err(|e: chrono::ParseError| conv_err(e.to_string()))?,
        status: match status_str.as_str() {
            "PENDING" => ReminderJobStatus::Pending,
            "SCHEDULED" => ReminderJobStatus::Scheduled,
            "FIRED" => ReminderJobStatus::Fired,
            "CANCELED" => ReminderJobStatus::Canceled,
            "FAILED" => ReminderJobStatus::Failed,
            other => return Err(conv_err(format!("unknown status: {other}"))),
        },
        created_at: created_str
            .parse()
            .map_err(|e: chrono::ParseError| conv_err(e.to_string()))?,
        updated_at: updated_str
            .parse()
            .map_err(|e: chrono::ParseError| conv_err(e.to_string()))?,
        attempts,
        last_error,
    })
}

fn status_str(s: &ReminderJobStatus) -> &'static str {
    match s {
        ReminderJobStatus::Pending => "PENDING",
        ReminderJobStatus::Scheduled => "SCHEDULED",
        ReminderJobStatus::Fired => "FIRED",
        ReminderJobStatus::Canceled => "CANCELED",
        ReminderJobStatus::Failed => "FAILED",
    }
}

impl ReminderJobRepository for SqliteReminderJobRepository {
    fn insert(&self, job: &ReminderJob) -> Result<(), RepoError> {
        self.conn.lock().execute(
            "INSERT INTO reminder_jobs (id, task_id, fire_at, status, created_at, updated_at, attempts, last_error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                job.id.to_string(),
                job.task_id.to_string(),
                job.fire_at.to_rfc3339(),
                status_str(&job.status),
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
                job.attempts,
                job.last_error,
            ],
        )?;
        Ok(())
    }

    fn get(&self, id: Uuid) -> Result<Option<ReminderJob>, RepoError> {
        let job = self
            .conn
            .lock()
            .query_row(
                "SELECT id, task_id, fire_at, status, created_at, updated_at, attempts, last_error
                 FROM reminder_jobs WHERE id = ?1",
                params![id.to_string()],
                row_to_job,
            )
            .optional()?;
        Ok(job)
    }

    fn list_by_task(
        &self,
        task_id: Uuid,
        statuses: &[ReminderJobStatus],
    ) -> Result<Vec<ReminderJob>, RepoError> {
        if statuses.is_empty() {
            return Ok(Vec::new());
        }
        // Build a dynamic IN clause.
        let placeholders: Vec<String> = (2..=statuses.len() + 1).map(|i| format!("?{i}")).collect();
        let in_clause = placeholders.join(", ");
        let sql = format!(
            "SELECT id, task_id, fire_at, status, created_at, updated_at, attempts, last_error
             FROM reminder_jobs WHERE task_id = ?1 AND status IN ({in_clause})
             ORDER BY fire_at ASC"
        );
        let mut param_list: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        param_list.push(Box::new(task_id.to_string()));
        for s in statuses {
            param_list.push(Box::new(status_str(s).to_string()));
        }
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            param_list.iter().map(|p| p.as_ref()).collect();
        let guard = self.conn.lock();
        let mut stmt = guard.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), row_to_job)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    fn list_due(&self, before: DateTime<Utc>, limit: i64) -> Result<Vec<ReminderJob>, RepoError> {
        let guard = self.conn.lock();
        let mut stmt = guard.prepare(
            "SELECT id, task_id, fire_at, status, created_at, updated_at, attempts, last_error
             FROM reminder_jobs
             WHERE status IN ('PENDING', 'SCHEDULED') AND fire_at <= ?1
             ORDER BY fire_at ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![before.to_rfc3339(), limit], row_to_job)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    fn update_status(
        &self,
        id: Uuid,
        status: ReminderJobStatus,
        last_error: Option<&str>,
    ) -> Result<(), RepoError> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE reminder_jobs SET status = ?2, updated_at = ?3, last_error = ?4 WHERE id = ?1",
            params![id.to_string(), status_str(&status), now, last_error],
        )?;
        Ok(())
    }

    fn cancel_active_by_task(&self, task_id: Uuid) -> Result<usize, RepoError> {
        let now = Utc::now().to_rfc3339();
        let count = self.conn.lock().execute(
            "UPDATE reminder_jobs SET status = 'CANCELED', updated_at = ?2
             WHERE task_id = ?1 AND status IN ('PENDING', 'SCHEDULED')",
            params![task_id.to_string(), now],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::sqlite::migrate;

    fn setup() -> (SqliteConnection, SqliteReminderJobRepository) {
        let conn = SqliteConnection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let repo = SqliteReminderJobRepository::new(conn.clone());
        (conn, repo)
    }

    fn make_task_row(conn: &SqliteConnection, id: &str) {
        conn.lock()
            .execute(
                "INSERT INTO tasks (id, title, status, original_date, original_is_all_day,
                    scheduled_date, is_all_day, reminder_mode, priority, created_at, updated_at, version)
                 VALUES (?1, 't', 'TODO', '2026-10-26', 0, '2026-10-26', 0, 'INHERIT_DEFAULT',
                         'NORMAL', '2026-10-26T00:00:00Z', '2026-10-26T00:00:00Z', 1)",
                params![id],
            )
            .unwrap();
    }

    #[test]
    fn insert_and_get_round_trips() {
        let (_conn, repo) = setup();
        let task_id = Uuid::new_v4();
        make_task_row(&repo.conn, &task_id.to_string());

        let now = Utc::now();
        let job = ReminderJob::new(task_id, now + chrono::Duration::minutes(10), now);
        repo.insert(&job).unwrap();

        let loaded = repo.get(job.id).unwrap().unwrap();
        assert_eq!(loaded.id, job.id);
        assert_eq!(loaded.task_id, task_id);
        assert_eq!(loaded.status, ReminderJobStatus::Pending);
        assert_eq!(loaded.fire_at, job.fire_at);
    }

    #[test]
    fn list_by_task_filters_by_status() {
        let (_conn, repo) = setup();
        let task_id = Uuid::new_v4();
        make_task_row(&repo.conn, &task_id.to_string());

        let now = Utc::now();
        let job1 = ReminderJob::new(task_id, now + chrono::Duration::minutes(5), now);
        let job2 = ReminderJob::new(task_id, now + chrono::Duration::minutes(10), now);
        repo.insert(&job1).unwrap();
        repo.insert(&job2).unwrap();

        repo.update_status(job1.id, ReminderJobStatus::Fired, None)
            .unwrap();

        let active = repo
            .list_by_task(
                task_id,
                &[ReminderJobStatus::Pending, ReminderJobStatus::Scheduled],
            )
            .unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, job2.id);
    }

    #[test]
    fn list_due_returns_only_active_jobs_before_cutoff() {
        let (_conn, repo) = setup();
        let task_id = Uuid::new_v4();
        make_task_row(&repo.conn, &task_id.to_string());

        let now = Utc::now();
        let due = ReminderJob::new(task_id, now - chrono::Duration::minutes(1), now);
        let future = ReminderJob::new(task_id, now + chrono::Duration::minutes(60), now);
        repo.insert(&due).unwrap();
        repo.insert(&future).unwrap();

        let result = repo.list_due(now, 10).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, due.id);
    }

    #[test]
    fn update_status_transitions_work() {
        let (_conn, repo) = setup();
        let task_id = Uuid::new_v4();
        make_task_row(&repo.conn, &task_id.to_string());

        let now = Utc::now();
        let job = ReminderJob::new(task_id, now + chrono::Duration::minutes(10), now);
        repo.insert(&job).unwrap();

        repo.update_status(job.id, ReminderJobStatus::Scheduled, None)
            .unwrap();
        let loaded = repo.get(job.id).unwrap().unwrap();
        assert_eq!(loaded.status, ReminderJobStatus::Scheduled);

        repo.update_status(job.id, ReminderJobStatus::Failed, Some("timeout"))
            .unwrap();
        let loaded = repo.get(job.id).unwrap().unwrap();
        assert_eq!(loaded.status, ReminderJobStatus::Failed);
        assert_eq!(loaded.last_error.as_deref(), Some("timeout"));
    }

    #[test]
    fn cancel_active_by_task_cancels_only_active() {
        let (_conn, repo) = setup();
        let task_id = Uuid::new_v4();
        make_task_row(&repo.conn, &task_id.to_string());

        let now = Utc::now();
        let job1 = ReminderJob::new(task_id, now + chrono::Duration::minutes(5), now);
        let job2 = ReminderJob::new(task_id, now + chrono::Duration::minutes(10), now);
        repo.insert(&job1).unwrap();
        repo.insert(&job2).unwrap();

        repo.update_status(job1.id, ReminderJobStatus::Fired, None)
            .unwrap();

        let count = repo.cancel_active_by_task(task_id).unwrap();
        assert_eq!(count, 1);

        let loaded1 = repo.get(job1.id).unwrap().unwrap();
        assert_eq!(loaded1.status, ReminderJobStatus::Fired);
        let loaded2 = repo.get(job2.id).unwrap().unwrap();
        assert_eq!(loaded2.status, ReminderJobStatus::Canceled);
    }
}
