def unique_sorted: unique | sort;

[
  .horizon_rows[]?
  | select(.next_action == "extend_market_l1_horizon_coverage")
  | {
      candidate_id,
      primary_symbol,
      hypothesis_type,
      horizon,
      window_start_ms:.forbidden_lookahead_boundary_ms,
      window_end_ms:.horizon_due_ms,
      horizon_market_data_materialized,
      replay_run_count,
      completed_count,
      missing_market_replay_data_count,
      reason_codes
    }
] as $rows
| {
    count:($rows | length),
    by_symbol_horizon:(
      $rows
      | sort_by(.primary_symbol, .horizon)
      | group_by(.primary_symbol + ":" + .horizon)
      | map({
          symbol:.[0].primary_symbol,
          horizon:.[0].horizon,
          count:length,
          missing_market_replay_data_count:(map(.missing_market_replay_data_count // 0) | add // 0)
        })
    ),
    rows:$rows
  }
