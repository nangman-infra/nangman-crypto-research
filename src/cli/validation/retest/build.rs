use crate::cli::Args;
use crate::cli::validation::{
    has_research_report_input, has_retest_horizon_plan_input, has_retest_horizon_status_input,
};
use crate::error::{AppError, AppResult};

pub(in crate::cli) fn validate_retest_horizon_plan_build_args(args: &Args) -> AppResult<()> {
    if args.input_manifest_file.is_some()
        && (args.input_manifest_s3_bucket.is_some() || args.input_manifest_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        ));
    }
    if args.input_manifest_s3_bucket.is_some() != args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        ));
    }
    if args.input_manifest_file.is_none() && args.input_manifest_s3_key.is_none() {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --input-manifest-file or S3 manifest input",
        ));
    }
    if !has_research_report_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --research-report-file or S3 report input",
        ));
    }
    if has_retest_horizon_plan_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-plan creates a plan; do not also pass retest horizon plan input",
        ));
    }
    if args.output_s3_bucket.is_some() && args.retest_horizon_plan_output_file.is_some() {
        return Err(AppError::config(
            "use either --retest-horizon-plan-output-file or --output-s3-bucket, not both",
        ));
    }
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--build-retest-horizon-plan uses --retest-horizon-plan-output-file or --output-s3-bucket, not --output-dir",
        ));
    }
    if args.retest_horizon_plan_output_file.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --retest-horizon-plan-output-file or --output-s3-bucket",
        ));
    }
    Ok(())
}

pub(in crate::cli) fn validate_focused_retest_manifest_build_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires a retest horizon status input",
        ));
    }
    if args.input_manifest_file.is_some()
        && (args.input_manifest_s3_bucket.is_some() || args.input_manifest_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        ));
    }
    if args.input_manifest_s3_bucket.is_some() != args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        ));
    }
    if args.input_manifest_file.is_none() && args.input_manifest_s3_key.is_none() {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires --input-manifest-file or S3 manifest input",
        ));
    }
    if args.output_s3_bucket.is_some() && args.focused_retest_manifest_output_file.is_some() {
        return Err(AppError::config(
            "use either --focused-retest-manifest-output-file or --output-s3-bucket, not both",
        ));
    }
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--build-focused-retest-manifest uses --focused-retest-manifest-output-file or --output-s3-bucket, not --output-dir",
        ));
    }
    if args.focused_retest_manifest_output_file.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires --focused-retest-manifest-output-file or --output-s3-bucket",
        ));
    }
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}

pub(in crate::cli) fn validate_retest_horizon_status_build_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_plan_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-status requires a retest horizon plan input",
        ));
    }
    if has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-status creates a status; do not also pass retest horizon status input",
        ));
    }
    if args.output_s3_bucket.is_some() && args.retest_horizon_status_output_file.is_some() {
        return Err(AppError::config(
            "use either --retest-horizon-status-output-file or --output-s3-bucket, not both",
        ));
    }
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--build-retest-horizon-status uses --retest-horizon-status-output-file or --output-s3-bucket, not --output-dir",
        ));
    }
    if args.retest_horizon_status_output_file.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--build-retest-horizon-status requires --retest-horizon-status-output-file or --output-s3-bucket",
        ));
    }
    Ok(())
}

pub(in crate::cli) fn validate_retest_cycle_scheduler_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--run-retest-cycle-scheduler requires a retest horizon status input",
        ));
    }
    validate_focused_retest_manifest_build_args(args).map_err(|error| {
        AppError::config(format!(
            "--run-retest-cycle-scheduler uses focused retest manifest inputs when execution is due: {error}"
        ))
    })
}
