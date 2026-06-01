pub(in super::super) fn wait_decision_json() -> &'static str {
    r#"{
  "schema_version": "research_shadow_cycle_decision_v1",
  "generated_at": "2026-05-24T12:16:00Z",
  "decision_id": "shadow_cycle_decision:run:WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION:1779670979756",
  "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
  "run_dir": "/tmp/run",
  "scheduler_action": "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES",
  "source_verdict": "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION",
  "run_not_before_ms": 1779670979756,
  "run_not_before_at": "2026-05-25T01:02:59Z",
  "run_not_before_source": "pending_shadow_target_exit_deadline_ms",
  "focused_research_manifest_file": null,
  "focused_research_summary_file": null,
  "latest_l1_as_of_ms": null,
  "shadow_sample_state": {
    "shadow_validation_count": 24,
    "target_window_materialized_count": 12,
    "candidate_lifecycle_count": 6,
    "partially_materialized_candidate_count": 6,
    "pending_target_window_candidate_count": 6,
    "total_sample_deficit": 168,
    "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
  },
  "safe_next_actions": ["wait_for_pending_shadow_target_window_materialization"],
  "blocked_actions": [
    "do_not_mark_pending_shadow_passed_from_sample_counts_only",
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

pub(in super::super) fn focused_accumulation_decision_json() -> &'static str {
    r#"{
  "schema_version": "research_shadow_cycle_decision_v1",
  "generated_at": "2026-05-24T12:16:00Z",
  "decision_id": "shadow_cycle_decision:run:ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION:1779700000000",
  "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
  "run_dir": "/tmp/run",
  "scheduler_action": "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH",
  "source_verdict": "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
  "run_not_before_ms": null,
  "run_not_before_at": null,
  "run_not_before_source": null,
  "focused_research_manifest_file": "/tmp/run/shadow-accumulation-input-manifest.next.json",
  "focused_research_summary_file": "/tmp/run/shadow-accumulation-input-manifest.next.summary.json",
  "latest_l1_as_of_ms": 1779700000000,
  "shadow_sample_state": {
    "shadow_validation_count": 24,
    "target_window_materialized_count": 24,
    "candidate_lifecycle_count": 6,
    "partially_materialized_candidate_count": 0,
    "pending_target_window_candidate_count": 0,
    "total_sample_deficit": 156,
    "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
  },
  "safe_next_actions": ["accumulate_shadow_observation_samples"],
  "blocked_actions": [
    "do_not_mark_pending_shadow_passed_from_sample_counts_only",
    "do_not_create_paper_without_completed_passed_shadow",
    "do_not_enable_live_from_shadow_accumulation_manifest",
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
