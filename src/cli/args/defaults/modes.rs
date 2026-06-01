use super::*;

pub(super) fn apply_mode_env(args: &mut Args) {
    args.build_shadow_cycle_decision = env_bool("RESEARCH_BUILD_SHADOW_CYCLE_DECISION");
    args.run_shadow_cycle_from_latest_state =
        env_bool("RESEARCH_RUN_SHADOW_CYCLE_FROM_LATEST_STATE");
    args.build_retest_horizon_plan = env_bool("RESEARCH_BUILD_RETEST_HORIZON_PLAN");
    args.run_retest_refresh_cycle = env_bool("RESEARCH_RUN_RETEST_REFRESH_CYCLE");
    args.run_retest_refresh_cycle_from_latest_state =
        env_bool("RESEARCH_RUN_RETEST_REFRESH_CYCLE_FROM_LATEST_STATE");
    args.run_retest_cycle_scheduler = env_bool("RESEARCH_RUN_RETEST_CYCLE_SCHEDULER");
    args.build_retest_horizon_status = env_bool("RESEARCH_BUILD_RETEST_HORIZON_STATUS");
    args.build_focused_retest_manifest = env_bool("RESEARCH_BUILD_FOCUSED_RETEST_MANIFEST");
    args.run_paper_watch_live_cycle = env_bool("RESEARCH_RUN_PAPER_WATCH_LIVE_CYCLE");
    args.run_paper_watch_observer = env_bool("RESEARCH_RUN_PAPER_WATCH_OBSERVER");
}
