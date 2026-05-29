use crate::error::{AppError, AppResult};
use crate::storage::partition::normalize_prefix;

pub(super) fn output_prefix(
    prefix: &str,
    default_prefix: &str,
    required_prefix: &str,
    error_message: &'static str,
) -> AppResult<String> {
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        default_prefix
    } else {
        prefix
    });
    if !prefix.starts_with(required_prefix) {
        return Err(AppError::config(error_message));
    }
    Ok(prefix)
}

pub(super) fn validate_key_component(value: &str, label: &str) -> AppResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::config(format!("{label} must not be empty")));
    }
    if matches!(trimmed, "." | "..")
        || trimmed.contains('/')
        || trimmed.chars().any(char::is_control)
    {
        return Err(AppError::config(format!(
            "{label} must be a single safe S3 key segment"
        )));
    }
    Ok(())
}
