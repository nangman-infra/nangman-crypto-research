#!/usr/bin/env bash
set -euo pipefail

SHADOW_VALIDATION_RUN_FILE="${RESEARCH_SHADOW_VALIDATION_RUN_FILE:-${1:-}}"
HORIZON_STATUS_FILE="${RESEARCH_RETEST_HORIZON_STATUS_FILE:-${2:-}}"

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

require_command cp
require_command date
require_command jq
require_command mktemp

require_absolute_file "RESEARCH_SHADOW_VALIDATION_RUN_FILE or first argument" "$SHADOW_VALIDATION_RUN_FILE"

if [[ -n "$HORIZON_STATUS_FILE" ]]; then
  require_absolute_file "RESEARCH_RETEST_HORIZON_STATUS_FILE or second argument" "$HORIZON_STATUS_FILE"
fi

horizon_status_input="$(mktemp)"
trap 'rm -f "$horizon_status_input"' EXIT

if [[ -n "$HORIZON_STATUS_FILE" ]]; then
  cp "$HORIZON_STATUS_FILE" "$horizon_status_input"
else
  printf 'null\n' > "$horizon_status_input"
fi

jq -s \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg shadow_validation_run_file "$SHADOW_VALIDATION_RUN_FILE" \
  --arg horizon_status_file "$HORIZON_STATUS_FILE" \
  --slurpfile horizon_status_input "$horizon_status_input" \
  '
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

    records as $runs
    | ($horizon_status_input[0] // null) as $horizon_status
    | ($runs | map(select(status_value == "pending"))) as $pending
    | ($runs | map(select(status_value == "completed"))) as $completed
    | ($runs | map(select(status_value == "failed"))) as $failed
    | ($runs | map(select(is_completed_passed_shadow))) as $paper_eligible
    | ($runs | map(select((.termination_policy.no_order_execution // false) != true))) as $order_execution_violations
    | ($runs | map(select((.paper_trade_candidate_contract_version // "") != "paper_trade_candidate_v1"))) as $paper_contract_mismatches
    | {
        schema_version:"research_shadow_validation_status_checkpoint_v1",
        generated_at:$generated_at,
        shadow_validation_run_file:$shadow_validation_run_file,
        retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
        safety:{
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          local_summary_only:true,
          paper_live_enabled:false
        },
        upstream_state:{
          retest_horizon_verdict:($horizon_status.verdict // null),
          research_factory_blocking_stage:($horizon_status.research_factory_gap_summary.blocking_stage // null),
          selected_candidate_count:($horizon_status.batch_state.selected_candidate_count // null),
          replay_run_count:($horizon_status.batch_state.replay_run_count // null),
          promotion_passed:($horizon_status.stage_state.promotion_passed // null),
          paper_created:($horizon_status.stage_state.paper_created // null),
          live_enabled:false
        },
        stage_state:{
          shadow_created:(($runs | length) > 0),
          shadow_completed:(($completed | length) > 0),
          shadow_passed:(($paper_eligible | length) > 0),
          paper_input_ready:(
            ($paper_eligible | length) > 0
            and ($order_execution_violations | length) == 0
            and ($paper_contract_mismatches | length) == 0
          ),
          paper_created:($horizon_status.stage_state.paper_created // false),
          live_enabled:false
        },
        shadow_validation_summary:{
          shadow_validation_count:($runs | length),
          candidate_lifecycle_count:($runs | map(.candidate_lifecycle_key // empty) | unique | length),
          symbol_count:($runs | map(.symbol_canonical // empty) | unique | length),
          symbols:($runs | map(.symbol_canonical // empty) | unique_sorted),
          schema_versions:($runs | map(.schema_version // "unknown") | unique_sorted),
          status_counts:($runs | counts_by(status_value)),
          passed_counts:($runs | counts_by((.passed // false))),
          pending_count:($pending | length),
          completed_count:($completed | length),
          failed_count:($failed | length),
          completed_passed_shadow_count:($paper_eligible | length),
          paper_contract_mismatch_count:($paper_contract_mismatches | length),
          no_order_execution_violation_count:($order_execution_violations | length)
        },
        paper_gate:{
          paper_generation_precondition_met:(
            ($paper_eligible | length) > 0
            and ($order_execution_violations | length) == 0
            and ($paper_contract_mismatches | length) == 0
          ),
          required_shadow_status:"completed",
          required_shadow_passed:true,
          required_paper_trade_candidate_contract_version:"paper_trade_candidate_v1",
          eligible_shadow_validation_run_ids:($paper_eligible | map(.shadow_validation_run_id) | unique_sorted),
          eligible_candidate_lifecycle_keys:($paper_eligible | map(.candidate_lifecycle_key) | unique_sorted),
          blocked_actions:[
            if ($paper_eligible | length) == 0 then "do_not_create_paper_without_completed_passed_shadow" else empty end,
            if ($pending | length) > 0 then "do_not_treat_pending_shadow_as_passed" else empty end,
            if ($paper_contract_mismatches | length) > 0 then "do_not_use_shadow_with_paper_contract_mismatch" else empty end,
            if ($order_execution_violations | length) > 0 then "do_not_use_shadow_with_order_execution_enabled" else empty end,
            "do_not_enable_live_from_shadow_review"
          ],
          safe_next_actions:[
            if ($pending | length) > 0 then "observe_pending_shadow_validation_runs_until_completed" else empty end,
            if ($failed | length) > 0 then "inspect_failed_shadow_validation_runs" else empty end,
            if ($paper_contract_mismatches | length) > 0 then "inspect_shadow_paper_contract_mismatches" else empty end,
            if ($paper_eligible | length) > 0 then "review_completed_passed_shadow_before_paper" else empty end
          ]
        },
        by_symbol:(
          $runs
          | sort_by(.symbol_canonical // "unknown", .candidate_lifecycle_key // "", .shadow_validation_run_id // "")
          | group_by(.symbol_canonical // "unknown")
          | map({
              symbol:.[0].symbol_canonical,
              shadow_validation_count:length,
              candidate_lifecycle_count:(map(.candidate_lifecycle_key // empty) | unique | length),
              status_counts:counts_by(status_value),
              pending_count:(map(select(status_value == "pending")) | length),
              completed_passed_shadow_count:(map(select(is_completed_passed_shadow)) | length),
              no_order_execution_violation_count:(map(select((.termination_policy.no_order_execution // false) != true)) | length)
            })
        ),
        by_candidate_lifecycle_key:(
          $runs
          | sort_by(.candidate_lifecycle_key // "", .symbol_canonical // "", .shadow_validation_run_id // "")
          | group_by(.candidate_lifecycle_key // "unknown")
          | map({
              candidate_lifecycle_key:.[0].candidate_lifecycle_key,
              symbols:(map(.symbol_canonical // empty) | unique_sorted),
              shadow_validation_count:length,
              status_counts:counts_by(status_value),
              pending_count:(map(select(status_value == "pending")) | length),
              completed_passed_shadow_count:(map(select(is_completed_passed_shadow)) | length),
              runs:(map(run_projection))
            })
        )
      }
  ' "$SHADOW_VALIDATION_RUN_FILE"

echo "research shadow validation status summary completed" >&2
