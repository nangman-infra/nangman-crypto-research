def records:
  if length == 1 and (.[0] | type) == "array" then .[0] else . end;

def hour_ms: 3600000;
def unique_sorted: unique | sort;

def counts_by(expr):
  map(expr)
  | sort
  | group_by(.)
  | map({value:.[0], count:length});

def latest_l1:
  if $latest_l1_as_of_ms == "" then null else ($latest_l1_as_of_ms | tonumber) end;

def status_value: (.status // "pending");

def decision_available_at_ms:
  (.holding_policy.absolute_exit_deadline_ms // null) as $absolute
  | (.holding_policy.absolute_max_holding_hours // null) as $absolute_hours
  | if $absolute == null or $absolute_hours == null then null
    else ($absolute - ($absolute_hours * hour_ms))
    end;

def target_exit_deadline_ms:
  decision_available_at_ms as $decision
  | (.holding_policy.target_max_holding_hours // null) as $target_hours
  | if $decision == null or $target_hours == null then null
    else ($decision + ($target_hours * hour_ms))
    end;

def target_window_materialized:
  latest_l1 as $l1
  | target_exit_deadline_ms as $target
  | if $l1 == null or $target == null then false else $l1 >= $target end;

def absolute_window_materialized:
  latest_l1 as $l1
  | (.holding_policy.absolute_exit_deadline_ms // null) as $absolute
  | if $l1 == null or $absolute == null then false else $l1 >= $absolute end;

def max_required_shadow_samples($runs):
  ($runs | map(.watch_window_policy.min_shadow_samples // 0) | max // 0);

def sample_status($runs):
  ($runs | length) as $observed_count
  | ($runs | map(select(target_window_materialized)) | length) as $materialized_count
  | max_required_shadow_samples($runs) as $required
  | {
      observed_shadow_run_count:$observed_count,
      target_window_materialized_shadow_run_count:$materialized_count,
      required_shadow_sample_count:$required,
      sample_requirement_basis:"target_window_materialized_shadow_run_count",
      sample_requirement_met:($required > 0 and $materialized_count >= $required),
      sample_deficit:(
        if $materialized_count >= $required then 0
        else ($required - $materialized_count)
        end
      )
    };

def run_projection:
  decision_available_at_ms as $decision_ms
  | target_exit_deadline_ms as $target_ms
  | {
      shadow_validation_run_id,
      candidate_lifecycle_key,
      symbol_canonical,
      status:status_value,
      passed:(.passed // false),
      decision_available_at_ms:$decision_ms,
      target_exit_deadline_ms:$target_ms,
      absolute_exit_deadline_ms:(.holding_policy.absolute_exit_deadline_ms // null),
      target_max_holding_hours:(.holding_policy.target_max_holding_hours // null),
      absolute_max_holding_hours:(.holding_policy.absolute_max_holding_hours // null),
      target_window_materialized:target_window_materialized,
      absolute_window_materialized:absolute_window_materialized,
      no_order_execution:(.termination_policy.no_order_execution // null),
      paper_trade_candidate_contract_version:(.paper_trade_candidate_contract_version // null),
      completed_count:(.start_condition_summary.completed_count // 0),
      mean_net_after_cost_bps:(.start_condition_summary.mean_net_after_cost_bps // null),
      gate_reason_codes:(.start_condition_summary.gate_reason_codes // [])
    };
