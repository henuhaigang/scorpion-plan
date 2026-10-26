# Implementation Plan for Coding Agent

Implement in this order. Do not start all milestones at once.

## Milestone 0 — Repository and ADR baseline
- Inspect environment and repository.
- Create monorepo skeleton and `.gitignore`.
- Add ADR for desktop framework, local DB library, reminder strategy, and app packaging.
- Pin compatible dependency versions.
- Add CI scripts for formatting, lint, unit tests and build.
- Report platform limitations honestly.

Exit criteria: clean baseline build and written technical decisions.

## Milestone 1 — Domain and local persistence
- Implement domain types and validation.
- Implement SQLite migrations and repositories.
- Implement optimistic version updates and soft deletion.
- Implement derived task view policy.
- Add comprehensive deterministic tests with injected Clock.

Exit criteria: domain and repository tests pass; persistence survives restart; no UI required to verify core rules.

## Milestone 2 — Desktop quick panel
- Implement Tauri 2 + React + TypeScript UI.
- Implement tray/menu-bar toggle, focus, blur and hide behavior.
- Implement robust event coordination and explicit quit action.
- Preserve draft on hide and across restart if feasible; treat restart persistence as release priority.
- Test on macOS in packaged build; add Windows test plan and run if environment is available.

Exit criteria: acceptance tests reported with PASS/FAIL/NOT RUN and platform evidence.

## Milestone 3 — Task management UI
- Quick add, task detail/edit, complete, undo, cancel, restore, reschedule.
- Today timed section, unscheduled section, legacy section, future list.
- Empty/loading/error states and accessibility.
- No business rules embedded in UI.

Exit criteria: UI integration tests plus acceptance tests pass.

## Milestone 4 — Reminder engine
- Implement typed settings and ReminderScheduler port.
- Implement idempotent reconciliation and OS adapters.
- Handle permissions, failures, reschedule, completion, cancellation and timezone changes.
- Test DST and timezone behavior.

Exit criteria: automated rule tests pass; platform notification status documented.

## Milestone 5 — Release hardening
- App packaging, signing guidance, auto-update decision (do not implement auto-update without security review), crash/error handling, logs without sensitive content, backup/export strategy.
- Verify upgrade migrations and packaged app.
- Accessibility and keyboard review.
- Final release checklist.

Exit criteria: build artifacts can be produced by documented commands; no unreported known critical defects.

## Milestone 6 — Mobile app
- Start Flutter app after desktop domain and data contracts stabilize.
- Implement local SQLite, task flows, notifications and widgets.
- Verify iOS and Android separately.

## Milestone 7 — Cloud sync (later)
- Design Java/Spring Boot API and PostgreSQL schema.
- Define authentication, ownership, cursor protocol, tombstone propagation, conflict policy and notification ownership before implementation.
- Do not ship sync based solely on `updated_at` timestamps.

## Required end-of-milestone report
Summarize files, commands, actual results, untested platforms, known issues, and next step. Never claim tests passed without running them.
