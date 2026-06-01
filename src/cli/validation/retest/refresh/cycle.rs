use super::manifest::validate_retest_refresh_manifest_input;
use super::output::{has_retest_refresh_individual_output, validate_retest_refresh_output_target};
use super::shared::{
    reject_retest_horizon_inputs, reject_s3_prefix, require_focused_retest_next_actions,
};
use crate::cli::Args;
use crate::cli::validation::has_research_report_input;
use crate::error::{AppError, AppResult};

pub(in crate::cli) fn validate_retest_refresh_cycle_args(args: &Args) -> AppResult<()> {
    validate_retest_refresh_manifest_input(args)?;
    require_research_report_input(args)?;
    reject_retest_horizon_inputs(
        args,
        "--run-retest-refresh-cycle creates fresh plan/status; do not pass retest horizon plan/status inputs",
    )?;
    if has_retest_refresh_individual_output(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle uses --output-dir or --output-s3-bucket, not individual retest/focus output files",
        ));
    }
    validate_retest_refresh_output_target(args)?;
    reject_s3_prefix(
        args,
        "--run-retest-refresh-cycle writes multiple artifact families; do not pass --output-s3-prefix",
    )?;
    require_focused_retest_next_actions(args)
}

fn require_research_report_input(args: &Args) -> AppResult<()> {
    if has_research_report_input(args) {
        Ok(())
    } else {
        Err(AppError::config(
            "--run-retest-refresh-cycle requires --research-report-file or S3 report input",
        ))
    }
}
