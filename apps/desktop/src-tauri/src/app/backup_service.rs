use std::path::{Path, PathBuf};

use crate::repo::sqlite::SqliteConnection;
use crate::repo::RepoError;

/// Backup service: exports the SQLite database to a file.
pub struct BackupService {
    conn: SqliteConnection,
}

impl BackupService {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }

    /// Export the database to a backup file. Returns the path to the backup.
    pub fn export(&self, dest: &PathBuf) -> Result<PathBuf, RepoError> {
        let conn = self.conn.lock();
        conn.backup(
            "main",
            dest,
            None::<Box<dyn Fn(rusqlite::backup::Progress)>>,
        )?;
        Ok(dest.clone())
    }

    /// Get the default backup directory (app data dir / backups).
    pub fn default_backup_dir(app_data_dir: &Path) -> PathBuf {
        app_data_dir.join("backups")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::sqlite::migrate;
    use std::fs;

    #[test]
    fn export_creates_valid_backup() {
        let src = SqliteConnection::open_in_memory().unwrap();
        migrate(&src).unwrap();

        // Insert a test task.
        src.lock()
            .execute(
                "INSERT INTO tasks (id, title, status, original_date, original_is_all_day,
                    scheduled_date, is_all_day, reminder_mode, priority, created_at, updated_at, version)
                 VALUES ('test-id', 'backup test', 'TODO', '2026-10-26', 0, '2026-10-26', 0,
                         'INHERIT_DEFAULT', 'NORMAL', '2026-10-26T00:00:00Z', '2026-10-26T00:00:00Z', 1)",
                [],
            )
            .unwrap();

        let service = BackupService::new(src);
        let dest = std::env::temp_dir().join("dayflow_test_backup.sqlite");
        let _ = fs::remove_file(&dest);

        let result = service.export(&dest).unwrap();
        assert!(result.exists());

        // Verify the backup contains the task.
        let verify_conn = rusqlite::Connection::open(&dest).unwrap();
        let count: i32 = verify_conn
            .query_row("SELECT COUNT(*) FROM tasks WHERE id = 'test-id'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_file(&dest);
    }

    #[test]
    fn default_backup_dir_is_correct() {
        let app_dir = PathBuf::from("/tmp/dayflow");
        let backup_dir = BackupService::default_backup_dir(&app_dir);
        assert_eq!(backup_dir, PathBuf::from("/tmp/dayflow/backups"));
    }
}
