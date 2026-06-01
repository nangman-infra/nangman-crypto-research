use crate::model::{
    ShadowCycleDecision, ShadowCycleDecisionSafety, ShadowCycleSampleState,
    ShadowCycleSchedulerAction,
};

pub(crate) fn test_shadow_decision(
    scheduler_action: ShadowCycleSchedulerAction,
    unsafe_boundary: bool,
) -> ShadowCycleDecision {
    ShadowCycleDecision {
        schema_version: "research_shadow_cycle_decision_v1".to_owned(),
        generated_at: "2026-05-26T00:00:00Z".to_owned(),
        decision_id: "decision_test".to_owned(),
        source_cycle_summary_file: None,
        run_dir: None,
        scheduler_action,
        source_verdict: "WAIT_FOR_TARGET_HOLDING_WINDOW".to_owned(),
        run_not_before_ms: Some(1),
        run_not_before_at: Some("2026-05-26T01:00:00Z".to_owned()),
        run_not_before_source: Some("target_window".to_owned()),
        focused_research_manifest_file: None,
        focused_research_summary_file: None,
        latest_l1_as_of_ms: Some(1),
        shadow_sample_state: ShadowCycleSampleState {
            shadow_validation_count: 1,
            target_window_materialized_count: 0,
            candidate_lifecycle_count: 1,
            partially_materialized_candidate_count: 0,
            pending_target_window_candidate_count: 1,
            total_sample_deficit: 3,
            symbols: vec!["DOGE".to_owned()],
        },
        safe_next_actions: vec!["wait for target window".to_owned()],
        blocked_actions: vec!["paper is blocked".to_owned()],
        safety: ShadowCycleDecisionSafety {
            s3_write: false,
            ecs_task_started: false,
            dispatcher_mode_changed: false,
            local_decision_only: true,
            shadow_status_mutated: false,
            paper_live_enabled: false,
            live_enabled: unsafe_boundary,
            order_execution_enabled: unsafe_boundary,
        },
    }
}
