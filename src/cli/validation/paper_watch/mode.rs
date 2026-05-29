use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn validate_paper_watch_live_cycle_mode_is_isolated(args: &Args) -> AppResult<()> {
    if has_research_retest_shadow_mode(args) || args.run_paper_watch_observer {
        return Err(AppError::config(
            "use --run-paper-watch-live-cycle separately from research/retest/shadow modes",
        ));
    }
    Ok(())
}

pub(super) fn validate_paper_watch_observer_mode_is_isolated(args: &Args) -> AppResult<()> {
    if has_research_retest_shadow_mode(args) || args.run_paper_watch_live_cycle {
        return Err(AppError::config(
            "use --run-paper-watch-observer separately from research/retest/shadow modes",
        ));
    }
    Ok(())
}

fn has_research_retest_shadow_mode(args: &Args) -> bool {
    [
        args.build_shadow_cycle_decision,
        args.run_shadow_cycle_from_latest_state,
        args.build_retest_horizon_plan,
        args.run_retest_refresh_cycle,
        args.run_retest_refresh_cycle_from_latest_state,
        args.run_retest_cycle_scheduler,
        args.build_retest_horizon_status,
        args.build_focused_retest_manifest,
    ]
    .into_iter()
    .any(|enabled| enabled)
}
