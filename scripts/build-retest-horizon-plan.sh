#!/usr/bin/env bash
set -euo pipefail

MANIFEST_FILE="${RESEARCH_RETEST_PLAN_MANIFEST_FILE:-${1:-}}"
REPORT_FILE="${RESEARCH_RETEST_PLAN_REPORT_FILE:-${2:-}}"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
MARKET_L1_BUCKET="${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
LATEST_L1_AS_OF_MS="${RESEARCH_RETEST_PLAN_LATEST_L1_AS_OF_MS:-}"

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

fetch_candidate_bundle() {
  local uri="$1"
  case "$uri" in
    s3://*)
      require_command aws
      aws_cmd s3 cp "$uri" - | jq -c .
      ;;
    /*)
      jq -c . "$uri"
      ;;
    *)
      echo "candidate bundle uri must be s3:// or absolute file path: $uri" >&2
      exit 1
      ;;
  esac
}

discover_latest_l1_as_of_ms() {
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

require_command jq
require_command sed
require_command mktemp

require_absolute_file "RESEARCH_RETEST_PLAN_MANIFEST_FILE or first argument" "$MANIFEST_FILE"
require_absolute_file "RESEARCH_RETEST_PLAN_REPORT_FILE or second argument" "$REPORT_FILE"
positive_or_empty_integer_arg "RESEARCH_RETEST_PLAN_LATEST_L1_AS_OF_MS" "$LATEST_L1_AS_OF_MS"

if [[ -z "$LATEST_L1_AS_OF_MS" ]]; then
  LATEST_L1_AS_OF_MS="$(discover_latest_l1_as_of_ms || true)"
fi

bundle_jsonl="$(mktemp)"
bundles_json_file="$(mktemp)"
trap 'rm -f "$bundle_jsonl" "$bundles_json_file"' EXIT

while IFS= read -r uri; do
  [[ -n "$uri" ]] || continue
  fetch_candidate_bundle "$uri" >> "$bundle_jsonl"
done < <(jq -r '.candidate_bundle_refs[]?.uri // empty' "$MANIFEST_FILE")

if [[ ! -s "$bundle_jsonl" ]]; then
  echo "manifest has no readable candidate_bundle_refs" >&2
  exit 1
fi

jq -s '.' "$bundle_jsonl" > "$bundles_json_file"

jq \
  --arg manifest_file "$MANIFEST_FILE" \
  --arg report_file "$REPORT_FILE" \
  --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
  --slurpfile bundles_file "$bundles_json_file" \
  '
    def horizon_ms($h):
      if $h == "1h" then 3600000
      elif $h == "4h" then 14400000
      elif $h == "24h" or $h == "1d" then 86400000
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

    ($bundles_file[0] // []) as $bundles
    | . as $report
    | ($report.research_gate_policy.min_completed_samples_for_shadow // 30) as $min_completed
    | ($report.partition_aggregates // []) as $aggregates
    | latest_as_of as $latest_l1
    | [
        $bundles[] as $bundle
        | ($bundle.allowed_horizons // [])[] as $horizon
        | (horizon_ms($horizon)) as $horizon_ms
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
          ) as $aggregate_reason_codes
        | (
            ($report.summary_findings // [])
            | map(select(.candidate_id == $bundle.candidate_id))
            | map(.reason_codes // [])
            | add // []
            | unique_sorted
          ) as $candidate_reason_codes
        | $aggregate_reason_codes as $reason_codes
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
            horizon_market_data_materialized:(
              if $latest_l1 == null or $due_ms == null then null else $latest_l1 >= $due_ms end
            ),
            replay_run_count:$replay_runs,
            completed_count:$completed,
            effective_completed_sample_weight:$effective,
            completed_sample_deficit:(
              if $completed >= $min_completed then 0 else ($min_completed - $completed) end
            ),
            inferred_unseen_window_count:$unseen,
            required_unseen_window_count:$required_unseen,
            unseen_window_deficit:(
              if $unseen >= $required_unseen then 0 else ($required_unseen - $unseen) end
            ),
            train_validation_split_required:$split_required,
            train_validation_split_materialized:$split_materialized,
            liquidity_filter_required:$liquidity_required,
            liquidity_filter_materialized_count:$liquidity_materialized,
            missing_market_replay_data_count:$missing_market_replay_data_count,
            aggregate_count:($matched | length),
            gate_biases:($matched | map(.gate_bias) | unique_sorted),
            reason_codes:$reason_codes,
            candidate_reason_codes:$candidate_reason_codes,
            next_action:(
              if $horizon_ms == null then "define_horizon_duration"
              elif $due_ms == null then "define_replay_boundary"
              elif $latest_l1 == null then "discover_latest_market_l1_as_of"
              elif $latest_l1 < $due_ms then "wait_for_market_l1_horizon"
              elif ($matched | length) == 0 then "run_research_replay_for_horizon"
              elif (
                ($reason_codes | index("missing_native_replay_market_data")) != null
                or ($reason_codes | index("native_replay_horizon_not_materialized")) != null
              ) then "extend_market_l1_horizon_coverage"
              elif $completed == 0 then "materialize_completed_native_replay_sample"
              elif $completed < $min_completed then "accumulate_completed_native_replay_samples"
              elif $unseen < $required_unseen then "materialize_unseen_replay_windows"
              elif $split_required and ($split_materialized | not) then "materialize_train_validation_split"
              elif $liquidity_required and $liquidity_materialized < $completed then "materialize_liquidity_filter_inputs"
              elif ($reason_codes | length) > 0 then "inspect_remaining_gate_reasons"
              else "promotion_gate_ready_for_review" end
            )
          }
      ] as $horizon_rows
    | {
        schema_version:"research_retest_horizon_plan_v1",
        manifest_file:$manifest_file,
        report_file:$report_file,
        latest_l1_as_of_ms:$latest_l1,
        research_gate_policy:$report.research_gate_policy,
        summary:{
          candidate_count:($bundles | length),
          horizon_count:($horizon_rows | length),
          symbols:($horizon_rows | map(.primary_symbol) | unique_sorted),
          next_action_counts:(
            $horizon_rows
            | group_by(.next_action)
            | map({next_action:.[0].next_action, count:length})
            | sort_by(.count, .next_action)
            | reverse
          ),
          ready_for_replay_count:(
            $horizon_rows
            | map(select(.next_action == "run_research_replay_for_horizon" or .next_action == "materialize_completed_native_replay_sample"))
            | length
          ),
          waiting_for_market_l1_count:(
            $horizon_rows
            | map(select(.next_action == "wait_for_market_l1_horizon"))
            | length
          ),
          market_l1_coverage_extension_count:(
            $horizon_rows
            | map(select(.next_action == "extend_market_l1_horizon_coverage"))
            | length
          ),
          sample_accumulation_count:(
            $horizon_rows
            | map(select(.next_action == "accumulate_completed_native_replay_samples"))
            | length
          ),
          promotion_ready_for_review_count:(
            $horizon_rows
            | map(select(.next_action == "promotion_gate_ready_for_review"))
            | length
          )
        },
        by_candidate:(
          $horizon_rows
          | group_by(.candidate_id)
          | map({
              candidate_id:.[0].candidate_id,
              symbols:.[0].symbols,
              horizons:map({
                horizon,
                horizon_due_ms,
                horizon_market_data_materialized,
                replay_run_count,
                completed_count,
                completed_sample_deficit,
                inferred_unseen_window_count,
                unseen_window_deficit,
                next_action,
                reason_codes
              })
            })
        ),
        horizon_rows:$horizon_rows
      }
  ' "$REPORT_FILE"

echo "research retest horizon plan completed" >&2
