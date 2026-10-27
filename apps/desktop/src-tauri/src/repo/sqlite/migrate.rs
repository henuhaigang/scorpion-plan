use rusqlite::Result;

use super::SqliteConnection;

/// A single forward-only migration.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "0001_initial",
    sql: r#"
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(title) > 0),
    status TEXT NOT NULL CHECK (status IN ('TODO', 'COMPLETED', 'CANCELED')),
    original_date TEXT NOT NULL,
    original_time TEXT,
    original_is_all_day INTEGER NOT NULL,
    scheduled_date TEXT NOT NULL,
    scheduled_time TEXT,
    is_all_day INTEGER NOT NULL,
    reminder_mode TEXT NOT NULL CHECK (reminder_mode IN ('INHERIT_DEFAULT', 'CUSTOM', 'DISABLED')),
    reminder_minutes INTEGER,
    priority TEXT NOT NULL CHECK (priority IN ('LOW', 'NORMAL', 'HIGH')),
    category_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    deleted_at TEXT,
    version INTEGER NOT NULL CHECK (version >= 1),
    CHECK (is_all_day = 0 OR scheduled_time IS NULL),
    CHECK (original_is_all_day = 0 OR original_time IS NULL),
    CHECK ((reminder_mode = 'CUSTOM') = (reminder_minutes IS NOT NULL)),
    CHECK (reminder_minutes IS NULL OR reminder_minutes >= 0)
);

CREATE INDEX idx_tasks_status_sched ON tasks (status, scheduled_date, scheduled_time);
CREATE INDEX idx_tasks_updated ON tasks (updated_at);
CREATE INDEX idx_tasks_category ON tasks (category_id, status);

CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    version INTEGER NOT NULL CHECK (version >= 1)
);

CREATE TABLE reminder_jobs (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    fire_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'SCHEDULED', 'FIRED', 'CANCELED', 'FAILED')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT
);

CREATE INDEX idx_jobs_status_fire ON reminder_jobs (status, fire_at);
CREATE INDEX idx_jobs_task ON reminder_jobs (task_id, status);

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#,
}];

/// Apply all pending migrations in order. Idempotent: already-applied
/// versions (per the `schema_migrations` ledger) are skipped.
pub fn migrate(conn: &SqliteConnection) -> Result<i64> {
    conn.lock().execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current: i64 = conn.lock().query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    let mut applied = current;
    for m in MIGRATIONS {
        if m.version <= current {
            continue;
        }
        {
            let mut guard = conn.lock();
            let tx = guard.transaction()?;
            tx.execute_batch(m.sql)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                (m.version, m.name, chrono::Utc::now().to_rfc3339()),
            )?;
            tx.commit()?;
        }
        applied = m.version;
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_is_idempotent() {
        let conn = SqliteConnection::open_in_memory().unwrap();
        assert_eq!(migrate(&conn).unwrap(), 1);
        assert_eq!(migrate(&conn).unwrap(), 1);
        let count: i64 = conn
            .lock()
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn schema_enforces_reminder_invariants() {
        let conn = SqliteConnection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // CUSTOM without minutes must be rejected by the CHECK constraint.
        let err = conn.lock().execute(
            "INSERT INTO tasks (id, title, status, original_date, original_is_all_day,
                scheduled_date, is_all_day, reminder_mode, reminder_minutes, priority,
                created_at, updated_at, version)
             VALUES ('x', 't', 'TODO', '2026-10-26', 0, '2026-10-26', 0, 'CUSTOM', NULL,
                     'NORMAL', '2026-10-26T00:00:00Z', '2026-10-26T00:00:00Z', 1)",
            [],
        );
        assert!(err.is_err(), "CUSTOM with NULL minutes must violate CHECK");
        // INHERIT_DEFAULT with minutes must be rejected.
        let err = conn.lock().execute(
            "INSERT INTO tasks (id, title, status, original_date, original_is_all_day,
                scheduled_date, is_all_day, reminder_mode, reminder_minutes, priority,
                created_at, updated_at, version)
             VALUES ('y', 't', 'TODO', '2026-10-26', 0, '2026-10-26', 0, 'INHERIT_DEFAULT', 5,
                     'NORMAL', '2026-10-26T00:00:00Z', '2026-10-26T00:00:00Z', 1)",
            [],
        );
        assert!(
            err.is_err(),
            "INHERIT_DEFAULT with minutes must violate CHECK"
        );
    }

    #[test]
    fn schema_enforces_all_day_invariant() {
        let conn = SqliteConnection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let err = conn.lock().execute(
            "INSERT INTO tasks (id, title, status, original_date, original_is_all_day,
                scheduled_date, scheduled_time, is_all_day, reminder_mode, priority,
                created_at, updated_at, version)
             VALUES ('z', 't', 'TODO', '2026-10-26', 0, '2026-10-26', '09:00:00', 1,
                     'INHERIT_DEFAULT', 'NORMAL', '2026-10-26T00:00:00Z', '2026-10-26T00:00:00Z', 1)",
            [],
        );
        assert!(err.is_err(), "all_day with a time must violate CHECK");
    }
}
