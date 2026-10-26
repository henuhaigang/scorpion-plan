# Architecture

## 1. Recommended stack
- Desktop V1: Tauri 2, React, TypeScript.
- Native desktop integration: Rust/Tauri commands and maintained official APIs.
- Mobile later: Flutter.
- Local DB: SQLite with versioned migrations.
- Future cloud: Java, Spring Boot, PostgreSQL.
- Tests: frontend unit tests, Rust tests where useful, integration tests around repositories/use cases, and platform acceptance tests.

This is a recommendation and must be verified by the coding agent against current supported versions and OS behavior before lock-in. Record changes in ADRs.

## 2. Layers
- UI: renders state and forwards user intent.
- Application/use cases: CreateTask, UpdateTask, RescheduleTask, CompleteTask, UndoCompletion, CancelTask, RestoreCanceledTask, SoftDeleteTask, ListTodayTasks, ListLegacyTasks.
- Domain: Task, TaskSchedule, ReminderPreference, TaskViewPolicy and lifecycle rules.
- Ports: TaskRepository, CategoryRepository, SettingsRepository, ReminderScheduler, Clock.
- Infrastructure: SQLite implementation, OS notifications, desktop tray/window integration, future sync adapter.

UI must not contain business rules or direct SQL. Domain must not depend on Tauri, React, Flutter, SQLite or OS APIs.

## 3. Desktop structure suggestion
- `apps/desktop/` React + TypeScript frontend
- `apps/desktop/src-tauri/` Rust host and platform integration
- `packages/domain-ts/` only if shared TypeScript domain code is genuinely useful
- `docs/` specifications and ADRs
Do not force cross-language code sharing. Flutter can implement the same documented domain contracts in Dart.

## 4. Desktop quick-panel behavior
- Tray click toggles visibility.
- Outside click/blur hides panel but never quits app.
- Escape hides panel unless a child dialog consumes it.
- Hiding must not destroy the window/webview.
- Prevent blur/tray event races using a tested state machine, not only arbitrary sleeps.
- Window placement must account for screen work area, display scale, menu/taskbar position, multi-monitor and edge clipping.
- Use native platform behavior where needed; document differences.
- Provide a way to quit the app explicitly from tray/menu.
- Ensure keyboard focus enters a sensible control when opened.
- Preserve draft on hide; persistence across process restart is a release priority.
- Do not claim macOS menu-bar and Windows tray behavior are equivalent without tests.

## 5. Security
- Keep Tauri capabilities/permissions least-privilege.
- Do not expose arbitrary shell execution or filesystem access to frontend.
- Validate IPC inputs.
- Use parameterized SQL.
- No secrets in source control.
- Provide clear local data location and backup/export strategy before release.
