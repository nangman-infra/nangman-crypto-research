use crate::error::{AppError, AppResult};

const MAX_S3_OBJECT_KEY_BYTES: usize = 1024;

pub(in crate::storage) fn validate_s3_location(
    bucket: &str,
    key: &str,
    label: &str,
) -> AppResult<()> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(format!(
            "{label} bucket must not be empty"
        )));
    }
    validate_s3_object_key(key, label)
}

pub(super) fn validate_content_type(content_type: &str) -> AppResult<()> {
    if content_type.trim().is_empty() {
        return Err(AppError::config("S3 content type must not be empty"));
    }
    Ok(())
}

pub(in crate::storage) fn validate_research_input_manifest_s3_key(key: &str) -> AppResult<()> {
    let trimmed = key.trim().trim_start_matches('/');
    validate_s3_object_key(trimmed, "research input manifest S3")?;
    if !trimmed.starts_with("research-input-manifest/") {
        return Err(AppError::config(
            "research input manifest S3 key must start with research-input-manifest/",
        ));
    }
    if !(trimmed.ends_with(".json") || trimmed.ends_with(".jsonl")) {
        return Err(AppError::config(
            "research input manifest S3 key must end with .json or .jsonl",
        ));
    }
    Ok(())
}

fn validate_s3_object_key(key: &str, label: &str) -> AppResult<()> {
    if key.trim().is_empty() {
        return Err(AppError::config(format!("{label} key must not be empty")));
    }
    if key.len() > MAX_S3_OBJECT_KEY_BYTES {
        return Err(AppError::config(format!(
            "{label} key must be at most {MAX_S3_OBJECT_KEY_BYTES} bytes"
        )));
    }
    if key.chars().any(|ch| ch.is_control() || ch == '\\') {
        return Err(AppError::config(format!(
            "{label} key must not contain control characters or backslashes"
        )));
    }
    if has_period_only_segment(key) {
        return Err(AppError::config(format!(
            "{label} key must not contain period-only path segments"
        )));
    }
    Ok(())
}

fn has_period_only_segment(key: &str) -> bool {
    key.split('/')
        .map(str::trim)
        .any(|segment| matches!(segment, "." | ".."))
}
