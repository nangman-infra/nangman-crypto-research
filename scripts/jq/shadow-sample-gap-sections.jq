def unique_sorted: unique | sort;

def observation_not_before_at($next_observation_not_before_ms):
  if $next_observation_not_before_ms == null then null
  else (($next_observation_not_before_ms / 1000) | todateiso8601)
  end;

def shadow_sample_gap_summary(
  $candidates;
  $pending;
  $target_waiting;
  $partial_materialized;
  $pending_target_window;
  $sample_ready;
  $deficient;
  $next_observation_not_before_ms
):
  {
    candidate_lifecycle_count:($candidates | length),
    symbol_count:($candidates | map(.symbols // []) | flatten | unique | length),
    symbols:($candidates | map(.symbols // []) | flatten | unique_sorted),
    pending_candidate_count:($pending | length),
    target_window_waiting_candidate_count:($target_waiting | length),
    partially_materialized_candidate_count:($partial_materialized | length),
    pending_target_window_candidate_count:($pending_target_window | length),
    next_observation_not_before_ms:$next_observation_not_before_ms,
    next_observation_not_before_at:observation_not_before_at($next_observation_not_before_ms),
    sample_requirement_met_candidate_count:($sample_ready | length),
    deficient_candidate_count:($deficient | length),
    total_sample_deficit:(($deficient | map(.sample_deficit) | add) // 0),
    largest_sample_deficit:(($deficient | map(.sample_deficit) | max) // 0),
    minimum_required_shadow_sample_count:(($candidates | map(.required_shadow_sample_count) | min) // 0),
    maximum_required_shadow_sample_count:(($candidates | map(.required_shadow_sample_count) | max) // 0)
  };

def shadow_sample_gap_verdict($plan; $candidates; $target_waiting; $partial_materialized; $deficient; $pending):
  if ($candidates | length) == 0 then "NO_SHADOW_CANDIDATES"
  elif ($plan.latest_l1_as_of_ms // null) == null then "DISCOVER_LATEST_MARKET_L1_AS_OF"
  elif ($target_waiting | length) > 0 then "WAIT_FOR_TARGET_HOLDING_WINDOW"
  elif ($partial_materialized | length) > 0 then "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION"
  elif ($deficient | length) > 0 then "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION"
  elif ($pending | length) > 0 then "REVIEW_SHADOW_COMPLETION_EVIDENCE"
  else "NO_SHADOW_SAMPLE_GAP_DETECTED" end;

def shadow_sample_gap_safe_next_actions($plan; $target_waiting; $partial_materialized; $deficient; $pending; $sample_ready):
  [
    if ($plan.latest_l1_as_of_ms // null) == null then "discover_latest_market_l1_as_of" else empty end,
    if ($target_waiting | length) > 0 then "wait_for_target_holding_window_materialization" else empty end,
    if ($partial_materialized | length) > 0 then "wait_for_pending_shadow_target_window_materialization" else empty end,
    if (($deficient | length) > 0 and ($target_waiting | length) == 0 and ($partial_materialized | length) == 0) then "accumulate_shadow_observation_samples" else empty end,
    if ($pending | length) > 0 then "keep_shadow_status_pending_until_completion_evidence_exists" else empty end,
    if ($sample_ready | length) > 0 then "review_sample_ready_candidates_for_shadow_completion" else empty end
  ];

def shadow_sample_gap_next_decision(
  $plan;
  $candidates;
  $target_waiting;
  $partial_materialized;
  $deficient;
  $pending;
  $sample_ready;
  $next_observation_not_before_ms
):
  {
    verdict:shadow_sample_gap_verdict($plan; $candidates; $target_waiting; $partial_materialized; $deficient; $pending),
    safe_next_actions:shadow_sample_gap_safe_next_actions($plan; $target_waiting; $partial_materialized; $deficient; $pending; $sample_ready),
    next_observation_not_before_ms:$next_observation_not_before_ms,
    next_observation_not_before_at:observation_not_before_at($next_observation_not_before_ms),
    next_observation_not_before_source:(
      if $next_observation_not_before_ms == null then null
      else "pending_shadow_target_exit_deadline_ms"
      end
    ),
    blocked_actions:[
      "do_not_mark_pending_shadow_passed_from_sample_counts_only",
      "do_not_create_paper_without_completed_passed_shadow",
      "do_not_enable_live_from_shadow_sample_gap_manifest"
    ]
  };
