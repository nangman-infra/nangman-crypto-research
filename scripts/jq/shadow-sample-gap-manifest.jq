include "shadow-sample-gap-candidates";
include "shadow-sample-gap-sections";

. as $plan
| ($plan | shadow_sample_gap_candidates) as $candidates
| ($candidates | map(select(.sample_deficit > 0))) as $deficient
| ($candidates | map(select(.sample_requirement_met == true))) as $sample_ready
| ($candidates | map(select(.target_window_materialized_count == 0))) as $target_waiting
| (
    $candidates
    | map(select(.target_window_materialized_shadow_run_count > 0 and .target_window_materialized_shadow_run_count < .observed_shadow_run_count))
  ) as $partial_materialized
| (
    $candidates
    | map(select(.pending_target_window_shadow_run_count > 0))
  ) as $pending_target_window
| (($pending_target_window | map(.next_pending_target_exit_deadline_ms) | map(select(. != null)) | min) // null) as $next_observation_not_before_ms
| ($candidates | map(select(.pending_count > 0))) as $pending
| {
    schema_version:"research_shadow_sample_gap_manifest_v1",
    generated_at:$generated_at,
    generated_at_ms:$generated_at_ms,
    shadow_observation_plan_file:$observation_plan_file,
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_manifest_only:true,
      shadow_status_mutated:false,
      paper_live_enabled:false
    },
    source_state:{
      observation_plan_schema_version:($plan.schema_version // null),
      observation_plan_verdict:($plan.next_decision.verdict // null),
      latest_l1_as_of_ms:($plan.latest_l1_as_of_ms // null),
      latest_l1_source:($plan.latest_l1_source // null),
      shadow_validation_run_file:($plan.shadow_validation_run_file // null),
      retest_horizon_status_file:($plan.retest_horizon_status_file // null)
    },
    shadow_sample_gap_summary:shadow_sample_gap_summary(
      $candidates;
      $pending;
      $target_waiting;
      $partial_materialized;
      $pending_target_window;
      $sample_ready;
      $deficient;
      $next_observation_not_before_ms
    ),
    next_decision:shadow_sample_gap_next_decision(
      $plan;
      $candidates;
      $target_waiting;
      $partial_materialized;
      $deficient;
      $pending;
      $sample_ready;
      $next_observation_not_before_ms
    ),
    shadow_sample_backlog:(
      $deficient
      | sort_by(-.sample_deficit, .candidate_lifecycle_key)
    ),
    sample_ready_candidates:(
      $sample_ready
      | sort_by(.candidate_lifecycle_key)
    ),
    by_candidate_lifecycle_key:(
      $candidates
      | sort_by(-.sample_deficit, .candidate_lifecycle_key)
    )
  }
