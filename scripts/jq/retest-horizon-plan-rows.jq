def horizon_ms($h):
  if $h == "1h" then 3600000
  elif $h == "4h" then 14400000
  elif $h == "24h" or $h == "1d" then 86400000
  elif $h == "72h" then 259200000
  elif $h == "7d" then 604800000
  else null end;

def horizon_from_aggregate_key:
  (.research_aggregate_key // "" | split(":")) as $parts
  | if ($parts | length) >= 4 then $parts[3] else "unknown" end;

def max_or_zero: if length == 0 then 0 else max end;
def any_true: any(. == true);
def unique_sorted: unique | sort;
def latest_as_of:
  if $latest_l1_as_of_ms == "" then null else ($latest_l1_as_of_ms | tonumber) end;

def retest_horizon_next_action(
  $horizon_ms;
  $due_ms;
  $latest_l1;
  $matched;
  $reason_codes;
  $completed;
  $min_completed;
  $unseen;
  $required_unseen;
  $split_required;
  $split_materialized;
  $liquidity_required;
  $liquidity_materialized
):
  if $horizon_ms == null then "define_horizon_duration"
  elif $due_ms == null then "define_replay_boundary"
  elif $latest_l1 == null then "discover_latest_market_l1_as_of"
  elif $latest_l1 < $due_ms then "wait_for_market_l1_horizon"
  elif ($matched | length) == 0 then "run_research_replay_for_horizon"
  elif (($reason_codes | index("missing_native_replay_market_data")) != null
    or ($reason_codes | index("native_replay_horizon_not_materialized")) != null) then "extend_market_l1_horizon_coverage"
  elif $completed == 0 then "materialize_completed_native_replay_sample"
  elif $completed < $min_completed then "accumulate_completed_native_replay_samples"
  elif $unseen < $required_unseen then "materialize_unseen_replay_windows"
  elif $split_required and ($split_materialized | not) then "materialize_train_validation_split"
  elif $liquidity_required and $liquidity_materialized < $completed then "materialize_liquidity_filter_inputs"
  elif ($reason_codes | length) > 0 then "inspect_remaining_gate_reasons"
  else "promotion_gate_ready_for_review" end;

def retest_horizon_row($bundle; $horizon; $report; $min_completed; $aggregates; $latest_l1):
  (horizon_ms($horizon)) as $horizon_ms
  | ($bundle.forbidden_lookahead_boundary_ms // $bundle.decision_available_at_ms) as $boundary_ms
  | (if $horizon_ms == null or $boundary_ms == null then null else ($boundary_ms + $horizon_ms) end) as $due_ms
  | (
      $aggregates
      | map(
          select(((.source_candidate_ids // []) | index($bundle.candidate_id)) != null)
          | select(horizon_from_aggregate_key == $horizon)
        )
    ) as $matched
  | ($matched | map(.completed_count // 0) | max_or_zero) as $completed
  | ($matched | map(.effective_completed_sample_weight // 0) | max_or_zero) as $effective
  | ($matched | map(.replay_run_count // 0) | add // 0) as $replay_runs
  | ($matched | map(.inferred_unseen_window_count // 0) | max_or_zero) as $unseen
  | ($bundle.validation_requirements.min_unseen_windows // 0) as $required_unseen
  | ($bundle.validation_requirements.required_train_validation_split // false) as $split_required
  | ($bundle.validation_requirements.include_liquidity_filter // false) as $liquidity_required
  | ($matched | map(.train_validation_split_summary.materialized // false) | any_true) as $split_materialized
  | ($matched | map(.liquidity_filter_materialized_count // 0) | max_or_zero) as $liquidity_materialized
  | ($matched | map(.missing_market_replay_data_count // 0) | add // 0) as $missing_market_replay_data_count
  | ($matched | map(.gate_reason_codes // []) | add // [] | unique_sorted) as $aggregate_gate_reason_codes
  | (
      $aggregate_gate_reason_codes
      + (if $missing_market_replay_data_count > 0 then ["missing_native_replay_market_data"] else [] end)
      | unique_sorted
    ) as $reason_codes
  | (
      ($report.summary_findings // [])
      | map(select(.candidate_id == $bundle.candidate_id))
      | map(.reason_codes // [])
      | add // []
      | unique_sorted
    ) as $candidate_reason_codes
  | {
      candidate_id:$bundle.candidate_id,
      candidate_lifecycle_key:$bundle.candidate_lifecycle_key,
      symbols:($bundle.normalized_symbols // []),
      primary_symbol:(($bundle.normalized_symbols // [])[0] // null),
      hypothesis_type:$bundle.hypothesis_type,
      research_priority:$bundle.research_priority,
      horizon:$horizon,
      horizon_ms:$horizon_ms,
      decision_available_at_ms:$bundle.decision_available_at_ms,
      forbidden_lookahead_boundary_ms:$boundary_ms,
      horizon_due_ms:$due_ms,
      latest_l1_as_of_ms:$latest_l1,
      horizon_market_data_materialized:(if $latest_l1 == null or $due_ms == null then null else $latest_l1 >= $due_ms end),
      replay_run_count:$replay_runs,
      completed_count:$completed,
      effective_completed_sample_weight:$effective,
      completed_sample_deficit:(if $completed >= $min_completed then 0 else ($min_completed - $completed) end),
      inferred_unseen_window_count:$unseen,
      required_unseen_window_count:$required_unseen,
      unseen_window_deficit:(if $unseen >= $required_unseen then 0 else ($required_unseen - $unseen) end),
      train_validation_split_required:$split_required,
      train_validation_split_materialized:$split_materialized,
      liquidity_filter_required:$liquidity_required,
      liquidity_filter_materialized_count:$liquidity_materialized,
      missing_market_replay_data_count:$missing_market_replay_data_count,
      aggregate_count:($matched | length),
      gate_biases:($matched | map(.gate_bias) | unique_sorted),
      reason_codes:$reason_codes,
      candidate_reason_codes:$candidate_reason_codes,
      next_action:retest_horizon_next_action(
        $horizon_ms;
        $due_ms;
        $latest_l1;
        $matched;
        $reason_codes;
        $completed;
        $min_completed;
        $unseen;
        $required_unseen;
        $split_required;
        $split_materialized;
        $liquidity_required;
        $liquidity_materialized
      )
    };

def retest_horizon_rows($bundles; $report; $min_completed; $aggregates; $latest_l1):
  [
    $bundles[] as $bundle
    | ($bundle.allowed_horizons // [])[] as $horizon
    | retest_horizon_row($bundle; $horizon; $report; $min_completed; $aggregates; $latest_l1)
  ];
