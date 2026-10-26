# Data, SQLite and Future Sync

## 1. Local-first requirements
- SQLite is the V1 source of truth for task data.
- Core CRUD must work offline.
- Use versioned migrations. Never rely on creating tables only at app startup without a migration ledger.
- Enable foreign keys per connection where supported.
- Use transactions for each logical task mutation and its database-side state changes.
- OS notification scheduling is not part of a SQLite transaction. Persist task changes first; then reconcile reminder jobs. Failures must be retriable.
- Avoid showing an empty list when a query fails; expose an error state.

## 2. Suggested tables
### tasks
`id TEXT PRIMARY KEY`, `title TEXT NOT NULL`, `status TEXT NOT NULL`, `original_date TEXT NOT NULL`, `original_time TEXT NULL`, `original_is_all_day INTEGER NOT NULL`, `scheduled_date TEXT NOT NULL`, `scheduled_time TEXT NULL`, `is_all_day INTEGER NOT NULL`, `reminder_mode TEXT NOT NULL`, `reminder_minutes INTEGER NULL`, `priority TEXT NOT NULL`, `category_id TEXT NULL`, `created_at TEXT NOT NULL`, `updated_at TEXT NOT NULL`, `completed_at TEXT NULL`, `deleted_at TEXT NULL`, `version INTEGER NOT NULL`.

Use ISO-8601 date/time encodings and UTC ISO-8601 instants. Validate encodings in application code and add SQL CHECK constraints where practical. All-day implies time is null. CUSTOM reminder requires nonnegative minutes; other modes require null minutes. Status is limited to TODO/COMPLETED/CANCELED.

### categories
`id`, `name`, `sort_order`, `created_at`, `updated_at`, `deleted_at`, `version`. Decide category deletion semantics before implementation; do not cascade-delete tasks by default.

### reminder_jobs
`id`, `task_id`, `fire_at` (UTC instant), `status`, `created_at`, `updated_at`, optional attempt/error fields. Suggested statuses: PENDING, SCHEDULED, FIRED, CANCELED, FAILED. Actual OS scheduling state can differ from DB state; reconcile on startup and after mutations.

### app_settings
A typed settings repository is required. It can use key/value storage initially. Store the default reminder preference, effective timezone policy, and UI settings. Do not let domain code read arbitrary raw strings.

### optional task_events
Not required for V1. Can later record reschedule/completion/cancellation events. Do not pretend `updated_at` is a complete audit log.

## 3. Index candidates
- `(status, scheduled_date, scheduled_time)`
- `(updated_at)`
- `(category_id, status)`
- `(status, fire_at)` on reminder_jobs
- `(task_id, status)` on reminder_jobs

Validate indexes using actual query plans and real queries; do not add indexes without reason.

## 4. Concurrency and versions
Every successful task mutation increments version. Updates must use optimistic locking, e.g. `UPDATE tasks SET ..., version = version + 1 WHERE id = ? AND version = ? AND deleted_at IS NULL`. If affected rows = 0, distinguish not found/deleted from version conflict. Do not use last-write-wins silently.

## 5. Deletion/tombstones
Cancellation is a business status. Deletion is `deleted_at != null` and acts as a tombstone. Keep tombstones until sync protocol defines safe compaction; do not physically purge them in V1.

## 6. Future sync principles (not V1 implementation)
- Local-first; sync is asynchronous and must not block local task operations.
- Stable UUIDs across devices.
- Use a server cursor/change sequence rather than relying only on client timestamps for robust incremental sync.
- Server must authorize user ownership and validate versions.
- Define conflict response and user resolution policy before enabling multi-device writes.
- A tombstone must propagate to all devices.
- Reminder scheduling ownership/deduplication across devices must be decided separately; avoid each device sending duplicate notifications.
- Do not add cloud account or sync code to V1 beyond clean ports and documented contracts.
