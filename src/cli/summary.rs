use crate::model::ShadowCycleSchedulerAction;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct RunSummary {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub retest_horizon_plans_created: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub retest_horizon_statuses_validated: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retest_cycle_scheduler_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retest_cycle_run_not_before_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub focused_retest_manifests_created: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub focused_retest_horizon_count: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub focused_retest_candidate_bundle_refs: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub shadow_cycle_decisions_validated: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub shadow_cycle_decisions_created: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_cycle_scheduler_action: Option<ShadowCycleSchedulerAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_cycle_run_not_before_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_cycle_focused_research_manifest_file: Option<String>,
    pub processed_bundles: usize,
    pub replay_runs_created: usize,
    pub historical_replay_runs_loaded: usize,
    pub oss_adapter_runs_loaded: usize,
    pub shadow_validation_runs_loaded: usize,
    pub shadow_validation_runs_created: usize,
    pub paper_trade_candidates_created: usize,
    pub paper_trade_runs_created: usize,
    pub paper_trade_summaries_created: usize,
    pub paper_trade_marks_created: usize,
    pub paper_watch_live_marks_created: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub paper_watch_observer_iterations: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub paper_watch_observer_snapshots_created: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub paper_watch_observer_active_candidates: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub paper_watch_observer_restored_live_marks: usize,
    pub portfolio_risk_reject_events_created: usize,
    pub portfolio_reduce_only_signals_created: usize,
    pub output_files: Vec<String>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}
