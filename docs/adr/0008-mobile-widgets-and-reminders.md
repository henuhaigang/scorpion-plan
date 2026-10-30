# ADR 0008: Mobile Widgets and Reminders

- Status: Accepted
- Date: 2026-10-30

## Context

The mobile app stores tasks in private SQLite. Android AppWidgets and iOS
WidgetKit extensions run outside the Flutter UI process and need a safe
cross-process view of today's tasks and the legacy count. Users also asked to
complete a task directly from the widget and receive a reminder ten minutes
before a timed task.

Widgets and notifications are controlled by their operating systems. Widget
refresh schedules and notification delivery cannot be guaranteed to exact
deadlines.

## Decision

- The Flutter app publishes a compact JSON snapshot to Android widget storage
  and to the iOS App Group `group.com.dayflow.dayflowMobile`; widgets never
  query SQLite directly.
- Android widget completion sends a validated task identifier to a Flutter
  background callback, which performs the normal versioned SQLite transition.
- iOS 17+ App Intents optimistically remove the task from the shared widget
  snapshot and persist a completion queue. The app applies that queue to SQLite
  on its next activation. iOS 15–16 widgets remain view-only and open DayFlow
  when tapped.
- Android widget callbacks do not close their temporary repository: sqflite
  may return the path-cached database handle also used by the foreground app.
- Timed TODO tasks inherit a ten-minute reminder. CUSTOM and DISABLED
  preferences are honored. Untimed, terminal, deleted, and past-fire-time tasks
  are not scheduled.
- Permission is requested in context when a timed task is saved, after an
  explanation. Denial does not block task persistence; the app shows a
  recoverable permission/status message. Reminders use inexact best-effort
  platform scheduling and are reconciled on mutation and app activation.

## Consequences

- iOS interactive widgets require iOS 17 or later; the app's iOS 15 minimum is
  retained for read-only widgets and task management.
- Real-device distribution requires matching App Group entitlements and
  provisioning profiles for both the app and widget extension.
- Widget refresh and notification delivery can be delayed by the OS.
- The task database remains the source of truth; an iOS widget completion is
  durably queued in the App Group until the app can apply it.
- Android background callbacks keep the cached SQLite connection available so
  widget completion cannot invalidate the foreground task repository.
