def aligned($value): (($value / $window_ms) | floor) * $window_ms;

[
  .[]
  | select(.result_summary.status == "missing_market_replay_data")
  | {
      replay_run_id,
      candidate_id:.source_candidate_id,
      candidate_lifecycle_key:.source_candidate_lifecycle_key,
      research_aggregate_key,
      symbol:.symbol_canonical,
      horizon:(.research_aggregate_key | split(":")[3]),
      window_start_ms,
      window_end_ms,
      reason_codes:.result_summary.reason_codes,
      expected_l1_window_starts:([
        range(aligned(.window_start_ms); aligned(.window_end_ms) + $window_ms; $window_ms)
      ])
    }
] as $rows
| {
    count:($rows | length),
    rows:$rows
  }
