# Acceptance Test Plan

Agent must convert applicable scenarios into automated tests and maintain a platform matrix for tests that require real OS behavior.

## Task creation
- Title is required; trim leading/trailing whitespace.
- Reject empty/whitespace-only title.
- Maximum title length is 200 Unicode code points.
- Duplicate titles are allowed.
- Default date is today in effective calendar timezone; time is null.
- Time accepts a validated 24-hour value and is normalized consistently.
- Past scheduled time is allowed and shown overdue; do not silently rewrite it.
- Task persists across app restart.
- Double submission creates one task only.

## Schedule and views
- Original and current schedule match at creation.
- Reschedule changes current schedule only.
- Untimed tasks are valid and never silently become midnight.
- Today's timed tasks sort by time; untimed tasks are in “待安排”.
- Past-date TODO tasks are legacy; they are not auto-moved.
- Future-date TODO tasks are future.
- Overdue timed tasks remain in today and display overdue.
- Completed/canceled/deleted tasks do not appear in default TODO views.

## Lifecycle
- Completing sets completedAt and removes from TODO lists.
- Undo completion clears completedAt and restores TODO.
- Canceling sets CANCELED and cancels pending reminders.
- Restoring canceled task returns to TODO and recalculates view.
- Soft delete sets deletedAt and keeps tombstone.
- Version increments on every successful mutation.
- Stale expectedVersion causes explicit conflict; newer data is not overwritten.

## Reminders
- INHERIT_DEFAULT follows global default changes.
- CUSTOM is unaffected by global default changes.
- DISABLED remains disabled.
- Untimed tasks have no scheduled-time reminder.
- Past fireAt is not sent as catch-up.
- Reschedule cancels/replaces pending job idempotently.
- Task persistence succeeds even if OS scheduling fails.
- Startup reconciliation can retry failed scheduling.
- Legacy tasks do not notify every day.

## Desktop
- Tray/menu-bar click shows/hides panel.
- Blur/outside click hides without quitting.
- Rapid repeated tray clicks do not produce incorrect visibility.
- Hiding preserves draft.
- Escape hides panel unless a dialog handles it.
- Explicit quit works.
- Panel placement respects display work area, multiple displays and scaling.
- Database read error is visibly distinct from empty list.
- Light/dark theme and keyboard navigation work.
- Test packaged builds, not only dev mode.

## Mobile (later milestone)
- Permission granted/denied states are handled.
- Notifications are best-effort according to OS constraints.
- Widgets show a useful state and refresh within OS limits.
- No claims of unrestricted background execution.

## Test reporting
For every scenario report PASS/FAIL/NOT RUN, OS/device, build version, exact command or manual steps, and evidence. A test that cannot be run is NOT RUN, not PASS.
