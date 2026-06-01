use super::super::types::FocusedRetestBuildOptions;
use crate::error::{AppError, AppResult};

pub(super) fn validate_options(options: &FocusedRetestBuildOptions) -> AppResult<()> {
    if options.next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    if options.research_packet_id.trim().is_empty() {
        return Err(AppError::config(
            "focused retest research_packet_id must not be empty",
        ));
    }
    if options.run_scope.trim().is_empty() {
        return Err(AppError::config(
            "focused retest run_scope must not be empty",
        ));
    }
    Ok(())
}
