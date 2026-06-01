pub(in super::super) fn wait_without_not_before_json() -> &'static str {
    r#"{
  "schema_version": "research_shadow_cycle_decision_v1",
  "generated_at": "2026-05-24T12:16:00Z",
  "decision_id": "shadow_cycle_decision:run:wait:none",
  "scheduler_action": "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES",
  "source_verdict": "WAIT_FOR_TARGET_HOLDING_WINDOW",
  "shadow_sample_state": {
    "shadow_validation_count": 1,
    "target_window_materialized_count": 0,
    "candidate_lifecycle_count": 1,
    "partially_materialized_candidate_count": 0,
    "pending_target_window_candidate_count": 1,
    "total_sample_deficit": 30,
    "symbols": ["BTC"]
  },
  "blocked_actions": [
    "do_not_create_paper_without_completed_passed_shadow",
    "do_not_enable_live_from_shadow_sample_gap_manifest"
  ],
  "safety": {
    "s3_write": false,
    "ecs_task_started": false,
    "dispatcher_mode_changed": false,
    "local_decision_only": true,
    "shadow_status_mutated": false,
    "paper_live_enabled": false,
    "live_enabled": false,
    "order_execution_enabled": false
  }
}"#
}

pub(in super::super) fn order_execution_enabled_json() -> &'static str {
    r#"{
  "schema_version": "research_shadow_cycle_decision_v1",
  "generated_at": "2026-05-24T12:16:00Z",
  "decision_id": "shadow_cycle_decision:run:unsafe",
  "scheduler_action": "NOOP",
  "source_verdict": "NO_SHADOW_SAMPLE_GAP_DETECTED",
  "shadow_sample_state": {
    "shadow_validation_count": 0,
    "target_window_materialized_count": 0,
    "candidate_lifecycle_count": 0,
    "partially_materialized_candidate_count": 0,
    "pending_target_window_candidate_count": 0,
    "total_sample_deficit": 0,
    "symbols": []
  },
  "blocked_actions": [
    "do_not_create_paper_without_completed_passed_shadow",
    "do_not_enable_live_from_shadow_sample_gap_manifest"
  ],
  "safety": {
    "s3_write": false,
    "ecs_task_started": false,
    "dispatcher_mode_changed": false,
    "local_decision_only": true,
    "shadow_status_mutated": false,
    "paper_live_enabled": false,
    "live_enabled": false,
    "order_execution_enabled": true
  }
}"#
}
