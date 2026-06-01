include "summarize-shadow-validation-status-common";

def safety_section:
  {
    s3_write:false,
    ecs_task_started:false,
    dispatcher_mode_changed:false,
    local_summary_only:true,
    paper_live_enabled:false
  };

def upstream_state_section($horizon_status):
  {
    retest_horizon_verdict:($horizon_status.verdict // null),
    research_factory_blocking_stage:($horizon_status.research_factory_gap_summary.blocking_stage // null),
    selected_candidate_count:($horizon_status.batch_state.selected_candidate_count // null),
    replay_run_count:($horizon_status.batch_state.replay_run_count // null),
    promotion_passed:($horizon_status.stage_state.promotion_passed // null),
    paper_created:($horizon_status.stage_state.paper_created // null),
    live_enabled:false
  };

def stage_state_section($runs; $completed; $paper_eligible; $order_execution_violations; $paper_contract_mismatches; $horizon_status):
  {
    shadow_created:(($runs | length) > 0),
    shadow_completed:(($completed | length) > 0),
    shadow_passed:(($paper_eligible | length) > 0),
    paper_input_ready:(
      ($paper_eligible | length) > 0
      and ($order_execution_violations | length) == 0
      and ($paper_contract_mismatches | length) == 0
    ),
    paper_created:($horizon_status.stage_state.paper_created // false),
    live_enabled:false
  };

def shadow_validation_summary_section($runs; $pending; $completed; $failed; $paper_eligible; $paper_contract_mismatches; $order_execution_violations):
  {
    shadow_validation_count:($runs | length),
    candidate_lifecycle_count:($runs | map(.candidate_lifecycle_key // empty) | unique | length),
    symbol_count:($runs | map(.symbol_canonical // empty) | unique | length),
    symbols:($runs | map(.symbol_canonical // empty) | unique_sorted),
    schema_versions:($runs | map(.schema_version // "unknown") | unique_sorted),
    status_counts:($runs | counts_by(status_value)),
    passed_counts:($runs | counts_by((.passed // false))),
    pending_count:($pending | length),
    completed_count:($completed | length),
    failed_count:($failed | length),
    completed_passed_shadow_count:($paper_eligible | length),
    paper_contract_mismatch_count:($paper_contract_mismatches | length),
    no_order_execution_violation_count:($order_execution_violations | length)
  };

def paper_gate_section($pending; $failed; $paper_eligible; $paper_contract_mismatches; $order_execution_violations):
  {
    paper_generation_precondition_met:(
      ($paper_eligible | length) > 0
      and ($order_execution_violations | length) == 0
      and ($paper_contract_mismatches | length) == 0
    ),
    required_shadow_status:"completed",
    required_shadow_passed:true,
    required_paper_trade_candidate_contract_version:"paper_trade_candidate_v1",
    eligible_shadow_validation_run_ids:($paper_eligible | map(.shadow_validation_run_id) | unique_sorted),
    eligible_candidate_lifecycle_keys:($paper_eligible | map(.candidate_lifecycle_key) | unique_sorted),
    blocked_actions:[
      if ($paper_eligible | length) == 0 then "do_not_create_paper_without_completed_passed_shadow" else empty end,
      if ($pending | length) > 0 then "do_not_treat_pending_shadow_as_passed" else empty end,
      if ($paper_contract_mismatches | length) > 0 then "do_not_use_shadow_with_paper_contract_mismatch" else empty end,
      if ($order_execution_violations | length) > 0 then "do_not_use_shadow_with_order_execution_enabled" else empty end,
      "do_not_enable_live_from_shadow_review"
    ],
    safe_next_actions:[
      if ($pending | length) > 0 then "observe_pending_shadow_validation_runs_until_completed" else empty end,
      if ($failed | length) > 0 then "inspect_failed_shadow_validation_runs" else empty end,
      if ($paper_contract_mismatches | length) > 0 then "inspect_shadow_paper_contract_mismatches" else empty end,
      if ($paper_eligible | length) > 0 then "review_completed_passed_shadow_before_paper" else empty end
    ]
  };

def by_symbol_section($runs):
  $runs
  | sort_by(.symbol_canonical // "unknown", .candidate_lifecycle_key // "", .shadow_validation_run_id // "")
  | group_by(.symbol_canonical // "unknown")
  | map({
      symbol:.[0].symbol_canonical,
      shadow_validation_count:length,
      candidate_lifecycle_count:(map(.candidate_lifecycle_key // empty) | unique | length),
      status_counts:counts_by(status_value),
      pending_count:(map(select(status_value == "pending")) | length),
      completed_passed_shadow_count:(map(select(is_completed_passed_shadow)) | length),
      no_order_execution_violation_count:(map(select((.termination_policy.no_order_execution // false) != true)) | length)
    });

def by_candidate_lifecycle_key_section($runs):
  $runs
  | sort_by(.candidate_lifecycle_key // "", .symbol_canonical // "", .shadow_validation_run_id // "")
  | group_by(.candidate_lifecycle_key // "unknown")
  | map({
      candidate_lifecycle_key:.[0].candidate_lifecycle_key,
      symbols:(map(.symbol_canonical // empty) | unique_sorted),
      shadow_validation_count:length,
      status_counts:counts_by(status_value),
      pending_count:(map(select(status_value == "pending")) | length),
      completed_passed_shadow_count:(map(select(is_completed_passed_shadow)) | length),
      runs:(map(run_projection))
    });
