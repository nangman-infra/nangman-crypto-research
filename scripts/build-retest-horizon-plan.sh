#!/usr/bin/env bash
set -euo pipefail

MANIFEST_FILE="${RESEARCH_RETEST_PLAN_MANIFEST_FILE:-${1:-}}"
REPORT_FILE="${RESEARCH_RETEST_PLAN_REPORT_FILE:-${2:-}}"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
MARKET_L1_BUCKET="${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
LATEST_L1_AS_OF_MS="${RESEARCH_RETEST_PLAN_LATEST_L1_AS_OF_MS:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=scripts/lib/retest-horizon-plan-output.sh
source "$SCRIPT_DIR/lib/retest-horizon-plan-output.sh"

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
write_retest_horizon_plan "$REPORT_FILE" "$bundles_json_file"

echo "research retest horizon plan completed" >&2
