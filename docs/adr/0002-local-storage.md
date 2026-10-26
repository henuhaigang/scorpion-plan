# ADR 0002: Local Storage

- Status: Accepted
- Decision: SQLite with versioned migrations
- Rationale: local-first, offline-capable task CRUD and durable data.
- Requirements: migrations, parameterized SQL, transactions, optimistic locking, tombstones, backup/export plan.
- Binding: resolved by ADR 0004 (`rusqlite` 0.41, `bundled` feature, embedded forward-only migrations with a `schema_migrations` ledger).
