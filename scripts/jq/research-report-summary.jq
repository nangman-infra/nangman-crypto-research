def reason_count_rows:
  [.summary_findings[]?.reason_codes[]]
  | group_by(.)
  | map({reason:.[0], count:length})
  | sort_by(.count, .reason)
  | reverse;

def bias_count_rows:
  [.summary_findings[]?.bias]
  | group_by(.)
  | map({bias:.[0], count:length})
  | sort_by(.count, .bias)
  | reverse;

reason_count_rows as $reason_counts
| bias_count_rows as $bias_counts
| {
    schema_version:"research_report_summary_v1",
    report_file:$report_file,
    registry_file:(if $registry_file == "" then null else $registry_file end),
    report:{
      schema_version,
      research_run_report_id,
      research_run_status,
      research_packet_id,
      run_scope,
      created_at_ms,
      source_candidate_count:((.source_candidate_ids // []) | length),
      replay_run_count:((.replay_run_ids // []) | length),
      partition_count,
      retest_candidate_count:((.retest_candidate_keys // []) | length),
      pruned_candidate_count:((.pruned_candidate_keys // []) | length),
      surviving_candidate_count:((.surviving_candidate_keys // []) | length),
      shadow_validation_count:((.shadow_validation_runs // []) | length),
      paper_trade_candidate_count:((.paper_trade_candidates // []) | length),
      top_symbols
    },
    stage_state:{
      research_replay_completed:(.research_run_status == "completed"),
      all_candidates_retest:(
        ((.source_candidate_ids // []) | length) > 0
        and ((.retest_candidate_keys // []) | length) == ((.source_candidate_ids // []) | length)
      ),
      promotion_passed:(((.surviving_candidate_keys // []) | length) > 0),
      shadow_created:(((.shadow_validation_runs // []) | length) > 0),
      paper_created:(((.paper_trade_candidates // []) | length) > 0)
    },
    bias_counts:$bias_counts,
    reason_counts:$reason_counts,
    top_blockers:($reason_counts[0:10]),
    registry:$registry,
    next_research_needs:[
      if (($reason_counts[]? | select(.reason == "promotion_sample_count_below_minimum") | .count) // 0) > 0
        then "increase_completed_native_replay_samples" else empty end,
      if (($reason_counts[]? | select(.reason == "unseen_window_validation_not_proven") | .count) // 0) > 0
        then "materialize_unseen_replay_windows" else empty end,
      if (($reason_counts[]? | select(.reason == "train_validation_split_not_materialized") | .count) // 0) > 0
        then "materialize_train_validation_split" else empty end,
      if (($reason_counts[]? | select(.reason == "missing_native_replay_market_data") | .count) // 0) > 0
        then "extend_market_l1_horizon_coverage" else empty end,
      if (($reason_counts[]? | select(.reason == "liquidity_filter_not_materialized") | .count) // 0) > 0
        then "materialize_liquidity_filter_inputs" else empty end
    ]
  }
