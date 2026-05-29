use super::*;

pub(super) fn apply_mode_flag(args: &mut Args, arg: &str) -> bool {
    match arg {
        "--build-shadow-cycle-decision" => {
            args.build_shadow_cycle_decision = true;
            true
        }
        "--run-shadow-cycle-from-latest-state" => {
            args.run_shadow_cycle_from_latest_state = true;
            true
        }
        "--build-retest-horizon-plan" => {
            args.build_retest_horizon_plan = true;
            true
        }
        "--run-retest-refresh-cycle" => {
            args.run_retest_refresh_cycle = true;
            true
        }
        "--run-retest-refresh-cycle-from-latest-state" => {
            args.run_retest_refresh_cycle_from_latest_state = true;
            true
        }
        "--run-retest-cycle-scheduler" => {
            args.run_retest_cycle_scheduler = true;
            true
        }
        "--build-retest-horizon-status" => {
            args.build_retest_horizon_status = true;
            true
        }
        "--build-focused-retest-manifest" => {
            args.build_focused_retest_manifest = true;
            true
        }
        "--run-paper-watch-live-cycle" => {
            args.run_paper_watch_live_cycle = true;
            true
        }
        "--run-paper-watch-observer" => {
            args.run_paper_watch_observer = true;
            true
        }
        _ => false,
    }
}
