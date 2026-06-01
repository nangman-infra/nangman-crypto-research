def unique_sorted: unique | sort;
def finite_number:
  if type == "number" and (. | tostring) != "nan" then . else null end;

group_by(.symbol_canonical) as $by_symbol
| {
    present:true,
    aggregate_count:length,
    symbol_count:($by_symbol | length),
    symbols:(
      $by_symbol
      | map({
          symbol:.[0].symbol_canonical,
          aggregate_count:length,
          replay_run_count:(map(.replay_run_count // 0) | add),
          completed_count:(map(.completed_count // 0) | add),
          active_replay_run_count:(map(.active_replay_run_count // 0) | add),
          expired_replay_run_count:(map(.expired_replay_run_count // 0) | add),
          max_completed_count:(map(.completed_count // 0) | max // 0),
          max_effective_completed_sample_weight:(map(.effective_completed_sample_weight // 0) | max // 0),
          gate_biases:(map(.gate_bias) | unique_sorted),
          reason_codes:(map(.latest_reason_codes // []) | add | unique_sorted),
          best_weighted_mean_net_after_cost_bps:(
            map(.weighted_mean_net_after_cost_bps | finite_number)
            | map(select(. != null))
            | max // null
          ),
          best_cost_stressed_mean_net_after_cost_bps:(
            map(.cost_stressed_mean_net_after_cost_bps | finite_number)
            | map(select(. != null))
            | max // null
          )
        })
      | sort_by(.symbol)
    ),
    strongest_positive_retest:(
      map(select(.gate_bias == "RETEST_BIAS"))
      | map(select(.weighted_mean_net_after_cost_bps? != null))
      | sort_by(.weighted_mean_net_after_cost_bps)
      | reverse
      | .[0:10]
      | map({
          symbol:.symbol_canonical,
          research_aggregate_key,
          completed_count,
          effective_completed_sample_weight,
          replay_run_count,
          weighted_mean_net_after_cost_bps,
          cost_stressed_mean_net_after_cost_bps,
          latest_reason_codes
        })
    )
  }
