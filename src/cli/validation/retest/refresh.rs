use crate::cli::Args;
use crate::cli::validation::{
    has_research_report_input, has_retest_horizon_plan_input, has_retest_horizon_status_input,
};
use crate::error::{AppError, AppResult};

pub(in crate::cli) fn validate_retest_refresh_cycle_args(args: &Args) -> AppResult<()> {
    validate_retest_refresh_manifest_input(args)?;
    if !has_research_report_input(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle requires --research-report-file or S3 report input",
        ));
    }
    if has_retest_horizon_plan_input(args) || has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle creates fresh plan/status; do not pass retest horizon plan/status inputs",
        ));
    }
    if has_retest_refresh_individual_output(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle uses --output-dir or --output-s3-bucket, not individual retest/focus output files",
        ));
    }
    validate_retest_refresh_output_target(args)?;
    if args.output_s3_prefix.is_some() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle writes multiple artifact families; do not pass --output-s3-prefix",
        ));
    }
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}

pub(in crate::cli) fn validate_retest_refresh_cycle_from_latest_state_args(
    args: &Args,
) -> AppResult<()> {
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state uses --output-s3-bucket, not --output-dir",
        ));
    }
    if args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --output-s3-bucket",
        ));
    }
    if args.market_l1_s3_bucket.is_none() && args.retest_horizon_latest_l1_as_of_ms.is_none() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --market-l1-s3-bucket or --retest-horizon-latest-l1-as-of-ms",
        ));
    }
    if args.input_manifest_file.is_some()
        || args.input_manifest_s3_bucket.is_some()
        || args.input_manifest_s3_key.is_some()
        || args.research_report_file.is_some()
        || args.research_report_s3_bucket.is_some()
        || args.research_report_s3_key.is_some()
    {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state discovers manifest/report inputs from retest-cycle-source-state; do not pass manifest/report inputs",
        ));
    }
    if has_retest_horizon_plan_input(args) || has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state creates fresh plan/status; do not pass retest horizon plan/status inputs",
        ));
    }
    if has_retest_refresh_individual_output(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state uses --output-s3-bucket, not individual retest/focus output files",
        ));
    }
    if args.output_s3_prefix.is_some() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state writes multiple artifact families; do not pass --output-s3-prefix",
        ));
    }
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}

pub(in crate::cli) fn validate_retest_refresh_manifest_input(args: &Args) -> AppResult<()> {
    match (
        args.input_manifest_file.is_some(),
        args.input_manifest_s3_bucket.is_some(),
        args.input_manifest_s3_key.is_some(),
    ) {
        (true, false, false) | (false, true, true) => Ok(()),
        (true, _, _) => Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        )),
        (false, true, false) | (false, false, true) => Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        )),
        (false, false, false) => Err(AppError::config(
            "--run-retest-refresh-cycle requires --input-manifest-file or S3 manifest input",
        )),
    }
}

pub(in crate::cli) fn has_retest_refresh_individual_output(args: &Args) -> bool {
    [
        args.retest_horizon_plan_output_file.is_some(),
        args.retest_horizon_status_output_file.is_some(),
        args.focused_retest_manifest_output_file.is_some(),
        args.focused_retest_summary_output_file.is_some(),
        args.retest_driver_summary_file.is_some(),
    ]
    .into_iter()
    .any(|present| present)
}

pub(in crate::cli) fn validate_retest_refresh_output_target(args: &Args) -> AppResult<()> {
    match (args.output_dir.is_some(), args.output_s3_bucket.is_some()) {
        (true, false) | (false, true) => Ok(()),
        (true, true) => Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        )),
        (false, false) => Err(AppError::config(
            "--run-retest-refresh-cycle requires --output-dir or --output-s3-bucket",
        )),
    }
}
