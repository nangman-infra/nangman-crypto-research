use super::*;

pub(super) async fn load_latest_shadow_runs(args: &Args) -> AppResult<Vec<ShadowValidationRun>> {
    let output_bucket = args.output_s3_bucket.as_deref().ok_or_else(|| {
        AppError::config("--run-shadow-cycle-from-latest-state requires --output-s3-bucket")
    })?;
    let shadow_keys = discover_shadow_validation_run_keys_from_s3(
        output_bucket,
        DEFAULT_SHADOW_VALIDATION_RUN_PREFIX,
        DEFAULT_SHADOW_VALIDATION_RUN_READ_LIMIT,
        DEFAULT_SHADOW_VALIDATION_RUN_SCAN_LIMIT,
    )
    .await?;
    if shadow_keys.is_empty() {
        return Ok(Vec::new());
    }
    read_shadow_validation_runs_from_s3(output_bucket, &shadow_keys).await
}
