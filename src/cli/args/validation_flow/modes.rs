use super::*;

pub(super) fn validate_paper_watch_mode(args: &Args) -> AppResult<bool> {
    if args.run_paper_watch_live_cycle {
        validate_paper_watch_live_cycle_args(args)?;
        return Ok(true);
    }
    if args.run_paper_watch_observer {
        validate_paper_watch_observer_args(args)?;
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn validate_retest_or_shadow_mode(args: &Args) -> AppResult<bool> {
    if args.build_retest_horizon_plan {
        validate_retest_horizon_plan_build_args(args)?;
        return Ok(true);
    }
    if args.run_retest_refresh_cycle {
        validate_retest_refresh_cycle_args(args)?;
        return Ok(true);
    }
    if args.run_retest_refresh_cycle_from_latest_state {
        validate_retest_refresh_cycle_from_latest_state_args(args)?;
        return Ok(true);
    }
    if args.build_retest_horizon_status {
        validate_retest_horizon_status_build_args(args)?;
        return Ok(true);
    }
    if args.run_retest_cycle_scheduler {
        validate_retest_cycle_scheduler_args(args)?;
        return Ok(true);
    }
    if args.build_focused_retest_manifest {
        validate_focused_retest_manifest_build_args(args)?;
        return Ok(true);
    }
    if args.run_shadow_cycle_from_latest_state {
        validate_shadow_cycle_from_latest_state_args(args)?;
        return Ok(true);
    }
    if has_retest_horizon_status_input(args) {
        return Ok(true);
    }
    if args.shadow_cycle_decision_file.is_some() {
        return Ok(true);
    }
    if args.build_shadow_cycle_decision {
        validate_shadow_cycle_build_args(args)?;
        return Ok(true);
    }
    Ok(false)
}
