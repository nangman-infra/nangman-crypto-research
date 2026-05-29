use super::*;

mod conflicts;
mod modes;
mod research_io;

pub(super) fn validate_args(args: &Args) -> AppResult<()> {
    conflicts::validate_mode_conflicts(args)?;
    if modes::validate_paper_watch_mode(args)? {
        return Ok(());
    }
    validate_retest_horizon_plan_input_args(args)?;
    validate_retest_horizon_status_input_args(args)?;
    validate_research_report_input_args(args)?;
    if modes::validate_retest_or_shadow_mode(args)? {
        return Ok(());
    }
    research_io::validate_default_research_io_args(args)
}
