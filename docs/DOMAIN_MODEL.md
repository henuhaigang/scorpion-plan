# DayFlow Domain Model

## 1. Task identity and fields
Task is an entity identified by UUID generated at creation. A suggested logical model:

- `id: UUID`
- `title: String` (trimmed, non-empty, max 200 Unicode code points)
- `status: TaskStatus`
- `originalSchedule: TaskSchedule`
- `currentSchedule: TaskSchedule`
- `priority: TaskPriority`
- `categoryId: UUID?`
- `reminderPreference: ReminderPreference`
- `createdAt: Instant`
- `updatedAt: Instant`
- `completedAt: Instant?`
- `deletedAt: Instant?`
- `version: Long` (starts at 1 and increments on every successful mutation)
- optional diagnostic `lastReminderTimezoneId` only if needed; do not treat it as immutable task timezone by default.

## 2. Enums
`TaskStatus`: TODO, COMPLETED, CANCELED only.  
`TaskPriority`: LOW, NORMAL, HIGH.  
`ReminderMode`: INHERIT_DEFAULT, CUSTOM, DISABLED.  
`TaskBucket` (derived, not persisted as task status): TODAY, FUTURE, LEGACY, COMPLETED, CANCELED, DELETED.  
`isOverdue` is derived, not a persisted task status.

## 3. Schedule semantics
`TaskSchedule`:
- `date: LocalDate` (required)
- `time: LocalTime?` (optional)
- `allDay: boolean`
- If `allDay == true`, `time` must be null.
- If `allDay == false` and `time == null`, this is a valid task with no specific time.
- Do not represent “no time” as 00:00.
- For V1, task date/time follows the user's effective calendar timezone (wall-clock semantics). When timezone changes, preserve calendar date and wall-clock time, and recalculate pending reminder instants. Document and test daylight-saving edge cases on platforms that observe DST.
- Timestamps such as createdAt and completedAt are UTC instants.
- If product later requires a task tied to a fixed global instant, add a distinct explicit concept rather than changing current semantics silently.

## 4. Original vs current schedule
At creation, originalSchedule equals currentSchedule.
Reschedule changes currentSchedule only. It never overwrites originalSchedule.
For V1, direct rescheduling of COMPLETED/CANCELED tasks is disallowed; user must restore to TODO first.
Rescheduling preserves time if the task had a specific time. User may explicitly clear it. Untimed tasks remain untimed unless user explicitly sets a time.

## 5. Lifecycle transitions
- TODO -> COMPLETED: set completedAt = now; keep schedule.
- COMPLETED -> TODO: undo completion; clear completedAt; recalculate bucket and reminders; do not send a missed reminder.
- TODO -> CANCELED: set status CANCELED; keep schedule; cancel pending reminders.
- CANCELED -> TODO: explicit restore action; clear any terminal-specific fields if introduced; recalculate bucket/reminders.
- Soft delete: set deletedAt; exclude from normal views; cancel pending reminders. Deletion is distinct from cancellation.
- All successful changes update updatedAt and increment version.
- A stale expectedVersion must return a conflict; never silently overwrite newer data.

## 6. Derived view policy
For active TODO tasks:
- scheduled date before local today => LEGACY.
- scheduled date after local today => FUTURE.
- scheduled date equals local today => TODAY.
- Within TODAY, timed tasks sorted ascending by time; untimed tasks appear in “待安排”.
- A timed task is overdue if scheduled date is today and its scheduled time is earlier than current local time.
- Untimed tasks are not overdue by default.
- Completed, canceled, and deleted tasks are not overdue.
- A legacy task stays in legacy; do not auto-reschedule it.
- Stable sort tie-breakers: scheduled time, createdAt, then id.

## 7. Reminder preference validation
- INHERIT_DEFAULT: minutesBefore is null.
- CUSTOM: minutesBefore is a non-negative integer; 0 means at scheduled time.
- DISABLED: minutesBefore is null.
- Untimed tasks have no scheduled-time reminder even if preference is INHERIT_DEFAULT or CUSTOM.
- A task explicitly DISABLED remains disabled when global defaults change.

## 8. Domain boundaries
Domain entities must not call database APIs, OS notification APIs, UI APIs, or read system time directly. Pass `now` into behavior methods and inject a Clock into application use cases.
