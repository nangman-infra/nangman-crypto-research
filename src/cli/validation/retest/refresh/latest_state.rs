use super::output::has_retest_refresh_individual_output;
use super::shared::{
    reject_retest_horizon_inputs, reject_s3_prefix, require_focused_retest_next_actions,
};
use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(in crate::cli) fn validate_retest_refresh_cycle_from_latest_state_args(
    args: &Args,
) -> AppResult<()> {
    reject_local_output_dir(args)?;
    require_s3_output_bucket(args)?;
    require_market_l1_source(args)?;
    reject_explicit_manifest_report_inputs(args)?;
    reject_retest_horizon_inputs(
        args,
        "--run-retest-refresh-cycle-from-latest-state creates fresh plan/status; do not pass retest horizon plan/status inputs",
    )?;
    reject_individual_outputs(args)?;
    reject_s3_prefix(
        args,
        "--run-retest-refresh-cycle-from-latest-state writes multiple artifact families; do not pass --output-s3-prefix",
    )?;
    require_focused_retest_next_actions(args)
}

fn reject_local_output_dir(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() {
        Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state uses --output-s3-bucket, not --output-dir",
        ))
    } else {
        Ok(())
    }
}

fn require_s3_output_bucket(args: &Args) -> AppResult<()> {
    if args.output_s3_bucket.is_some() {
        Ok(())
    } else {
        Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --output-s3-bucket",
        ))
    }
}

fn require_market_l1_source(args: &Args) -> AppResult<()> {
    if args.market_l1_s3_bucket.is_some() || args.retest_horizon_latest_l1_as_of_ms.is_some() {
        Ok(())
    } else {
        Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --market-l1-s3-bucket or --retest-horizon-latest-l1-as-of-ms",
        ))
    }
}

fn reject_explicit_manifest_report_inputs(args: &Args) -> AppResult<()> {
    if args.input_manifest_file.is_some()
        || args.input_manifest_s3_bucket.is_some()
        || args.input_manifest_s3_key.is_some()
        || args.research_report_file.is_some()
        || args.research_report_s3_bucket.is_some()
        || args.research_report_s3_key.is_some()
    {
        Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state discovers manifest/report inputs from retest-cycle-source-state; do not pass manifest/report inputs",
        ))
    } else {
        Ok(())
    }
}

fn reject_individual_outputs(args: &Args) -> AppResult<()> {
    if has_retest_refresh_individual_output(args) {
        Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state uses --output-s3-bucket, not individual retest/focus output files",
        ))
    } else {
        Ok(())
    }
}
