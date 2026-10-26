# ADR 0005: Domain Layer Placement

- Status: Accepted
- Date: 2026-10-26
- Deciders: DayFlow engineering (agent-assisted)

## Context

`ARCHITECTURE.md` requires: UI must not own business rules; UI calls application
use cases; use cases call domain logic and repository ports; domain must not
depend on Tauri, React, SQLite or OS APIs. It also allows `packages/domain-ts/`
"only if shared TypeScript domain code is genuinely useful".

Two placements were considered:

1. **Rust domain** — domain, use cases, and SQLite repository all live in
   `src-tauri`; React is a thin view that calls Tauri commands (the use-case
   boundary).
2. **TypeScript domain** — `packages/domain-ts` holds domain + use cases; the
   Rust side exposes raw repository commands over IPC and the TS layer composes
   them.

## Decision

Option 1: implement the domain model, validation, view policy, application use
cases, and SQLite repositories in Rust inside `apps/desktop/src-tauri`.
Tauri commands are the use-case boundary; the React UI renders state and
forwards user intent only.

## Rationale

- One language owns business rules + persistence + reminder reconciliation,
  eliminating a cross-IPC contract for invariants (versioning, lifecycle,
  derived buckets) that are easiest to get wrong.
- `cargo test` with an injected `Clock` gives deterministic domain tests without
  a browser.
- The UI stays thin, satisfying the layering rule with the least surface area.
- `packages/domain-ts` is deferred: it is only justified once a second TS
  consumer (e.g., a web build) exists. Flutter (Milestone 6) implements the
  documented contracts in Dart per ADR 0003.

## Consequences

- Tauri command signatures are the public API of the app core; they must be
  validated and versioned deliberately.
- UI tests mock the command boundary (`@tauri-apps/api` invoke) rather than
  business logic.

## Reversal criteria

If a second TypeScript consumer appears, or Rust-side iteration cost becomes a
bottleneck for UI-adjacent logic, extract a `packages/domain-ts` with an ADR.
