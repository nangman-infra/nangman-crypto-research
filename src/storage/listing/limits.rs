use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy)]
pub(super) struct PayloadListOptions<'a> {
    pub(super) file_suffix: &'a str,
    pub(super) scan_limit: usize,
    pub(super) artifact_label: &'a str,
}

pub(super) fn ensure_scan_limit(
    object_count: usize,
    bucket: &str,
    prefix: &str,
    options: PayloadListOptions<'_>,
) -> AppResult<()> {
    if object_count > options.scan_limit {
        return Err(scan_limit_exceeded_error(bucket, prefix, options));
    }
    Ok(())
}

pub(super) fn scan_limit_exceeded_error(
    bucket: &str,
    prefix: &str,
    options: PayloadListOptions<'_>,
) -> AppError {
    AppError::validation(format!(
        "{} S3 scan limit exceeded for s3://{bucket}/{prefix}: limit={}; narrow the prefix",
        options.artifact_label, options.scan_limit
    ))
}
