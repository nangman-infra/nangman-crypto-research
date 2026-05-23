#!/usr/bin/env bash
set -euo pipefail

PLAN_FILE="${RESEARCH_RETEST_HORIZON_PLAN_FILE:-${1:-}}"
REPORT_FILE="${RESEARCH_REPORT_FILE:-${2:-}}"
REPLAY_RUN_FILE="${RESEARCH_REPLAY_RUN_FILE:-${3:-}}"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
MARKET_L1_BUCKET="${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
CHECK_S3="${RESEARCH_MARKET_L1_COVERAGE_CHECK_S3:-false}"
CHECK_SYMBOLS="${RESEARCH_MARKET_L1_COVERAGE_CHECK_SYMBOLS:-false}"
MAX_S3_WINDOWS="${RESEARCH_MARKET_L1_COVERAGE_MAX_S3_WINDOWS:-200}"
MARKET_L1_REPLAY_WINDOW_MS="${RESEARCH_MARKET_L1_REPLAY_WINDOW_MS:-900000}"

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

positive_integer() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

normalize_bool() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

aws_cmd() {
  aws --region "$REGION" "$@"
}

latest_delta_key_for_window() {
  local window_start_ms="$1"
  aws_cmd s3api list-objects-v2 \
    --bucket "$MARKET_L1_BUCKET" \
    --prefix "market_feature_delta/run_id=l1_${window_start_ms}_" \
    --output json \
  | jq -r '
      (.Contents // [])
      | map(select(.Key | endswith("/delta.json")))
      | sort_by(.Key)
      | last
      | .Key // empty
    '
}

symbol_delta_count_for_key() {
  local key="$1"
  local symbol="$2"
  aws_cmd s3 cp "s3://${MARKET_L1_BUCKET}/${key}" - \
  | jq -s --arg symbol "$symbol" '
      def rows:
        if length == 1 and (.[0] | type) == "array" then .[0] else . end;
      [rows[]? | select(.symbol_canonical == $symbol)] | length
    '
}

require_command date
require_command jq
require_absolute_file "RESEARCH_RETEST_HORIZON_PLAN_FILE or first argument" "$PLAN_FILE"
positive_integer "RESEARCH_MARKET_L1_COVERAGE_MAX_S3_WINDOWS" "$MAX_S3_WINDOWS"
positive_integer "RESEARCH_MARKET_L1_REPLAY_WINDOW_MS" "$MARKET_L1_REPLAY_WINDOW_MS"

check_s3_normalized="$(normalize_bool "$CHECK_S3")"
check_symbols_normalized="$(normalize_bool "$CHECK_SYMBOLS")"
case "$check_s3_normalized" in
  true | false) ;;
  *)
    echo "RESEARCH_MARKET_L1_COVERAGE_CHECK_S3 must be true or false; got $CHECK_S3" >&2
    exit 1
    ;;
esac
case "$check_symbols_normalized" in
  true | false) ;;
  *)
    echo "RESEARCH_MARKET_L1_COVERAGE_CHECK_SYMBOLS must be true or false; got $CHECK_SYMBOLS" >&2
    exit 1
    ;;
esac

if [[ -n "$REPORT_FILE" ]]; then
  require_absolute_file "RESEARCH_REPORT_FILE or second argument" "$REPORT_FILE"
fi
if [[ -n "$REPLAY_RUN_FILE" ]]; then
  require_absolute_file "RESEARCH_REPLAY_RUN_FILE or third argument" "$REPLAY_RUN_FILE"
fi
if [[ "$check_s3_normalized" == "true" ]]; then
  require_command aws
  if [[ -z "$MARKET_L1_BUCKET" || "$MARKET_L1_BUCKET" == *"<"* || "$MARKET_L1_BUCKET" == *">"* ]]; then
    echo "RESEARCH_MARKET_L1_S3_BUCKET or MARKET_L1_BUCKET must be set to a real bucket for S3 checks" >&2
    exit 1
  fi
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

plan_gaps_json="${tmp_dir}/plan-gaps.json"
aggregate_gaps_json="${tmp_dir}/aggregate-gaps.json"
current_missing_json="${tmp_dir}/current-missing-replay-windows.json"
s3_window_plan_json="${tmp_dir}/s3-window-plan.json"
s3_checks_jsonl="${tmp_dir}/s3-checks.jsonl"

jq '
  def unique_sorted: unique | sort;
  [
    .horizon_rows[]?
    | select(.next_action == "extend_market_l1_horizon_coverage")
    | {
        candidate_id,
        primary_symbol,
        hypothesis_type,
        horizon,
        window_start_ms:.forbidden_lookahead_boundary_ms,
        window_end_ms:.horizon_due_ms,
        horizon_market_data_materialized,
        replay_run_count,
        completed_count,
        missing_market_replay_data_count,
        reason_codes
      }
  ] as $rows
  | {
      count:($rows | length),
      by_symbol_horizon:(
        $rows
        | sort_by(.primary_symbol, .horizon)
        | group_by(.primary_symbol + ":" + .horizon)
        | map({
            symbol:.[0].primary_symbol,
            horizon:.[0].horizon,
            count:length,
            missing_market_replay_data_count:(map(.missing_market_replay_data_count // 0) | add // 0)
          })
      ),
      rows:$rows
    }
' "$PLAN_FILE" > "$plan_gaps_json"

if [[ -n "$REPORT_FILE" ]]; then
  jq '
    [
      .partition_aggregates[]?
      | select((.missing_market_replay_data_count // 0) > 0)
      | {
          research_aggregate_key,
          symbol_canonical,
          hypothesis_type,
          replay_run_count,
          completed_count,
          missing_market_replay_data_count,
          gate_reason_codes,
          research_partition_keys
        }
    ] as $rows
    | {
        count:($rows | length),
        rows:$rows
      }
  ' "$REPORT_FILE" > "$aggregate_gaps_json"
else
  jq -n '{count:0, rows:[]}' > "$aggregate_gaps_json"
fi

if [[ -n "$REPLAY_RUN_FILE" ]]; then
  jq -s --argjson window_ms "$MARKET_L1_REPLAY_WINDOW_MS" '
    def aligned($value): (($value / $window_ms) | floor) * $window_ms;
    [
      .[]
      | select(.result_summary.status == "missing_market_replay_data")
      | {
          replay_run_id,
          candidate_id:.source_candidate_id,
          candidate_lifecycle_key:.source_candidate_lifecycle_key,
          research_aggregate_key,
          symbol:.symbol_canonical,
          horizon:(.research_aggregate_key | split(":")[3]),
          window_start_ms,
          window_end_ms,
          reason_codes:.result_summary.reason_codes,
          expected_l1_window_starts:([
            range(aligned(.window_start_ms); aligned(.window_end_ms) + $window_ms; $window_ms)
          ])
        }
    ] as $rows
    | {
        count:($rows | length),
        rows:$rows
      }
  ' "$REPLAY_RUN_FILE" > "$current_missing_json"
else
  jq -n '{count:0, rows:[]}' > "$current_missing_json"
fi

jq '
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
' "$current_missing_json" > "$s3_window_plan_json"

if [[ "$check_s3_normalized" == "true" ]]; then
  checked_count=0
  : > "$s3_checks_jsonl"
  while IFS=$'\t' read -r symbol window_start_ms; do
    checked_count=$((checked_count + 1))
    if (( checked_count > MAX_S3_WINDOWS )); then
      break
    fi

    key="$(latest_delta_key_for_window "$window_start_ms")"
    if [[ -z "$key" ]]; then
      jq -nc \
        --arg symbol "$symbol" \
        --argjson window_start_ms "$window_start_ms" \
        '{symbol:$symbol, window_start_ms:$window_start_ms, delta_key:null, delta_key_present:false, symbol_delta_count:null}' \
        >> "$s3_checks_jsonl"
      continue
    fi

    symbol_delta_count=null
    if [[ "$check_symbols_normalized" == "true" ]]; then
      symbol_delta_count="$(symbol_delta_count_for_key "$key" "$symbol")"
    fi

    jq -nc \
      --arg symbol "$symbol" \
      --argjson window_start_ms "$window_start_ms" \
      --arg key "$key" \
      --argjson symbol_delta_count "$symbol_delta_count" \
      '{
        symbol:$symbol,
        window_start_ms:$window_start_ms,
        delta_key:$key,
        delta_key_present:true,
        symbol_delta_count:$symbol_delta_count
      }' >> "$s3_checks_jsonl"
  done < <(jq -r '.[] | [.symbol, .window_start_ms] | @tsv' "$s3_window_plan_json")
else
  : > "$s3_checks_jsonl"
fi

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg plan_file "$PLAN_FILE" \
  --arg report_file "$REPORT_FILE" \
  --arg replay_run_file "$REPLAY_RUN_FILE" \
  --arg market_l1_bucket "$MARKET_L1_BUCKET" \
  --argjson check_s3 "$([[ "$check_s3_normalized" == "true" ]] && echo true || echo false)" \
  --argjson check_symbols "$([[ "$check_symbols_normalized" == "true" ]] && echo true || echo false)" \
  --argjson max_s3_windows "$MAX_S3_WINDOWS" \
  --slurpfile plan_gaps "$plan_gaps_json" \
  --slurpfile aggregate_gaps "$aggregate_gaps_json" \
  --slurpfile current_missing "$current_missing_json" \
  --slurpfile s3_window_plan "$s3_window_plan_json" \
  --slurpfile s3_checks "$s3_checks_jsonl" \
  '
    ($s3_checks // []) as $checks
    | {
        schema_version:"research_market_l1_coverage_gap_diagnosis_v1",
        generated_at:$generated_at,
        inputs:{
          plan_file:$plan_file,
          report_file:(if $report_file == "" then null else $report_file end),
          replay_run_file:(if $replay_run_file == "" then null else $replay_run_file end),
          market_l1_bucket_set:($market_l1_bucket != ""),
          check_s3:$check_s3,
          check_symbols:$check_symbols,
          max_s3_windows:$max_s3_windows
        },
        plan_gaps:$plan_gaps[0],
        aggregate_gaps:$aggregate_gaps[0],
        current_missing_replay_windows:$current_missing[0],
        s3_window_plan:{
          required_symbol_window_count:($s3_window_plan[0] | length),
          checked_window_count:($checks | length),
          truncated:($check_s3 and (($s3_window_plan[0] | length) > ($checks | length))),
          rows:$s3_window_plan[0]
        },
        s3_checks:{
          checked_window_count:($checks | length),
          missing_delta_key_count:($checks | map(select(.delta_key_present == false)) | length),
          present_delta_key_count:($checks | map(select(.delta_key_present == true)) | length),
          zero_symbol_delta_count:(
            $checks
            | map(select(.symbol_delta_count != null and .symbol_delta_count == 0))
            | length
          ),
          positive_symbol_delta_count:(
            $checks
            | map(select(.symbol_delta_count != null and .symbol_delta_count > 0))
            | length
          ),
          missing_delta_key_rows:($checks | map(select(.delta_key_present == false))),
          zero_symbol_delta_rows:($checks | map(select(.symbol_delta_count != null and .symbol_delta_count == 0))),
          sample_rows:($checks[0:20])
        }
      }
  '

echo "market L1 coverage gap diagnosis completed" >&2
