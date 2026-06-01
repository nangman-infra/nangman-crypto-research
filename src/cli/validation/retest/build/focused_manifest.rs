use crate::cli::Args;
use crate::cli::validation::has_retest_horizon_status_input;
use crate::error::{AppError, AppResult};

use super::common::{validate_manifest_input_source, validate_output_target};

pub(in crate::cli) fn validate_focused_retest_manifest_build_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires a retest horizon status input",
        ));
    }
    validate_manifest_input_source(
        args,
        "--build-focused-retest-manifest requires --input-manifest-file or S3 manifest input",
    )?;
    validate_output_target(
        args.focused_retest_manifest_output_file.is_some(),
        args.output_s3_bucket.is_some(),
        args.output_dir.is_some(),
        "use either --focused-retest-manifest-output-file or --output-s3-bucket, not both",
        "--build-focused-retest-manifest uses --focused-retest-manifest-output-file or --output-s3-bucket, not --output-dir",
        "--build-focused-retest-manifest requires --focused-retest-manifest-output-file or --output-s3-bucket",
    )?;
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}
