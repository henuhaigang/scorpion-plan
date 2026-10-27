use thiserror::Error;

/// Errors produced by domain validation and lifecycle rules.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("title must be non-empty after trimming")]
    EmptyTitle,
    #[error("title must be at most 200 characters")]
    TitleTooLong,
    #[error("an all-day schedule must not have a time")]
    AllDayWithTime,
    #[error("reminder mode CUSTOM requires a non-negative minutes_before")]
    InvalidReminder,
    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: super::TaskStatus,
        to: super::TaskStatus,
    },
    #[error("task is soft-deleted; operation not allowed")]
    Deleted,
    #[error("reschedule is not allowed for {0:?} tasks; restore to TODO first")]
    RescheduleNotAllowed(super::TaskStatus),
}
