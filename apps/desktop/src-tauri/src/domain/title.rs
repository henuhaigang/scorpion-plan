use super::error::DomainError;

/// Maximum title length in Unicode code points.
pub const MAX_TITLE_LEN: usize = 200;

/// Trim and validate a task title.
///
/// - Leading/trailing whitespace is trimmed.
/// - Empty or whitespace-only titles are rejected.
/// - More than 200 Unicode code points is rejected.
pub fn normalize_title(raw: &str) -> Result<String, DomainError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(DomainError::EmptyTitle);
    }
    if trimmed.chars().count() > MAX_TITLE_LEN {
        return Err(DomainError::TitleTooLong);
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_whitespace() {
        assert_eq!(normalize_title("  写周报  ").unwrap(), "写周报");
    }

    #[test]
    fn rejects_empty_and_whitespace_only() {
        assert_eq!(normalize_title(""), Err(DomainError::EmptyTitle));
        assert_eq!(normalize_title("   \t\n "), Err(DomainError::EmptyTitle));
    }

    #[test]
    fn enforces_200_code_points_not_bytes() {
        // 200 CJK chars = 600 bytes, must pass.
        let ok: String = "中".repeat(200);
        assert!(normalize_title(&ok).is_ok());
        // 201 must fail.
        let bad: String = "中".repeat(201);
        assert_eq!(normalize_title(&bad), Err(DomainError::TitleTooLong));
    }

    #[test]
    fn allows_internal_whitespace() {
        assert_eq!(normalize_title("a  b").unwrap(), "a  b");
    }
}
