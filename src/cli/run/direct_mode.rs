use super::*;

pub(super) async fn run_direct_mode(args: &Args) -> AppResult<Option<RunSummary>> {
    if args.run_paper_watch_live_cycle {
        return run_paper_watch_live_cycle_mode(args).await.map(Some);
    }
    if args.run_paper_watch_observer {
        return run_paper_watch_observer_mode(args).await.map(Some);
    }
    if args.run_retest_refresh_cycle {
        return run_retest_refresh_cycle_mode(args).await.map(Some);
    }
    if args.run_retest_refresh_cycle_from_latest_state {
        return run_retest_refresh_cycle_from_latest_state_mode(args)
            .await
            .map(Some);
    }
    if args.run_shadow_cycle_from_latest_state {
        return run_shadow_cycle_from_latest_state_mode(args)
            .await
            .map(Some);
    }
    if args.build_retest_horizon_plan {
        return build_retest_horizon_plan_mode(args).await.map(Some);
    }
    if args.build_retest_horizon_status {
        return build_retest_horizon_status_mode(args).await.map(Some);
    }
    if args.run_retest_cycle_scheduler {
        return run_retest_cycle_scheduler_mode(args).await.map(Some);
    }
    if args.build_focused_retest_manifest {
        return build_focused_retest_manifest_mode(args).await.map(Some);
    }
    Ok(None)
}
