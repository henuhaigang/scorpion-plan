# Architecture

## 1. Implemented stack and future scope
- Desktop: Tauri 2, React, TypeScript, Rust, and SQLite.
- Mobile: Flutter/Dart, SQLite, platform notification adapters, Android AppWidget, and iOS WidgetKit.
- Local databases use versioned migrations; core task operations work offline.
- Future cloud: Java, Spring Boot, PostgreSQL. Cloud sync is not part of V1.
- Validation uses frontend and Dart tests, Rust tests, mobile integration tests, and platform acceptance checks where the OS permits them.

Framework and storage decisions are recorded in `docs/adr/`. Builds, simulator evidence, and remaining platform gaps are tracked in `docs/V1_RELEASE.md` and the milestone reports.

## 2. Layers
- UI: renders state and forwards user intent.
- Application/use cases: CreateTask, UpdateTask, RescheduleTask, CompleteTask, UndoCompletion, CancelTask, RestoreCanceledTask, SoftDeleteTask, ListTodayTasks, ListLegacyTasks.
- Domain: Task, TaskSchedule, ReminderPreference, TaskViewPolicy and lifecycle rules.
- Ports: TaskRepository, CategoryRepository, SettingsRepository, ReminderScheduler, Clock.
- Infrastructure: SQLite implementation, OS notifications, desktop tray/window integration, future sync adapter.

UI must not contain business rules or direct SQL. Domain must not depend on Tauri, React, Flutter, SQLite or OS APIs.

## 3. Client structure
- `apps/desktop/` React + TypeScript frontend
- `apps/desktop/src-tauri/` Rust host and platform integration
- `apps/mobile/` Flutter app, Dart application/domain code, and native platform extensions
- `docs/` specifications, ADRs, and milestone evidence
Do not force cross-language code sharing. Desktop Rust and mobile Dart implement the documented domain contracts independently.

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
