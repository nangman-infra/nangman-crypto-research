#!/usr/bin/env bash
set -euo pipefail

SHADOW_VALIDATION_RUN_FILE="${RESEARCH_SHADOW_VALIDATION_RUN_FILE:-${1:-}}"
HORIZON_STATUS_FILE="${RESEARCH_RETEST_HORIZON_STATUS_FILE:-${2:-}}"
LATEST_L1_AS_OF_MS="${RESEARCH_SHADOW_OBSERVATION_LATEST_L1_AS_OF_MS:-${3:-}}"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
MARKET_L1_BUCKET="${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"

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

positive_or_empty_integer_arg() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" ]]; then
    return
  fi
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

aws_cmd() {
  aws --region "$REGION" "$@"
}

discover_latest_l1_as_of_ms_from_s3() {
  if [[ -z "$MARKET_L1_BUCKET" ]]; then
    return
  fi
  require_command aws
  aws_cmd s3api list-objects-v2 \
    --bucket "$MARKET_L1_BUCKET" \
    --prefix "symbol_universe_snapshot/run_id=" \
    --output json \
  | jq -r '
      (.Contents // [])
      | map(
          . as $object
          | ($object.Key | capture("run_id=l1_(?<start>[0-9]+)_(?<end>[0-9]+)_(?<generated>[0-9]+)")? // {}) as $run
          | {
              key:$object.Key,
              last_modified:$object.LastModified,
              run_end_ms:(($run.end // "0") | tonumber),
              run_generated_ms:(($run.generated // "0") | tonumber)
            }
        )
      | sort_by(.run_end_ms, .last_modified, .key)
      | last
      | if . == null or .run_end_ms == 0 then empty else .run_end_ms end
    '
}

discover_latest_l1_as_of_ms_from_horizon_status() {
  if [[ -z "$HORIZON_STATUS_FILE" ]]; then
    return
  fi
  local plan_file
  plan_file="$(jq -r '.retest_horizon_plan_file // empty' "$HORIZON_STATUS_FILE")"
  if [[ -z "$plan_file" || ! -f "$plan_file" ]]; then
    return
  fi
  jq -r '.latest_l1_as_of_ms // empty' "$plan_file"
}

require_command cp
require_command date
require_command jq
require_command mktemp

require_absolute_file "RESEARCH_SHADOW_VALIDATION_RUN_FILE or first argument" "$SHADOW_VALIDATION_RUN_FILE"
if [[ -n "$HORIZON_STATUS_FILE" ]]; then
  require_absolute_file "RESEARCH_RETEST_HORIZON_STATUS_FILE or second argument" "$HORIZON_STATUS_FILE"
fi
positive_or_empty_integer_arg "RESEARCH_SHADOW_OBSERVATION_LATEST_L1_AS_OF_MS or third argument" "$LATEST_L1_AS_OF_MS"

latest_l1_source="explicit"
if [[ -z "$LATEST_L1_AS_OF_MS" ]]; then
  LATEST_L1_AS_OF_MS="$(discover_latest_l1_as_of_ms_from_horizon_status || true)"
  latest_l1_source="retest_horizon_plan"
fi
if [[ -z "$LATEST_L1_AS_OF_MS" ]]; then
  LATEST_L1_AS_OF_MS="$(discover_latest_l1_as_of_ms_from_s3 || true)"
  latest_l1_source="s3_symbol_universe_snapshot"
fi
if [[ -z "$LATEST_L1_AS_OF_MS" ]]; then
  latest_l1_source="absent"
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
  --argjson generated_at_ms "$(date -u +%s)000" \
  --arg shadow_validation_run_file "$SHADOW_VALIDATION_RUN_FILE" \
  --arg horizon_status_file "$HORIZON_STATUS_FILE" \
  --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
  --arg latest_l1_source "$latest_l1_source" \
  --slurpfile horizon_status_input "$horizon_status_input" \
  '
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
      ($runs | length) as $count
      | max_required_shadow_samples($runs) as $required
      | {
          observed_shadow_run_count:$count,
          required_shadow_sample_count:$required,
          sample_requirement_met:($required > 0 and $count >= $required),
          sample_deficit:(if $count >= $required then 0 else ($required - $count) end)
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

    records as $runs
    | ($horizon_status_input[0] // null) as $horizon_status
    | latest_l1 as $latest_l1
    | ($runs | map(select(status_value == "pending"))) as $pending
    | ($runs | map(select(target_window_materialized))) as $target_materialized
    | ($runs | map(select(absolute_window_materialized))) as $absolute_materialized
    | (
        $runs
        | sort_by(.candidate_lifecycle_key // "", .symbol_canonical // "", .shadow_validation_run_id // "")
        | group_by(.candidate_lifecycle_key // "unknown")
        | map(. as $candidate_runs | sample_status($candidate_runs) as $sample | {
            candidate_lifecycle_key:.[0].candidate_lifecycle_key,
            symbols:(map(.symbol_canonical // empty) | unique_sorted),
            status_counts:counts_by(status_value),
            target_window_materialized_count:(map(select(target_window_materialized)) | length),
            absolute_window_materialized_count:(map(select(absolute_window_materialized)) | length),
            observation_sample_status:$sample,
            runs:(map(run_projection))
          })
      ) as $by_candidate
    | (
        $runs
        | sort_by(.symbol_canonical // "unknown", .candidate_lifecycle_key // "", .shadow_validation_run_id // "")
        | group_by(.symbol_canonical // "unknown")
        | map(. as $symbol_runs | sample_status($symbol_runs) as $sample | {
            symbol:.[0].symbol_canonical,
            candidate_lifecycle_count:(map(.candidate_lifecycle_key // empty) | unique | length),
            shadow_validation_count:length,
            status_counts:counts_by(status_value),
            target_window_materialized_count:(map(select(target_window_materialized)) | length),
            absolute_window_materialized_count:(map(select(absolute_window_materialized)) | length),
            observation_sample_status:$sample
          })
      ) as $by_symbol
    | ($by_candidate | map(select(.observation_sample_status.sample_requirement_met == true))) as $sample_ready_candidates
    | ($by_candidate | map(select(.target_window_materialized_count > 0))) as $target_ready_candidates
    | {
        schema_version:"research_shadow_observation_plan_v1",
        generated_at:$generated_at,
        generated_at_ms:$generated_at_ms,
        shadow_validation_run_file:$shadow_validation_run_file,
        retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
        latest_l1_as_of_ms:$latest_l1,
        latest_l1_source:$latest_l1_source,
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
          promotion_passed:($horizon_status.stage_state.promotion_passed // null),
          shadow_created:($horizon_status.stage_state.shadow_created // null),
          paper_created:($horizon_status.stage_state.paper_created // null),
          live_enabled:false
        },
        observation_summary:{
          shadow_validation_count:($runs | length),
          pending_count:($pending | length),
          symbol_count:($runs | map(.symbol_canonical // empty) | unique | length),
          candidate_lifecycle_count:($by_candidate | length),
          symbols:($runs | map(.symbol_canonical // empty) | unique_sorted),
          status_counts:($runs | counts_by(status_value)),
          target_window_materialized_count:($target_materialized | length),
          absolute_window_materialized_count:($absolute_materialized | length),
          target_window_materialized_candidate_count:($target_ready_candidates | length),
          sample_requirement_met_candidate_count:($sample_ready_candidates | length),
          earliest_target_exit_deadline_ms:($runs | map(target_exit_deadline_ms) | map(select(. != null)) | min // null),
          latest_absolute_exit_deadline_ms:($runs | map(.holding_policy.absolute_exit_deadline_ms // null) | map(select(. != null)) | max // null)
        },
        next_decision:{
          verdict:(
            if ($runs | length) == 0 then "NO_SHADOW_VALIDATION_RUNS"
            elif $latest_l1 == null then "DISCOVER_LATEST_MARKET_L1_AS_OF"
            elif ($target_ready_candidates | length) == 0 then "WAIT_FOR_TARGET_HOLDING_WINDOW"
            elif ($sample_ready_candidates | length) == 0 then "TARGET_WINDOW_MATERIALIZED_SAMPLE_REQUIREMENT_NOT_PROVEN"
            else "REVIEW_SHADOW_OBSERVATION_FOR_COMPLETION" end
          ),
          safe_next_actions:[
            if $latest_l1 == null then "discover_latest_market_l1_as_of" else empty end,
            if ($target_ready_candidates | length) == 0 then "wait_for_target_holding_window_materialization" else empty end,
            if ($target_ready_candidates | length) > 0 then "review_target_window_materialized_shadow_runs" else empty end,
            if ($sample_ready_candidates | length) == 0 then "accumulate_or_define_shadow_observation_samples" else empty end
          ],
          blocked_actions:[
            "do_not_mark_pending_shadow_passed_without_completion_evidence",
            "do_not_create_paper_without_completed_passed_shadow",
            "do_not_enable_live_from_shadow_observation"
          ]
        },
        by_symbol:$by_symbol,
        by_candidate_lifecycle_key:$by_candidate
      }
  ' "$SHADOW_VALIDATION_RUN_FILE"

echo "research shadow observation plan completed" >&2
