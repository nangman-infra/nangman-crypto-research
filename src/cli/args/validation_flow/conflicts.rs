use super::*;

pub(super) fn validate_mode_conflicts(args: &Args) -> AppResult<()> {
    validate_shadow_cycle_conflicts(args)?;
    validate_retest_mode_conflicts(args)?;
    validate_retest_status_shadow_conflict(args)
}

fn validate_shadow_cycle_conflicts(args: &Args) -> AppResult<()> {
    if args.shadow_cycle_decision_file.is_some()
        && (args.build_shadow_cycle_decision || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use shadow cycle decision validation separately from shadow cycle build modes",
        ));
    }
    Ok(())
}

fn validate_retest_mode_conflicts(args: &Args) -> AppResult<()> {
    if args.run_retest_cycle_scheduler && args.build_focused_retest_manifest {
        return Err(AppError::config(
            "use either --run-retest-cycle-scheduler or --build-focused-retest-manifest, not both",
        ));
    }
    if args.build_retest_horizon_plan
        && (args.build_retest_horizon_status
            || args.run_retest_refresh_cycle
            || args.run_retest_refresh_cycle_from_latest_state
            || args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --build-retest-horizon-plan separately from retest status, scheduler, or focused manifest modes",
        ));
    }
    if args.run_retest_refresh_cycle
        && (args.build_retest_horizon_status
            || args.run_retest_refresh_cycle_from_latest_state
            || args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --run-retest-refresh-cycle separately from retest status, scheduler, or focused manifest modes",
        ));
    }
    if args.run_retest_refresh_cycle_from_latest_state
        && (args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --run-retest-refresh-cycle-from-latest-state separately from retest scheduler or focused manifest modes",
        ));
    }
    if args.build_retest_horizon_status
        && (args.run_retest_refresh_cycle_from_latest_state
            || args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --build-retest-horizon-status separately from retest scheduler or focused manifest modes",
        ));
    }
    Ok(())
}

fn validate_retest_status_shadow_conflict(args: &Args) -> AppResult<()> {
    if has_retest_horizon_status_input(args)
        && (args.shadow_cycle_decision_file.is_some()
            || args.build_shadow_cycle_decision
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use retest horizon status inputs separately from shadow cycle decision modes",
        ));
    }
    Ok(())
}
