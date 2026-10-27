use chrono::{DateTime, Offset as _, Utc};
use chrono_tz::Tz;

/// Source of "now" and the effective calendar timezone.
///
/// Domain and application code must never read system time directly; they take
/// a `Clock` so tests can inject deterministic instants and timezones.
pub trait Clock: Send + Sync {
    /// Current UTC instant.
    fn now_utc(&self) -> DateTime<Utc>;
    /// Effective calendar timezone (IANA) for wall-clock semantics.
    fn zone(&self) -> Tz;

    /// Current local date in the effective timezone.
    fn today_local(&self) -> chrono::NaiveDate {
        self.now_utc().with_timezone(&self.zone()).date_naive()
    }

    /// Current local datetime in the effective timezone.
    fn now_local(&self) -> chrono::DateTime<chrono::FixedOffset> {
        let dt = self.now_utc().with_timezone(&self.zone());
        let fixed = dt.offset().fix();
        dt.naive_local().and_local_timezone(fixed).unwrap()
    }
}

/// Production clock: system UTC time + system IANA timezone.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn zone(&self) -> Tz {
        iana_time_zone::get_timezone()
            .ok()
            .as_deref()
            .and_then(|s| s.parse::<Tz>().ok())
            .unwrap_or(chrono_tz::UTC)
    }
}

/// Deterministic clock for tests: fixed instant + fixed IANA zone.
#[derive(Debug, Clone)]
pub struct FixedClock {
    instant: DateTime<Utc>,
    zone: Tz,
}

impl FixedClock {
    pub fn new(instant: DateTime<Utc>, zone: Tz) -> Self {
        Self { instant, zone }
    }
}

impl Clock for FixedClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.instant
    }

    fn zone(&self) -> Tz {
        self.zone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_clock_reports_injected_instant_and_zone() {
        let tz: Tz = "America/New_York".parse().unwrap();
        let instant = Utc.with_ymd_and_hms(2026, 10, 26, 12, 0, 0).unwrap();
        let clock = FixedClock::new(instant, tz);
        assert_eq!(clock.now_utc(), instant);
        assert_eq!(clock.zone(), tz);
        // 12:00 UTC = 08:00 EDT in New York.
        assert_eq!(
            clock.today_local(),
            chrono::NaiveDate::from_ymd_opt(2026, 10, 26).unwrap()
        );
    }

    #[test]
    fn system_clock_returns_a_real_instant() {
        let c = SystemClock;
        let now = c.now_utc();
        assert!(now.timestamp() > 0);
        assert!(!c.zone().to_string().is_empty());
    }
}
