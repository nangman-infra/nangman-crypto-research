use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn has_retest_refresh_individual_output(args: &Args) -> bool {
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

pub(super) fn validate_retest_refresh_output_target(args: &Args) -> AppResult<()> {
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
