use chrono::{NaiveDate, NaiveTime};

use super::error::DomainError;

/// A calendar date with an optional wall-clock time.
///
/// Invariants:
/// - `all_day == true`  => `time` is `None`.
/// - `all_day == false` and `time == None` is a valid "no specific time" schedule.
/// - "No time" is never represented as `00:00`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TaskSchedule {
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub all_day: bool,
}

impl TaskSchedule {
    /// A date without a specific time (the quick-add default).
    pub fn date_only(date: NaiveDate) -> Self {
        Self {
            date,
            time: None,
            all_day: false,
        }
    }

    /// An explicit all-day schedule.
    pub fn all_day(date: NaiveDate) -> Self {
        Self {
            date,
            time: None,
            all_day: true,
        }
    }

    /// A schedule with a specific wall-clock time.
    pub fn timed(date: NaiveDate, time: NaiveTime) -> Self {
        Self {
            date,
            time: Some(time),
            all_day: false,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.all_day && self.time.is_some() {
            return Err(DomainError::AllDayWithTime);
        }
        Ok(())
    }

    /// Whether this schedule has a specific wall-clock time.
    pub fn is_timed(&self) -> bool {
        self.time.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn date_only_is_valid_and_untimed() {
        let s = TaskSchedule::date_only(d("2026-10-26"));
        assert!(s.validate().is_ok());
        assert!(!s.is_timed());
        assert!(!s.all_day);
    }

    #[test]
    fn all_day_with_time_is_rejected() {
        let s = TaskSchedule {
            date: d("2026-10-26"),
            time: Some(NaiveTime::MIN),
            all_day: true,
        };
        assert_eq!(s.validate(), Err(DomainError::AllDayWithTime));
    }

    #[test]
    fn midnight_is_a_real_time_not_a_no_time_marker() {
        let s = TaskSchedule::timed(d("2026-10-26"), NaiveTime::MIN);
        assert!(s.validate().is_ok());
        assert!(s.is_timed());
    }
}
