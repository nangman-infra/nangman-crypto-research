#!/usr/bin/env bash
set -euo pipefail

REPORT_FILE="${RESEARCH_REPORT_FILE:-${1:-}}"
REGISTRY_FILE="${RESEARCH_AGGREGATE_REGISTRY_FILE:-${2:-}}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_absolute_file() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute file path" >&2
    exit 1
  fi
  if [[ ! -f "$path" ]]; then
    echo "$name does not exist: $path" >&2
    exit 1
  fi
}

registry_summary_json() {
  local registry_file="$1"
  if [[ -z "$registry_file" ]]; then
    jq -n '{
      present:false,
      aggregate_count:0,
      symbol_count:0,
      symbols:[],
      strongest_positive_retest:[]
    }'
    return
  fi

  require_absolute_file "RESEARCH_AGGREGATE_REGISTRY_FILE or second argument" "$registry_file"
  jq -s '
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
  ' "$registry_file"
}

require_command jq
require_absolute_file "RESEARCH_REPORT_FILE or first argument" "$REPORT_FILE"

registry_summary="$(registry_summary_json "$REGISTRY_FILE")"

jq \
  --arg report_file "$REPORT_FILE" \
  --arg registry_file "$REGISTRY_FILE" \
  --argjson registry "$registry_summary" '
    def reason_count_rows:
      [.summary_findings[]?.reason_codes[]]
      | group_by(.)
      | map({reason:.[0], count:length})
      | sort_by(.count, .reason)
      | reverse;

    def bias_count_rows:
      [.summary_findings[]?.bias]
      | group_by(.)
      | map({bias:.[0], count:length})
      | sort_by(.count, .bias)
      | reverse;

    reason_count_rows as $reason_counts
    | bias_count_rows as $bias_counts
    | {
        schema_version:"research_report_summary_v1",
        report_file:$report_file,
        registry_file:(if $registry_file == "" then null else $registry_file end),
        report:{
          schema_version,
          research_run_report_id,
          research_run_status,
          research_packet_id,
          run_scope,
          created_at_ms,
          source_candidate_count:((.source_candidate_ids // []) | length),
          replay_run_count:((.replay_run_ids // []) | length),
          partition_count,
          retest_candidate_count:((.retest_candidate_keys // []) | length),
          pruned_candidate_count:((.pruned_candidate_keys // []) | length),
          surviving_candidate_count:((.surviving_candidate_keys // []) | length),
          shadow_validation_count:((.shadow_validation_runs // []) | length),
          paper_trade_candidate_count:((.paper_trade_candidates // []) | length),
          top_symbols
        },
        stage_state:{
          research_replay_completed:(.research_run_status == "completed"),
          all_candidates_retest:(
            ((.source_candidate_ids // []) | length) > 0
            and ((.retest_candidate_keys // []) | length) == ((.source_candidate_ids // []) | length)
          ),
          promotion_passed:(((.surviving_candidate_keys // []) | length) > 0),
          shadow_created:(((.shadow_validation_runs // []) | length) > 0),
          paper_created:(((.paper_trade_candidates // []) | length) > 0)
        },
        bias_counts:$bias_counts,
        reason_counts:$reason_counts,
        top_blockers:($reason_counts[0:10]),
        registry:$registry,
        next_research_needs:[
          if (($reason_counts[]? | select(.reason == "promotion_sample_count_below_minimum") | .count) // 0) > 0
            then "increase_completed_native_replay_samples" else empty end,
          if (($reason_counts[]? | select(.reason == "unseen_window_validation_not_proven") | .count) // 0) > 0
            then "materialize_unseen_replay_windows" else empty end,
          if (($reason_counts[]? | select(.reason == "train_validation_split_not_materialized") | .count) // 0) > 0
            then "materialize_train_validation_split" else empty end,
          if (($reason_counts[]? | select(.reason == "missing_native_replay_market_data") | .count) // 0) > 0
            then "extend_market_l1_horizon_coverage" else empty end,
          if (($reason_counts[]? | select(.reason == "liquidity_filter_not_materialized") | .count) // 0) > 0
            then "materialize_liquidity_filter_inputs" else empty end
        ]
      }
  ' "$REPORT_FILE"

echo "research report summary completed" >&2
