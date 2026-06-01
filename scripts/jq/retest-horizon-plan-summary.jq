def unique_sorted: unique | sort;

def retest_horizon_plan_summary($bundles; $horizon_rows):
  {
    candidate_count:($bundles | length),
    horizon_count:($horizon_rows | length),
    symbols:($horizon_rows | map(.primary_symbol) | unique_sorted),
    next_action_counts:(
      $horizon_rows
      | group_by(.next_action)
      | map({next_action:.[0].next_action, count:length})
      | sort_by(.count, .next_action)
      | reverse
    ),
    ready_for_replay_count:(
      $horizon_rows
      | map(select(.next_action == "run_research_replay_for_horizon" or .next_action == "materialize_completed_native_replay_sample"))
      | length
    ),
    waiting_for_market_l1_count:(
      $horizon_rows
      | map(select(.next_action == "wait_for_market_l1_horizon"))
      | length
    ),
    market_l1_coverage_extension_count:(
      $horizon_rows
      | map(select(.next_action == "extend_market_l1_horizon_coverage"))
      | length
    ),
    sample_accumulation_count:(
      $horizon_rows
      | map(select(.next_action == "accumulate_completed_native_replay_samples"))
      | length
    ),
    promotion_ready_for_review_count:(
      $horizon_rows
      | map(select(.next_action == "promotion_gate_ready_for_review"))
      | length
    )
  };

def retest_horizon_by_candidate($horizon_rows):
  $horizon_rows
  | group_by(.candidate_id)
  | map({
      candidate_id:.[0].candidate_id,
      symbols:.[0].symbols,
      horizons:map({
        horizon,
        horizon_due_ms,
        horizon_market_data_materialized,
        replay_run_count,
        completed_count,
        completed_sample_deficit,
        inferred_unseen_window_count,
        unseen_window_deficit,
        next_action,
        reason_codes
      })
    });
