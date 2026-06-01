def scheduler_action($verdict; $focused_research_manifest_file; $accumulation_blocked_reason):
  if $verdict == "DISCOVER_LATEST_MARKET_L1_AS_OF" then "DISCOVER_MARKET_L1_WATERMARK"
  elif $verdict == "WAIT_FOR_TARGET_HOLDING_WINDOW" then "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES"
  elif $verdict == "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION" then "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES"
  elif $verdict == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" and $focused_research_manifest_file != null then "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH"
  elif $verdict == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" and $accumulation_blocked_reason == "missing_retest_horizon_status_file" then "HOLD_FOR_OPERATOR_REVIEW"
  elif $verdict == "REVIEW_SHADOW_COMPLETION_EVIDENCE" then "REVIEW_SHADOW_COMPLETION_EVIDENCE"
  elif $verdict == "NO_SHADOW_SAMPLE_GAP_DETECTED" then "NOOP"
  elif $verdict == "NO_SHADOW_CANDIDATES" then "NOOP"
  else "HOLD_FOR_OPERATOR_REVIEW" end;

def wait_action($action):
  ($action == "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES"
   or $action == "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES");

($cycle[0] // {}) as $summary
| ($summary.next_decision.verdict // "UNKNOWN") as $verdict
| scheduler_action($verdict; ($summary.accumulation_manifest_file // null); ($summary.accumulation_blocked_reason // null)) as $action
| ($summary.next_decision.next_observation_not_before_ms // null) as $not_before_ms
| {
    schema_version:"research_shadow_cycle_decision_v1",
    generated_at:$generated_at,
    decision_id:(
      "shadow_cycle_decision:"
      + (($summary.run_dir // "unknown") | split("/") | last)
      + ":"
      + $verdict
      + ":"
      + (($not_before_ms // $summary.latest_l1_as_of_ms // $summary.generated_at // $generated_at) | tostring)
    ),
    source_cycle_summary_file:($summary.cycle_summary_file // null),
    run_dir:($summary.run_dir // null),
    scheduler_action:$action,
    source_verdict:$verdict,
    run_not_before_ms:(if wait_action($action) then $not_before_ms else null end),
    run_not_before_at:(if wait_action($action) then ($summary.next_decision.next_observation_not_before_at // null) else null end),
    run_not_before_source:(if wait_action($action) then ($summary.next_decision.next_observation_not_before_source // null) else null end),
    focused_research_manifest_file:(
      if $action == "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH" then $summary.accumulation_manifest_file
      else null
      end
    ),
    focused_research_summary_file:(
      if $action == "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH" then $summary.accumulation_summary_file
      else null
      end
    ),
    latest_l1_as_of_ms:($summary.latest_l1_as_of_ms // null),
    shadow_sample_state:{
      shadow_validation_count:($summary.observation_summary.shadow_validation_count // 0),
      target_window_materialized_count:($summary.observation_summary.target_window_materialized_count // 0),
      candidate_lifecycle_count:($summary.gap_summary.candidate_lifecycle_count // 0),
      partially_materialized_candidate_count:($summary.gap_summary.partially_materialized_candidate_count // 0),
      pending_target_window_candidate_count:($summary.gap_summary.pending_target_window_candidate_count // 0),
      total_sample_deficit:($summary.gap_summary.total_sample_deficit // 0),
      symbols:($summary.gap_summary.symbols // [])
    },
    safe_next_actions:($summary.next_decision.safe_next_actions // []),
    blocked_actions:($summary.next_decision.blocked_actions // []),
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_decision_only:true,
      shadow_status_mutated:false,
      paper_live_enabled:false,
      live_enabled:false,
      order_execution_enabled:false
    }
  }
