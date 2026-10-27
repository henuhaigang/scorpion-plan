use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;

use super::reminder::{GlobalReminderDefault, ReminderPreference};
use super::task::Task;

/// Compute the UTC instant at which a reminder should fire, or `None` if no
/// reminder is warranted.
///
/// Rules (from REMINDER_RULES.md):
/// - Only TODO tasks with a specific time can get scheduled-time reminders.
/// - `fire_at = scheduled local date/time − lead duration`, converted to UTC.
/// - If `fire_at <= now`, do not schedule a catch-up notification.
/// - `DISABLED` preference never reminds.
/// - `INHERIT_DEFAULT` follows the global default (enabled + minutes).
/// - `CUSTOM` uses its own minutes.
/// - DST gaps (nonexistent local time) → no reminder.
/// - DST overlaps (ambiguous local time) → use the earliest occurrence.
pub fn compute_fire_at(
    task: &Task,
    global_default: &GlobalReminderDefault,
    now: DateTime<Utc>,
    zone: Tz,
) -> Option<DateTime<Utc>> {
    // Only active TODO tasks that are not deleted.
    if task.status != crate::domain::TaskStatus::Todo {
        return None;
    }
    if task.deleted_at.is_some() {
        return None;
    }

    // Only tasks with a specific time get scheduled-time reminders in V1.
    let time = task.current_schedule.time?;

    // Determine the lead time in minutes.
    let minutes_before: u32 = match task.reminder_preference {
        ReminderPreference::Disabled => return None,
        ReminderPreference::InheritDefault => {
            if !global_default.enabled {
                return None;
            }
            global_default.minutes_before
        }
        ReminderPreference::Custom { minutes_before } => minutes_before,
    };

    // Convert the scheduled local date/time to a UTC instant.
    let scheduled_naive = task.current_schedule.date.and_time(time);
    let scheduled_local = scheduled_naive.and_local_timezone(zone);

    let scheduled_instant: DateTime<Utc> = match scheduled_local {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(earlier, _) => earlier,
        chrono::LocalResult::None => return None,
    }
    .with_timezone(&Utc);

    let fire_at = scheduled_instant - Duration::minutes(minutes_before as i64);

    // No catch-up for past fire times.
    if fire_at <= now {
        return None;
    }

    Some(fire_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Task, TaskPriority, TaskSchedule};
    use chrono::TimeZone;

    fn make_task(date: &str, time: Option<&str>, preference: ReminderPreference) -> Task {
        let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let t = time.map(|s| chrono::NaiveTime::parse_from_str(s, "%H:%M").unwrap());
        let schedule = TaskSchedule {
            date: d,
            time: t,
            all_day: false,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        Task::new(
            uuid::Uuid::new_v4(),
            "test".to_string(),
            schedule,
            TaskPriority::Normal,
            None,
            preference,
            now,
        )
        .unwrap()
    }

    fn zone(s: &str) -> Tz {
        s.parse().unwrap()
    }

    #[test]
    fn timed_task_with_inherit_default_gets_reminder() {
        let task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::InheritDefault,
        );
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        // Now is 00:00 UTC on 2026-10-26. Zone is UTC for simplicity.
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let fire_at = compute_fire_at(&task, &global, now, zone("UTC")).unwrap();
        // 14:00 UTC - 10 min = 13:50 UTC
        assert_eq!(
            fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 50, 0).unwrap()
        );
    }

    #[test]
    fn untimed_task_never_gets_reminder() {
        let task = make_task("2026-10-26", None, ReminderPreference::InheritDefault);
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn disabled_preference_never_gets_reminder() {
        let task = make_task("2026-10-26", Some("14:00"), ReminderPreference::Disabled);
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn inherit_default_with_global_disabled_gets_no_reminder() {
        let task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::InheritDefault,
        );
        let global = GlobalReminderDefault {
            enabled: false,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn custom_preference_ignores_global_default() {
        let task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::Custom { minutes_before: 30 },
        );
        let global = GlobalReminderDefault {
            enabled: false,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let fire_at = compute_fire_at(&task, &global, now, zone("UTC")).unwrap();
        // 14:00 UTC - 30 min = 13:30 UTC
        assert_eq!(
            fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 13, 30, 0).unwrap()
        );
    }

    #[test]
    fn past_fire_at_is_suppressed() {
        let task = make_task(
            "2026-10-26",
            Some("00:05"),
            ReminderPreference::InheritDefault,
        );
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        // Now is 00:00 UTC; fire_at would be 23:55 UTC yesterday → past.
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn completed_task_gets_no_reminder() {
        let mut task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::InheritDefault,
        );
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        task.complete(now).unwrap();
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn canceled_task_gets_no_reminder() {
        let mut task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::InheritDefault,
        );
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        task.cancel(now).unwrap();
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn deleted_task_gets_no_reminder() {
        let mut task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::InheritDefault,
        );
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        task.soft_delete(now).unwrap();
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }

    #[test]
    fn custom_zero_minutes_fires_at_scheduled_time() {
        let task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::Custom { minutes_before: 0 },
        );
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let fire_at = compute_fire_at(&task, &global, now, zone("UTC")).unwrap();
        assert_eq!(
            fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 14, 0, 0).unwrap()
        );
    }

    #[test]
    fn timezone_conversion_uses_effective_zone() {
        // Task scheduled at 14:00 in Asia/Shanghai (UTC+8) = 06:00 UTC.
        let task = make_task(
            "2026-10-26",
            Some("14:00"),
            ReminderPreference::InheritDefault,
        );
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let fire_at = compute_fire_at(&task, &global, now, zone("Asia/Shanghai")).unwrap();
        // 14:00 CST - 10 min = 13:50 CST = 05:50 UTC
        assert_eq!(
            fire_at,
            Utc.with_ymd_and_hms(2026, 10, 26, 5, 50, 0).unwrap()
        );
    }

    #[test]
    fn dst_ambiguous_time_uses_earliest_occurrence() {
        // 2026-11-01 in America/New_York: clocks fall back at 02:00 → 01:00.
        // 01:30 occurs twice. We use the earliest (EDT, UTC-4).
        let task = make_task(
            "2026-11-01",
            Some("01:30"),
            ReminderPreference::Custom { minutes_before: 0 },
        );
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let fire_at = compute_fire_at(&task, &global, now, zone("America/New_York")).unwrap();
        // Earliest 01:30 EDT = 05:30 UTC
        assert_eq!(
            fire_at,
            Utc.with_ymd_and_hms(2026, 11, 1, 5, 30, 0).unwrap()
        );
    }

    #[test]
    fn dst_gap_nonexistent_time_gets_no_reminder() {
        // 2026-03-08 in America/New_York: clocks spring forward at 02:00 → 03:00.
        // 02:30 does not exist.
        let task = make_task(
            "2026-03-08",
            Some("02:30"),
            ReminderPreference::Custom { minutes_before: 0 },
        );
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        assert!(compute_fire_at(&task, &global, now, zone("America/New_York")).is_none());
    }

    #[test]
    fn all_day_task_gets_no_reminder() {
        let d = chrono::NaiveDate::parse_from_str("2026-10-26", "%Y-%m-%d").unwrap();
        let schedule = TaskSchedule {
            date: d,
            time: None,
            all_day: true,
        };
        let now = Utc.with_ymd_and_hms(2026, 10, 26, 0, 0, 0).unwrap();
        let task = Task::new(
            uuid::Uuid::new_v4(),
            "all day".to_string(),
            schedule,
            TaskPriority::Normal,
            None,
            ReminderPreference::InheritDefault,
            now,
        )
        .unwrap();
        let global = GlobalReminderDefault {
            enabled: true,
            minutes_before: 10,
        };
        assert!(compute_fire_at(&task, &global, now, zone("UTC")).is_none());
    }
}
