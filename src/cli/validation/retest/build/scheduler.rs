use crate::cli::Args;
use crate::cli::validation::has_retest_horizon_status_input;
use crate::error::{AppError, AppResult};

use super::focused_manifest::validate_focused_retest_manifest_build_args;

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
