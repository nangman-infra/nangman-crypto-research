use super::checkpoint::RetestRefreshCycleCheckpoint;
use super::focused::RetestRefreshCycleFocusedDispatch;
use super::*;

pub(super) fn retest_refresh_cycle_summary(
    checkpoint: RetestRefreshCycleCheckpoint,
    focused: RetestRefreshCycleFocusedDispatch,
) -> RunSummary {
    RunSummary {
        retest_horizon_plans_created: 1,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(focused.scheduler_action),
        retest_cycle_run_not_before_ms: checkpoint.validation.run_not_before_ms,
        focused_retest_manifests_created: focused.manifests_created,
        focused_retest_horizon_count: focused.horizon_count,
        focused_retest_candidate_bundle_refs: focused.candidate_bundle_refs,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        paper_watch_observer_iterations: 0,
        paper_watch_observer_snapshots_created: 0,
        paper_watch_observer_active_candidates: 0,
        paper_watch_observer_restored_live_marks: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files: checkpoint.output_files,
    }
}
