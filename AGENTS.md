# Agent Operating Contract — DayFlow

Read this file and all documents referenced below before implementing features.

## Required reading
1. `docs/PRD.md`
2. `docs/DOMAIN_MODEL.md`
3. `docs/DATA_AND_SYNC.md`
4. `docs/REMINDER_RULES.md`
5. `docs/ARCHITECTURE.md`
6. `docs/ACCEPTANCE_TESTS.md`
7. `docs/IMPLEMENTATION_PLAN.md`

## Working rules
- Do not ask the product owner to manually validate every small step. Use automated tests, static analysis, build checks, and platform-specific test plans wherever possible.
- Do not claim a build, test, or platform behavior passed unless you actually ran it. Report unavailable platforms and unverified behaviors explicitly.
- Before editing, inspect the existing repository, toolchain versions, and working tree. Never overwrite user work without understanding it.
- Work milestone-by-milestone. After each milestone: run relevant formatters, lint, unit tests, integration tests, and build checks; fix failures before moving on.
- Keep changes small and reviewable. Explain changed files, commands run, results, remaining risks, and exact next action.
- If a product requirement conflicts with OS restrictions or framework capability, document the conflict and propose a decision in an ADR. Do not silently weaken requirements.
- Prefer official APIs and maintained dependencies. Pin compatible versions; avoid unmaintained plugins and duplicate dependencies.
- Do not expose secrets, tokens, private keys, or local environment values in logs or committed files.
- Do not add analytics, advertising, telemetry, account login, or cloud dependency to V1 without explicit product approval.
- No destructive database migration without backup/upgrade tests. Use forward migrations and preserve existing data.
- UI code must not own business rules. UI calls application use cases; use cases call domain logic and repository ports.
- No direct SQL in UI code. No OS notification API calls from domain entities.
- Avoid `TODO` placeholders in features declared complete. If blocked, state the blocker and leave a compiling, tested boundary where possible.

## Mandatory engineering constraints
- Stable UUID task IDs from creation.
- Keep original plan separate from current schedule.
- `TODO`, `COMPLETED`, `CANCELED` are the only task business statuses.
- TODAY/FUTURE/LEGACY/OVERDUE/UNSCHEDULED are derived view states, never persisted as task status.
- A task without a time is valid and must never be silently assigned midnight.
- A canceled task is not deleted. Deletion is a tombstone/soft delete.
- Reminder preference and actual scheduled reminder jobs are separate concepts.
- Task save must not roll back just because OS reminder scheduling failed.
- Prevent duplicate submissions and stale-version writes.
- No network requirement for core local task CRUD.
- Store timestamps as UTC instants; store calendar date/time separately with explicit timezone policy.
- User-facing copy should be concise, accessible, and localized-ready; initial UI language is Simplified Chinese.
- Use dependency injection for current time (`Clock` or equivalent) in domain/application tests.
- Keep a changelog of decisions in `docs/adr/`.

## Default technical direction
- Desktop: Tauri 2 + React + TypeScript.
- Mobile: Flutter, implemented after desktop core is stable.
- Local persistence: SQLite with versioned migrations.
- Future cloud: Java + Spring Boot + PostgreSQL.
- Shared contract: document API/schema first; do not force Rust, TypeScript, Dart and Java into an artificial shared-code build.
- The desktop framework choice is a recommendation, not permission to ignore platform tests. If a required tray/popover behavior cannot be implemented reliably, write an ADR comparing alternatives before changing frameworks.

## Required final report for every milestone
1. What changed.
2. Files added/modified.
3. Exact commands run.
4. Actual test/build output summary.
5. What was not tested and why.
6. Remaining risks.
7. Next milestone.
