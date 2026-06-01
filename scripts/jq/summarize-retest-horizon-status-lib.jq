def unique_sorted: unique | sort;

def intersect($other):
  map(select(. as $value | ($other | index($value)) != null));

def horizon_order:
  if . == "1h" then 1
  elif . == "4h" then 2
  elif . == "24h" or . == "1d" then 3
  elif . == "72h" then 4
  elif . == "7d" then 5
  else 99 end;

def action_counts:
  sort_by(.next_action)
  | group_by(.next_action)
  | map({next_action:.[0].next_action, count:length})
  | sort_by(.count, .next_action)
  | reverse;

def count_action($action):
  map(select(.next_action == $action)) | length;

def min_ms_for_action($action; $field):
  [ .[] | select(.next_action == $action) | .[$field] | select(. != null) ]
  | if length == 0 then null else min end;

def max_ms_for_action($action; $field):
  [ .[] | select(.next_action == $action) | .[$field] | select(. != null) ]
  | if length == 0 then null else max end;

def iso_ms($ms):
  if $ms == null then null else (($ms / 1000) | floor | todate) end;

def horizon_counts:
  sort_by(.horizon, .next_action)
  | group_by(.horizon)
  | map({
      horizon:.[0].horizon,
      horizon_count:length,
      candidate_count:(map(.candidate_id) | unique | length),
      next_action_counts:action_counts,
      waiting_for_market_l1_count:count_action("wait_for_market_l1_horizon"),
      market_l1_coverage_extension_count:count_action("extend_market_l1_horizon_coverage"),
      ready_for_replay_count:(
        count_action("run_research_replay_for_horizon")
        + count_action("materialize_completed_native_replay_sample")
      ),
      sample_accumulation_count:count_action("accumulate_completed_native_replay_samples"),
      promotion_ready_for_review_count:count_action("promotion_gate_ready_for_review"),
      max_completed_sample_deficit:(map(.completed_sample_deficit // 0) | max // 0),
      max_unseen_window_deficit:(map(.unseen_window_deficit // 0) | max // 0)
    })
  | sort_by(.horizon | horizon_order);

def compact_rows:
  map({
    candidate_id,
    candidate_lifecycle_key,
    primary_symbol,
    symbols,
    hypothesis_type,
    research_priority,
    horizon,
    horizon_market_data_materialized,
    replay_run_count,
    completed_count,
    completed_sample_deficit,
    inferred_unseen_window_count,
    unseen_window_deficit,
    train_validation_split_required,
    train_validation_split_materialized,
    liquidity_filter_required,
    liquidity_filter_materialized_count,
    missing_market_replay_data_count,
    gate_biases,
    reason_codes,
    next_action
  });

def tracked_horizons: ["1h", "4h", "24h"];

def candidate_horizon_state($candidate_rows; $horizon):
  ($candidate_rows | map(select(.horizon == $horizon)) | .[0]) as $row
  | if $row == null then
      {
        horizon:$horizon,
        requested:false,
        next_action:"not_requested",
        horizon_market_data_materialized:false,
        replay_run_count:0,
        completed_count:0,
        completed_sample_deficit:null,
        inferred_unseen_window_count:0,
        unseen_window_deficit:null,
        train_validation_split_materialized:false,
        liquidity_filter_materialized_count:0,
        missing_market_replay_data_count:0,
        gate_biases:[],
        reason_codes:["horizon_not_requested_by_candidate_bundle"],
        promotion_gate_ready_for_review:false
      }
    else
      {
        horizon:$horizon,
        requested:true,
        next_action:$row.next_action,
        horizon_market_data_materialized:($row.horizon_market_data_materialized // false),
        replay_run_count:($row.replay_run_count // 0),
        completed_count:($row.completed_count // 0),
        completed_sample_deficit:($row.completed_sample_deficit // null),
        inferred_unseen_window_count:($row.inferred_unseen_window_count // 0),
        unseen_window_deficit:($row.unseen_window_deficit // null),
        train_validation_split_materialized:($row.train_validation_split_materialized // false),
        liquidity_filter_materialized_count:($row.liquidity_filter_materialized_count // 0),
        missing_market_replay_data_count:($row.missing_market_replay_data_count // 0),
        gate_biases:($row.gate_biases // []),
        reason_codes:($row.reason_codes // []),
        promotion_gate_ready_for_review:($row.next_action == "promotion_gate_ready_for_review")
      }
    end;
