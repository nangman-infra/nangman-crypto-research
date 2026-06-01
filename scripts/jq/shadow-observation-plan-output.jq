include "shadow-observation-plan-calculations";

records as $runs
| ($horizon_status_input[0] // null) as $horizon_status
| latest_l1 as $latest_l1
| ($runs | map(select(status_value == "pending"))) as $pending
| ($runs | map(select(target_window_materialized))) as $target_materialized
| ($runs | map(select(absolute_window_materialized))) as $absolute_materialized
| (
    $runs
    | sort_by(.candidate_lifecycle_key // "", .symbol_canonical // "", .shadow_validation_run_id // "")
    | group_by(.candidate_lifecycle_key // "unknown")
    | map(. as $candidate_runs | sample_status($candidate_runs) as $sample | {
        candidate_lifecycle_key:.[0].candidate_lifecycle_key,
        symbols:(map(.symbol_canonical // empty) | unique_sorted),
        status_counts:counts_by(status_value),
        target_window_materialized_count:(map(select(target_window_materialized)) | length),
        absolute_window_materialized_count:(map(select(absolute_window_materialized)) | length),
        observation_sample_status:$sample,
        runs:(map(run_projection))
      })
  ) as $by_candidate
| (
    $runs
    | sort_by(.symbol_canonical // "unknown", .candidate_lifecycle_key // "", .shadow_validation_run_id // "")
    | group_by(.symbol_canonical // "unknown")
    | map(. as $symbol_runs | sample_status($symbol_runs) as $sample | {
        symbol:.[0].symbol_canonical,
        candidate_lifecycle_count:(map(.candidate_lifecycle_key // empty) | unique | length),
        shadow_validation_count:length,
        status_counts:counts_by(status_value),
        target_window_materialized_count:(map(select(target_window_materialized)) | length),
        absolute_window_materialized_count:(map(select(absolute_window_materialized)) | length),
        observation_sample_status:$sample
      })
  ) as $by_symbol
| ($by_candidate | map(select(.observation_sample_status.sample_requirement_met == true))) as $sample_ready_candidates
| ($by_candidate | map(select(.target_window_materialized_count > 0))) as $target_ready_candidates
| {
    schema_version:"research_shadow_observation_plan_v1",
    generated_at:$generated_at,
    generated_at_ms:$generated_at_ms,
    shadow_validation_run_file:$shadow_validation_run_file,
    retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
    latest_l1_as_of_ms:$latest_l1,
    latest_l1_source:$latest_l1_source,
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_summary_only:true,
      paper_live_enabled:false
    },
    upstream_state:{
      retest_horizon_verdict:($horizon_status.verdict // null),
      research_factory_blocking_stage:($horizon_status.research_factory_gap_summary.blocking_stage // null),
      promotion_passed:($horizon_status.stage_state.promotion_passed // null),
      shadow_created:($horizon_status.stage_state.shadow_created // null),
      paper_created:($horizon_status.stage_state.paper_created // null),
      live_enabled:false
    },
    observation_summary:{
      shadow_validation_count:($runs | length),
      pending_count:($pending | length),
      symbol_count:($runs | map(.symbol_canonical // empty) | unique | length),
      candidate_lifecycle_count:($by_candidate | length),
      symbols:($runs | map(.symbol_canonical // empty) | unique_sorted),
      status_counts:($runs | counts_by(status_value)),
      target_window_materialized_count:($target_materialized | length),
      absolute_window_materialized_count:($absolute_materialized | length),
      target_window_materialized_candidate_count:($target_ready_candidates | length),
      sample_requirement_met_candidate_count:($sample_ready_candidates | length),
      earliest_target_exit_deadline_ms:($runs | map(target_exit_deadline_ms) | map(select(. != null)) | min // null),
      latest_absolute_exit_deadline_ms:($runs | map(.holding_policy.absolute_exit_deadline_ms // null) | map(select(. != null)) | max // null)
    },
    next_decision:{
      verdict:(
        if ($runs | length) == 0 then "NO_SHADOW_VALIDATION_RUNS"
        elif $latest_l1 == null then "DISCOVER_LATEST_MARKET_L1_AS_OF"
        elif ($target_ready_candidates | length) == 0 then "WAIT_FOR_TARGET_HOLDING_WINDOW"
        elif ($sample_ready_candidates | length) == 0 then "TARGET_WINDOW_MATERIALIZED_SAMPLE_REQUIREMENT_NOT_PROVEN"
        else "REVIEW_SHADOW_OBSERVATION_FOR_COMPLETION" end
      ),
      safe_next_actions:[
        if $latest_l1 == null then "discover_latest_market_l1_as_of" else empty end,
        if ($target_ready_candidates | length) == 0 then "wait_for_target_holding_window_materialization" else empty end,
        if ($target_ready_candidates | length) > 0 then "review_target_window_materialized_shadow_runs" else empty end,
        if ($sample_ready_candidates | length) == 0 then "accumulate_or_define_shadow_observation_samples" else empty end
      ],
      blocked_actions:[
        "do_not_mark_pending_shadow_passed_without_completion_evidence",
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_shadow_observation"
      ]
    },
    by_symbol:$by_symbol,
    by_candidate_lifecycle_key:$by_candidate
  }
