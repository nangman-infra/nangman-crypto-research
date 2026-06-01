[
  .partition_aggregates[]?
  | select((.missing_market_replay_data_count // 0) > 0)
  | {
      research_aggregate_key,
      symbol_canonical,
      hypothesis_type,
      replay_run_count,
      completed_count,
      missing_market_replay_data_count,
      gate_reason_codes,
      research_partition_keys
    }
] as $rows
| {
    count:($rows | length),
    rows:$rows
  }
