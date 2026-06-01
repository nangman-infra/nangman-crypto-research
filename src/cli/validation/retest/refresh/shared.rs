use crate::cli::Args;
use crate::cli::validation::{has_retest_horizon_plan_input, has_retest_horizon_status_input};
use crate::error::{AppError, AppResult};

pub(super) fn reject_retest_horizon_inputs(args: &Args, message: &'static str) -> AppResult<()> {
    if has_retest_horizon_plan_input(args) || has_retest_horizon_status_input(args) {
        Err(AppError::config(message))
    } else {
        Ok(())
    }
}

pub(super) fn reject_s3_prefix(args: &Args, message: &'static str) -> AppResult<()> {
    if args.output_s3_prefix.is_some() {
        Err(AppError::config(message))
    } else {
        Ok(())
    }
}

pub(super) fn require_focused_retest_next_actions(args: &Args) -> AppResult<()> {
    if args.focused_retest_next_actions.is_empty() {
        Err(AppError::config(
            "focused retest next action list must not be empty",
        ))
    } else {
        Ok(())
    }
}
