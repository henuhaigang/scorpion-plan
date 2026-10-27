# ADR 0004: SQLite Binding and Migrations

- Status: Accepted
- Date: 2026-10-26
- Deciders: DayFlow engineering (agent-assisted)
- Resolves: open decision in ADR 0002 (local storage)

## Context

ADR 0002 mandates SQLite with versioned migrations but leaves the binding open.
Candidates considered for the Tauri 2 (Rust) desktop core:

1. `rusqlite` (synchronous, thin wrapper over `libsqlite3-sys`)
2. `sqlx` (async, feature-rich, heavier dependency tree)
3. Node-side `better-sqlite3` (would move persistence into the webview process)

Requirements: parameterized SQL, transactions, optimistic locking, per-connection
foreign keys, deterministic in-process tests, no system SQLite version drift,
and a small, maintained dependency surface.

## Decision

Use `rusqlite` 0.41 with the `bundled` feature (compiles SQLite from source,
pinned by `Cargo.lock`). Migrations are plain SQL statements embedded at compile
time (M1 uses inline `&'static str` constants; `include_str!` files remain
acceptable) and applied by a small migration runner that maintains a
`schema_migrations(version, name, applied_at)` ledger table. Migrations are
forward-only and idempotent per ledger entry.

## Consequences

- Synchronous API fits Tauri command handlers and keeps tests deterministic.
- `bundled` removes dependence on the OS-provided SQLite (macOS ships an old one).
- Migration runner is ~100 lines of auditable code instead of a framework.
- `sqlx` remains a viable swap later if async queries become necessary; the
  repository port isolates the choice.

## Reversal criteria

If a required workload (e.g., large batch sync imports) shows measurable
contention on the synchronous API, re-evaluate `sqlx` with an ADR.
