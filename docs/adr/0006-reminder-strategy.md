# ADR 0006: Reminder Strategy (Desktop V1)

- Status: Accepted
- Date: 2026-10-26
- Deciders: DayFlow engineering (agent-assisted)

## Context

`REMINDER_RULES.md` requires: only timed tasks get scheduled-time reminders;
`fireAt = scheduled local date/time − lead`, converted via the effective
calendar timezone; no catch-up for past `fireAt`; idempotent reconciliation on
startup/mutation; task save must succeed even if OS scheduling fails; cancel
pending jobs on complete/cancel/delete.

Platform reality: macOS/Windows notification APIs differ; neither offers a
portable "fire a notification at instant X while the app is closed" primitive
through a maintained cross-platform plugin. `tauri-plugin-notification`
(2.7.x) provides permission handling and `send`, but not OS-level scheduled
delivery.

## Decision

1. **Reminder state lives in SQLite** (`reminder_jobs`: PENDING, SCHEDULED,
   FIRED, CANCELED, FAILED) — the single source of truth for what *should*
   fire.
2. **Delivery is app-resident**: a Rust scheduler task (spawned on app start)
   polls due jobs on a short interval and fires them through
   `tauri-plugin-notification`. The desktop app is a tray-resident app by
   design, so "app running" is the normal state.
3. **Reconciliation** runs on startup and after every task mutation:
   compute the desired job set from task state + settings, diff against
   `reminder_jobs`, and apply idempotent transitions. Scheduling failures mark
   the job FAILED and are retried by the next reconciliation; they never roll
   back the task write.
4. **No hard real-time guarantee** is made to users; UI copy says reminders
   are best-effort while the app is running.

## Consequences

- If the app is fully quit, reminders do not fire (documented limitation;
  acceptable for V1 desktop because the tray app is expected to stay resident).
- All reminder *rules* (lead time, DST, timezone, past-fire suppression) are
  pure Rust functions tested with an injected `Clock` and fixed IANA zones.
- Mobile (Milestone 6) will use OS local notifications with the same
  `reminder_jobs` contract; ownership/dedup across devices is deferred to the
  sync design (ADR to be written in Milestone 7).

## Reversal criteria

If a maintained plugin/OS API provides reliable scheduled delivery on both
macOS and Windows, prefer it for delivery while keeping the `reminder_jobs`
state model and reconciliation.
