use crate::cli::Args;
use crate::cli::validation::{has_research_report_input, has_retest_horizon_plan_input};
use crate::error::{AppError, AppResult};

use super::common::{validate_manifest_input_source, validate_output_target};

pub(in crate::cli) fn validate_retest_horizon_plan_build_args(args: &Args) -> AppResult<()> {
    validate_manifest_input_source(
        args,
        "--build-retest-horizon-plan requires --input-manifest-file or S3 manifest input",
    )?;
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
    validate_output_target(
        args.retest_horizon_plan_output_file.is_some(),
        args.output_s3_bucket.is_some(),
        args.output_dir.is_some(),
        "use either --retest-horizon-plan-output-file or --output-s3-bucket, not both",
        "--build-retest-horizon-plan uses --retest-horizon-plan-output-file or --output-s3-bucket, not --output-dir",
        "--build-retest-horizon-plan requires --retest-horizon-plan-output-file or --output-s3-bucket",
    )
}
