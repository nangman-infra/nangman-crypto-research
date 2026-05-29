use crate::error::{AppError, AppResult};

pub(super) fn validate_discovery_request(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
    artifact_label: &str,
) -> AppResult<()> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(format!(
            "{artifact_label} S3 bucket must not be empty"
        )));
    }
    if prefix.trim().is_empty() {
        return Err(AppError::config(format!(
            "{artifact_label} S3 prefix must not be empty"
        )));
    }
    if read_limit == 0 {
        return Err(AppError::config(format!(
            "{artifact_label} S3 read limit must be greater than zero"
        )));
    }
    if scan_limit == 0 {
        return Err(AppError::config(format!(
            "{artifact_label} S3 scan limit must be greater than zero"
        )));
    }
    Ok(())
}
