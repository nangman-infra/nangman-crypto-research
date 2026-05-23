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

redact() {
  sed -E \
    -e 's/nangman-crypto-dev-[A-Za-z0-9-]+-[0-9]{6}/nangman-crypto-dev-<bucket-family>-<account-suffix>/g' \
    -e 's/[0-9]{12}\.dkr\.ecr/<aws-account-id>.dkr.ecr/g' \
    -e 's/account=[0-9]{12}/account=<aws-account-id>/g' \
    -e 's/"Account"[[:space:]]*:[[:space:]]*"[0-9]{12}"/"Account":"<aws-account-id>"/g' \
    -e 's/[0-9]{12}/<aws-account-id>/g' \
    -e 's#arn:aws:iam::[^[:space:]"]+#arn:aws:iam::<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:ecs:[^[:space:]"]+#arn:aws:ecs:<region>:<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:lambda:[^[:space:]"]+#arn:aws:lambda:<region>:<aws-account-id>:<resource>#g' \
    -e 's/subnet-[A-Za-z0-9]+/<subnet-id>/g' \
    -e 's/sg-[A-Za-z0-9]+/<security-group-id>/g'
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

verify_aws_access() {
  local identity_output
  if ! identity_output="$(aws_cmd sts get-caller-identity --output json 2>&1)"; then
    {
      echo "AWS credentials unavailable or expired for region=$REGION"
      echo "Refresh the AWS login/session, then rerun this check."
      echo "$identity_output"
    } | redact >&2
    exit 1
  fi

  echo "aws identity ok: account=$(jq -r '.Account' <<<"$identity_output")" | redact >&2
}

latest_direct_key_for_window() {
  local window_start_ms="$1"
  local family_prefix="$2"
  local file_suffix="$3"
  aws_cmd s3api list-objects-v2 \
    --bucket "$MARKET_L1_BUCKET" \
    --prefix "${family_prefix}/run_id=l1_${window_start_ms}_" \
    --output json \
  | jq -r --arg file_suffix "$file_suffix" '
      (.Contents // [])
      | map(select(.Key | endswith($file_suffix)))
      | sort_by(.Key)
      | last
      | .Key // empty
    '
}

latest_delta_key_for_window() {
  local window_start_ms="$1"
  latest_direct_key_for_window "$window_start_ms" "market_feature_delta" "/delta.json"
}

latest_regime_context_key_for_window() {
  local window_start_ms="$1"
  latest_direct_key_for_window "$window_start_ms" "market_regime_context" "/context.json"
}

s3_object_exists() {
  local key="$1"
  aws_cmd s3api head-object --bucket "$MARKET_L1_BUCKET" --key "$key" >/dev/null 2>&1
}

normalize_s3_key() {
  local value="$1"
  value="${value#/}"
  if [[ "$value" == s3://* ]]; then
    value="${value#s3://}"
    value="${value#*/}"
  fi
  printf '%s' "$value"
}

l1_index_pointer_key_for_window() {
  local window_start_ms="$1"
  local event_date
  local hour
  IFS=$'\t' read -r event_date hour < <(
    jq -nr --argjson window_start_ms "$window_start_ms" '
      (($window_start_ms / 1000) | floor | gmtime)
      | [strftime("%Y-%m-%d"), strftime("%H")]
      | @tsv
    '
  )
  printf 'l1_index/window_ms=1000/event_date=%s/hour=%s/window_start_ms=%s.json' \
    "$event_date" "$hour" "$window_start_ms"
}

manifest_key_from_l1_index_pointer() {
  local pointer_key="$1"
  aws_cmd s3 cp "s3://${MARKET_L1_BUCKET}/${pointer_key}" - \
  | jq -r '
      select(.schema_version == "l1_index_pointer_v1")
      | select((.status // "" | ascii_downcase) == "success")
      | (.canonical_manifest_key // .manifest_key // empty)
    ' \
  | while IFS= read -r key; do
      normalize_s3_key "$key"
    done
}

artifact_key_from_l1_manifest() {
  local manifest_key="$1"
  local manifest_field="$2"
  aws_cmd s3 cp "s3://${MARKET_L1_BUCKET}/${manifest_key}" - \
  | jq -r --arg manifest_field "$manifest_field" '
      select(.schema_version == "l1_manifest_v1")
      | select((.status // "" | ascii_downcase) == "success")
      | (.[$manifest_field] // empty)
    ' \
  | while IFS= read -r key; do
      normalize_s3_key "$key"
    done
}

feature_delta_key_from_l1_manifest() {
  local manifest_key="$1"
  artifact_key_from_l1_manifest "$manifest_key" "market_feature_delta_key"
}

regime_context_key_from_l1_manifest() {
  local manifest_key="$1"
  artifact_key_from_l1_manifest "$manifest_key" "market_regime_context_key"
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
  require_command sed
  if [[ -z "$MARKET_L1_BUCKET" || "$MARKET_L1_BUCKET" == *"<"* || "$MARKET_L1_BUCKET" == *">"* ]]; then
    echo "RESEARCH_MARKET_L1_S3_BUCKET or MARKET_L1_BUCKET must be set to a real bucket for S3 checks" >&2
    exit 1
  fi
  verify_aws_access
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
    direct_delta_key_present=false
    if [[ -n "$key" ]]; then
      direct_delta_key_present=true
    fi

    regime_key="$(latest_regime_context_key_for_window "$window_start_ms")"
    direct_regime_context_key_present=false
    if [[ -n "$regime_key" ]]; then
      direct_regime_context_key_present=true
    fi

    index_pointer_key="$(l1_index_pointer_key_for_window "$window_start_ms")"
    index_pointer_present=false
    manifest_key=""
    manifest_present=false
    manifest_delta_key=""
    manifest_delta_key_present=false
    manifest_regime_context_key=""
    manifest_regime_context_key_present=false
    if s3_object_exists "$index_pointer_key"; then
      index_pointer_present=true
      manifest_key="$(manifest_key_from_l1_index_pointer "$index_pointer_key" | sed -n '1p')"
      if [[ -n "$manifest_key" ]] && s3_object_exists "$manifest_key"; then
        manifest_present=true
        manifest_delta_key="$(feature_delta_key_from_l1_manifest "$manifest_key" | sed -n '1p')"
        if [[ -n "$manifest_delta_key" ]] && s3_object_exists "$manifest_delta_key"; then
          manifest_delta_key_present=true
        fi
        manifest_regime_context_key="$(regime_context_key_from_l1_manifest "$manifest_key" | sed -n '1p')"
        if [[ -n "$manifest_regime_context_key" ]] && s3_object_exists "$manifest_regime_context_key"; then
          manifest_regime_context_key_present=true
        fi
      fi
    fi

    discoverable_delta_key="$key"
    discoverable_delta_key_present="$direct_delta_key_present"
    if [[ "$discoverable_delta_key_present" != "true" && "$manifest_delta_key_present" == "true" ]]; then
      discoverable_delta_key="$manifest_delta_key"
      discoverable_delta_key_present=true
    fi

    discoverable_regime_context_key="$regime_key"
    discoverable_regime_context_key_present="$direct_regime_context_key_present"
    if [[ "$discoverable_regime_context_key_present" != "true" && "$manifest_regime_context_key_present" == "true" ]]; then
      discoverable_regime_context_key="$manifest_regime_context_key"
      discoverable_regime_context_key_present=true
    fi

    if [[ -z "$discoverable_delta_key" ]]; then
      jq -nc \
        --arg symbol "$symbol" \
        --argjson window_start_ms "$window_start_ms" \
        --arg regime_key "$regime_key" \
        --arg index_pointer_key "$index_pointer_key" \
        --arg manifest_key "$manifest_key" \
        --arg manifest_delta_key "$manifest_delta_key" \
        --arg manifest_regime_context_key "$manifest_regime_context_key" \
        --arg discoverable_regime_context_key "$discoverable_regime_context_key" \
        --argjson direct_delta_key_present "$direct_delta_key_present" \
        --argjson direct_regime_context_key_present "$direct_regime_context_key_present" \
        --argjson index_pointer_present "$index_pointer_present" \
        --argjson manifest_present "$manifest_present" \
        --argjson manifest_delta_key_present "$manifest_delta_key_present" \
        --argjson manifest_regime_context_key_present "$manifest_regime_context_key_present" \
        --argjson discoverable_delta_key_present "$discoverable_delta_key_present" \
        --argjson discoverable_regime_context_key_present "$discoverable_regime_context_key_present" \
        '{
          symbol:$symbol,
          window_start_ms:$window_start_ms,
          delta_key:null,
          delta_key_present:$direct_delta_key_present,
          regime_context_key:(if $regime_key == "" then null else $regime_key end),
          regime_context_key_present:$direct_regime_context_key_present,
          index_pointer_key:$index_pointer_key,
          index_pointer_present:$index_pointer_present,
          manifest_key:(if $manifest_key == "" then null else $manifest_key end),
          manifest_present:$manifest_present,
          manifest_delta_key:(if $manifest_delta_key == "" then null else $manifest_delta_key end),
          manifest_delta_key_present:$manifest_delta_key_present,
          manifest_regime_context_key:(if $manifest_regime_context_key == "" then null else $manifest_regime_context_key end),
          manifest_regime_context_key_present:$manifest_regime_context_key_present,
          discoverable_delta_key:null,
          discoverable_delta_key_present:$discoverable_delta_key_present,
          discoverable_regime_context_key:(if $discoverable_regime_context_key == "" then null else $discoverable_regime_context_key end),
          discoverable_regime_context_key_present:$discoverable_regime_context_key_present,
          symbol_delta_count:null
        }' \
        >> "$s3_checks_jsonl"
      continue
    fi

    symbol_delta_count=null
    if [[ "$check_symbols_normalized" == "true" ]]; then
      symbol_delta_count="$(symbol_delta_count_for_key "$discoverable_delta_key" "$symbol")"
    fi

    jq -nc \
      --arg symbol "$symbol" \
      --argjson window_start_ms "$window_start_ms" \
      --arg key "$key" \
      --arg regime_key "$regime_key" \
      --arg index_pointer_key "$index_pointer_key" \
      --arg manifest_key "$manifest_key" \
      --arg manifest_delta_key "$manifest_delta_key" \
      --arg manifest_regime_context_key "$manifest_regime_context_key" \
      --arg discoverable_delta_key "$discoverable_delta_key" \
      --arg discoverable_regime_context_key "$discoverable_regime_context_key" \
      --argjson direct_delta_key_present "$direct_delta_key_present" \
      --argjson direct_regime_context_key_present "$direct_regime_context_key_present" \
      --argjson index_pointer_present "$index_pointer_present" \
      --argjson manifest_present "$manifest_present" \
      --argjson manifest_delta_key_present "$manifest_delta_key_present" \
      --argjson manifest_regime_context_key_present "$manifest_regime_context_key_present" \
      --argjson discoverable_delta_key_present "$discoverable_delta_key_present" \
      --argjson discoverable_regime_context_key_present "$discoverable_regime_context_key_present" \
      --argjson symbol_delta_count "$symbol_delta_count" \
      '{
        symbol:$symbol,
        window_start_ms:$window_start_ms,
        delta_key:(if $key == "" then null else $key end),
        delta_key_present:$direct_delta_key_present,
        regime_context_key:(if $regime_key == "" then null else $regime_key end),
        regime_context_key_present:$direct_regime_context_key_present,
        index_pointer_key:$index_pointer_key,
        index_pointer_present:$index_pointer_present,
        manifest_key:(if $manifest_key == "" then null else $manifest_key end),
        manifest_present:$manifest_present,
        manifest_delta_key:(if $manifest_delta_key == "" then null else $manifest_delta_key end),
        manifest_delta_key_present:$manifest_delta_key_present,
        manifest_regime_context_key:(if $manifest_regime_context_key == "" then null else $manifest_regime_context_key end),
        manifest_regime_context_key_present:$manifest_regime_context_key_present,
        discoverable_delta_key:$discoverable_delta_key,
        discoverable_delta_key_present:$discoverable_delta_key_present,
        discoverable_regime_context_key:(if $discoverable_regime_context_key == "" then null else $discoverable_regime_context_key end),
        discoverable_regime_context_key_present:$discoverable_regime_context_key_present,
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
          missing_regime_context_key_count:($checks | map(select(.regime_context_key_present == false)) | length),
          present_regime_context_key_count:($checks | map(select(.regime_context_key_present == true)) | length),
          present_index_pointer_count:($checks | map(select(.index_pointer_present == true)) | length),
          present_manifest_count:($checks | map(select(.manifest_present == true)) | length),
          present_manifest_delta_key_count:($checks | map(select(.manifest_delta_key_present == true)) | length),
          present_manifest_regime_context_key_count:($checks | map(select(.manifest_regime_context_key_present == true)) | length),
          missing_discoverable_delta_key_count:(
            $checks
            | map(select(.discoverable_delta_key_present == false))
            | length
          ),
          present_discoverable_delta_key_count:(
            $checks
            | map(select(.discoverable_delta_key_present == true))
            | length
          ),
          missing_discoverable_regime_context_key_count:(
            $checks
            | map(select(.discoverable_regime_context_key_present == false))
            | length
          ),
          present_discoverable_regime_context_key_count:(
            $checks
            | map(select(.discoverable_regime_context_key_present == true))
            | length
          ),
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
          missing_regime_context_key_rows:($checks | map(select(.regime_context_key_present == false))),
          missing_discoverable_delta_key_rows:($checks | map(select(.discoverable_delta_key_present == false))),
          missing_discoverable_regime_context_key_rows:($checks | map(select(.discoverable_regime_context_key_present == false))),
          zero_symbol_delta_rows:($checks | map(select(.symbol_delta_count != null and .symbol_delta_count == 0))),
          sample_rows:($checks[0:20])
        }
      }
  '

echo "market L1 coverage gap diagnosis completed" >&2
