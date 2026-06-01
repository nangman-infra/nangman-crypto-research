#!/usr/bin/env bash
set -euo pipefail

SHADOW_VALIDATION_RUN_FILE="${RESEARCH_SHADOW_VALIDATION_RUN_FILE:-${1:-}}"
HORIZON_STATUS_FILE="${RESEARCH_RETEST_HORIZON_STATUS_FILE:-${2:-}}"
LATEST_L1_AS_OF_MS="${RESEARCH_SHADOW_OBSERVATION_LATEST_L1_AS_OF_MS:-${3:-}}"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
MARKET_L1_BUCKET="${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=scripts/lib/shadow-observation-plan-output.sh
source "$SCRIPT_DIR/lib/shadow-observation-plan-output.sh"

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

write_shadow_observation_plan "$horizon_status_input" "$latest_l1_source"

echo "research shadow observation plan completed" >&2
