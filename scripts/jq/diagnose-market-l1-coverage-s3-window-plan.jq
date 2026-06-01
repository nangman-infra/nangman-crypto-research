[
  .rows[]?
  | . as $row
  | $row.expected_l1_window_starts[]?
  | {
      symbol:$row.symbol,
      window_start_ms:.,
      source_replay_windows:[
        {
          replay_run_id:$row.replay_run_id,
          candidate_id:$row.candidate_id,
          research_aggregate_key:$row.research_aggregate_key,
          horizon:$row.horizon,
          window_start_ms:$row.window_start_ms,
          window_end_ms:$row.window_end_ms
        }
      ]
    }
]
| sort_by(.symbol, .window_start_ms)
| group_by(.symbol + ":" + (.window_start_ms | tostring))
| map({
    symbol:.[0].symbol,
    window_start_ms:.[0].window_start_ms,
    source_replay_windows:(map(.source_replay_windows[]) | unique)
  })
