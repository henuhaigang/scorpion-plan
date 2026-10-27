use super::error::DomainError;

/// Task-level reminder preference.
///
/// - `InheritDefault`: follows the global default (no task-level minutes).
/// - `Custom { minutes_before }`: explicit lead time; `0` means "at scheduled time".
/// - `Disabled`: never remind, even if the global default changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReminderPreference {
    InheritDefault,
    Custom { minutes_before: u32 },
    Disabled,
}

impl ReminderPreference {
    pub fn validate(&self) -> Result<(), DomainError> {
        // u32 guarantees non-negative; the invariant is structural.
        match self {
            ReminderPreference::InheritDefault => {}
            ReminderPreference::Custom { minutes_before: _ } => {}
            ReminderPreference::Disabled => {}
        }
        Ok(())
    }

    /// Whether this preference can ever produce a scheduled-time reminder.
    pub fn can_remind(&self) -> bool {
        !matches!(self, ReminderPreference::Disabled)
    }
}

/// The global reminder default that `InheritDefault` tasks follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GlobalReminderDefault {
    /// Whether inheriting tasks get a reminder at all.
    pub enabled: bool,
    /// Lead time in minutes; `0` means "at scheduled time".
    pub minutes_before: u32,
}

impl Default for GlobalReminderDefault {
    fn default() -> Self {
        Self {
            enabled: true,
            minutes_before: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_zero_means_at_scheduled_time() {
        let p = ReminderPreference::Custom { minutes_before: 0 };
        assert!(p.validate().is_ok());
        assert!(p.can_remind());
    }

    #[test]
    fn disabled_never_reminds() {
        assert!(!ReminderPreference::Disabled.can_remind());
    }

    #[test]
    fn global_default_round_trips() {
        let g = GlobalReminderDefault {
            enabled: false,
            minutes_before: 0,
        };
        let json = serde_json::to_string(&g).unwrap();
        let back: GlobalReminderDefault = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}
