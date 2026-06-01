use crate::error::{AppError, AppResult};

pub(super) fn trimmed_utf8<'a>(label: &str, bytes: &'a [u8]) -> AppResult<&'a str> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    Ok(trimmed)
}
