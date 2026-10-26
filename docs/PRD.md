# DayFlow Product Requirements Document

## 1. Product
**Name:** DayFlow (working name)  
**Core promise:** 让重要的事情，主动出现在你面前。  
**Platforms:** macOS, Windows, iPhone, Android.  
**Product strategy:** staged delivery, local-first core, architecture ready for eventual sync.

## 2. Priorities
Experience priority: quick glance first, smart reminders second, full task management third. Over time these should balance.

### V1 (local core)
- Create, read, edit, reschedule, complete, cancel, restore completion, and soft-delete tasks.
- Tasks may have a date without a specific time.
- Today view separates timed tasks from unscheduled-time tasks.
- Overdue timed tasks remain in today's list and are marked overdue; never silently move them.
- Legacy tasks remain in a separate “之前未完成” section. Do not automatically move them to today.
- Quick-add defaults to today, no time, normal priority, no category, reminder inherits default.
- Desktop tray/menu-bar quick panel.
- Local persistence and reliable recovery from errors.
- Reminder settings and local notifications, subject to platform capabilities.
- Basic settings, task details, task history fields, accessibility, light/dark appearance.

### V1.1
- Account and multi-device synchronization.
- Conflict detection, tombstone propagation, device registration.
- Sync status and recoverable errors.

### V1.2 / later
- Calendar view, category management enhancements, recurring tasks, advanced history, optional AI natural-language task parsing and decomposition.

## 3. Core user stories
- As a user, I can create a task quickly without opening the full app.
- As a user, I can create a task for a date without assigning a time.
- As a user, I can see today's timed tasks sorted by scheduled time.
- As a user, I can see today's tasks without a time in “待安排”.
- As a user, overdue timed tasks remain visible in today with an “已逾期” marker.
- As a user, I can review legacy tasks and choose whether to reschedule each one.
- As a user, I can complete or cancel a task and stop its pending reminders.
- As a user, I can configure default reminders while explicitly disabling reminders for an individual task.
- As a user, hiding the desktop panel does not discard an unfinished draft.
- As a user, I can continue using core task features offline.

## 4. Desktop quick panel
- Open/close by clicking tray/menu-bar icon.
- Clicking the icon while visible hides the panel; clicking while hidden shows it.
- Clicking outside hides the panel but does not quit the app.
- Escape hides the panel unless a modal/editor has first claim on Escape.
- Closing/hiding the panel does not discard unsaved draft state.
- Panel is a normal floating window; do not force always-on-top unless a future setting explicitly requests it.
- Settings opens the full settings page.
- Window dimensions, placement and focus behavior must be tested on each target OS; do not assume identical OS behavior.
- Provide loading, empty, error, and all-completed states. A database error must never look like an empty list.
- Task row supports completion, title/details, and an overflow menu for edit, reschedule, cancel.
- Legacy section is collapsed by default and shows a count.
- Quick add: title required; default date today; time empty; title trimmed; duplicate titles allowed.
- Panel dismissal preserves in-memory draft. Persist drafts across process restart if practical; otherwise document as a known limitation and prioritize it before release.

## 5. Mobile experience
- Timed reminders use OS notifications with permission-aware behavior.
- Widgets show today's tasks and legacy task count/tasks within platform limits.
- Full task management occurs in the app.
- Do not promise exact widget refresh intervals or unrestricted background execution.
- Provide permission explanation and a useful state when notifications are disabled.

## 6. Reminder defaults
- Task reminder mode: INHERIT_DEFAULT, CUSTOM, DISABLED.
- Untimed tasks do not get scheduled-time reminders by default.
- Changing global defaults must not silently override a task explicitly configured with CUSTOM or DISABLED.
- Legacy tasks do not notify every day merely because they remain incomplete.
- When a task is completed/canceled/deleted, cancel pending reminders.
- Rescheduling recalculates pending reminders. If the target reminder time is already past, do not send a catch-up notification.
- If saving a task succeeds but OS scheduling fails, keep the task and surface/retry the reminder failure.

## 7. Out of scope for V1
- Mandatory account/cloud connection.
- Recurring tasks.
- Team/shared tasks.
- AI task parsing.
- Guaranteed exact-time background execution on mobile OSes.
- Full audit history for every edit.

## 8. Success criteria
- Quick add requires only a title and can be completed quickly.
- Task data persists after app restart.
- Timed and untimed tasks are shown in correct groups.
- Overdue and legacy rules match the domain specification.
- Completing/canceling a task cancels pending reminders.
- Core task operations work without internet.
- App handles storage errors without falsely showing an empty state.
- Desktop app does not quit when quick panel hides.
- Accessibility, keyboard operation and system appearance are supported.
