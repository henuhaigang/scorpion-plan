# Reminder Rules

## Preferences
Task-level mode:
- `INHERIT_DEFAULT`
- `CUSTOM(minutesBefore)`
- `DISABLED`

Global defaults are a typed configuration. Changing the global default affects only tasks inheriting the default.

## Scheduling
- Only tasks with a specific time can get scheduled-time reminders in V1.
- `fireAt = scheduled local date/time minus lead duration`, converted to an instant using the effective calendar timezone.
- If `fireAt <= now`, do not schedule a catch-up notification.
- On task reschedule, reminder preference change, global default change, timezone change, app startup, or scheduler recovery, reconcile pending reminder jobs.
- On complete, cancel, restore-to-canceled, or soft delete, cancel pending jobs.
- Legacy tasks do not generate daily nag notifications.
- A task save must succeed independently of OS scheduler success. Show/retry reminder failure rather than rolling back task data.
- Avoid duplicate schedules for the same task/version/target fire time. Make reconciliation idempotent.

## Platform constraints
- macOS, Windows, iOS and Android notification APIs differ.
- The OS may delay notifications or restrict background scheduling; never promise hard real-time delivery.
- Request permissions contextually and explain how to enable notifications if denied.
- On startup, reconcile app-owned reminder state with OS-supported state where APIs permit.
- Future multi-device sync must define which device owns a notification or implement a deduplication strategy.

## Testing
Use an injected clock and deterministic timezone test cases. Cover daylight-saving gaps/overlaps, time-zone changes, past reminder time, task completion, cancellation, deletion, reschedule, inherited default change, custom override, disabled override, and scheduler failure.
