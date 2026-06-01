use crate::cli::Args;
use crate::cli::validation::{has_retest_horizon_plan_input, has_retest_horizon_status_input};
use crate::error::{AppError, AppResult};

use super::common::validate_output_target;

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
    validate_output_target(
        args.retest_horizon_status_output_file.is_some(),
        args.output_s3_bucket.is_some(),
        args.output_dir.is_some(),
        "use either --retest-horizon-status-output-file or --output-s3-bucket, not both",
        "--build-retest-horizon-status uses --retest-horizon-status-output-file or --output-s3-bucket, not --output-dir",
        "--build-retest-horizon-status requires --retest-horizon-status-output-file or --output-s3-bucket",
    )
}
