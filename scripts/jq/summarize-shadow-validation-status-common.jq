def records:
  if length == 1 and (.[0] | type) == "array" then .[0] else . end;

def unique_sorted: unique | sort;

def counts_by(expr):
  map(expr)
  | sort
  | group_by(.)
  | map({value:.[0], count:length});

def status_value: (.status // "pending");

def is_completed_passed_shadow:
  status_value == "completed"
  and (.passed == true)
  and ((.paper_trade_candidate_contract_version // "") == "paper_trade_candidate_v1");

def run_projection:
  {
    shadow_validation_run_id,
    candidate_lifecycle_key,
    symbol_canonical,
    status:status_value,
    passed:(.passed // false),
    paper_trade_candidate_contract_version:(.paper_trade_candidate_contract_version // null),
    no_order_execution:(.termination_policy.no_order_execution // null),
    completed_count:(.start_condition_summary.completed_count // 0),
    mean_net_after_cost_bps:(.start_condition_summary.mean_net_after_cost_bps // null),
    win_rate_ppm:(.start_condition_summary.win_rate_ppm // null),
    profit_factor_ppm:(.start_condition_summary.profit_factor_ppm // null),
    gate_reason_codes:(.start_condition_summary.gate_reason_codes // [])
  };
