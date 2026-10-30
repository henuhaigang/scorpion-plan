# ADR 0003: Mobile Platform

- Status: Accepted
- Decision: Flutter for iOS and Android
- Rationale: one mobile UI codebase, suitable for task management and widgets/notifications through platform plugins and native integration.
- Risks: widgets and notification scheduling are OS-controlled; plugin support and platform restrictions must be verified separately on iOS and Android.
- Rule: mobile implementation starts after domain contracts and desktop local core stabilize.

## Platform notes

- Android home-screen widgets use the native AppWidget API and may be refreshed
  by the OS at intervals; task mutations also request an immediate update.
- iOS uses a WidgetKit extension with an App Group snapshot. Interactive task
  completion uses App Intents on iOS 17 and later. iOS 15–16 users can view the
  widget and open the app but do not get the completion button.
- iOS widget completion is queued in the App Group and applied to SQLite when
  DayFlow next becomes active. The persisted queue survives app suspension.
- Local reminders are best-effort. Android uses inexact alarms, and neither
  platform promises exact delivery or unrestricted background execution.
- The mobile app uses a shared Material 3 color scheme; Android AppWidget and
  iOS WidgetKit surfaces follow a compact indigo/slate palette with explicit
  untimed-task labels. Widget placement, layout, and refresh remain OS-managed.
- SQLite initialization is single-flight so the first task-list load and
  startup platform synchronization cannot independently open/replace the
  database connection.
