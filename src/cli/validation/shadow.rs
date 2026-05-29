use super::*;
use crate::error::{AppError, AppResult};

pub(in crate::cli) fn validate_shadow_cycle_build_args(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() && args.output_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        ));
    }
    if let Some(path) = args.shadow_cycle_decision_output_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_SHADOW_CYCLE_DECISION_OUTPUT_FILE must be an absolute path",
        ));
    }
    if args.shadow_cycle_decision_output_file.is_none()
        && args.output_dir.is_none()
        && args.output_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--build-shadow-cycle-decision requires --shadow-cycle-decision-output-file, --output-dir, or --output-s3-bucket",
        ));
    }
    if !args.shadow_validation_run_s3_keys.is_empty()
        && args.shadow_validation_run_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--shadow-validation-run-s3-bucket is required when --shadow-validation-run-s3-key is set",
        ));
    }
    if args.shadow_validation_run_files.is_empty()
        && args.shadow_validation_run_s3_keys.is_empty()
        && args.input_manifest_file.is_none()
        && args.input_manifest_s3_key.is_none()
    {
        return Err(AppError::config(
            "--build-shadow-cycle-decision requires a shadow validation run file, shadow validation S3 key, or manifest with shadow_validation_run_refs",
        ));
    }
    Ok(())
}

pub(in crate::cli) fn validate_shadow_cycle_from_latest_state_args(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state uses --output-s3-bucket, not --output-dir",
        ));
    }
    if args.shadow_cycle_decision_output_file.is_some() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state uses --output-s3-bucket, not --shadow-cycle-decision-output-file",
        ));
    }
    if args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state requires --output-s3-bucket",
        ));
    }
    if args.input_manifest_file.is_some() || args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state discovers shadow inputs from S3; do not pass manifest input",
        ));
    }
    if !args.shadow_validation_run_files.is_empty()
        || !args.shadow_validation_run_s3_keys.is_empty()
    {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state discovers shadow inputs from S3; do not pass explicit shadow validation inputs",
        ));
    }
    Ok(())
}
