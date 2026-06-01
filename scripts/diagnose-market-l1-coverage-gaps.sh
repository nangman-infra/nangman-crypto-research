#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
JQ_DIR="$SCRIPT_DIR/jq"
PLAN_GAPS_JQ="$JQ_DIR/diagnose-market-l1-coverage-plan-gaps.jq"
AGGREGATE_GAPS_JQ="$JQ_DIR/diagnose-market-l1-coverage-aggregate-gaps.jq"
CURRENT_MISSING_JQ="$JQ_DIR/diagnose-market-l1-coverage-current-missing.jq"
S3_WINDOW_PLAN_JQ="$JQ_DIR/diagnose-market-l1-coverage-s3-window-plan.jq"

# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/market-l1-coverage-gap-validation.sh
source "$SCRIPT_DIR/lib/market-l1-coverage-gap-validation.sh"
# shellcheck source=scripts/lib/market-l1-coverage-gap-s3.sh
source "$SCRIPT_DIR/lib/market-l1-coverage-gap-s3.sh"
# shellcheck source=scripts/lib/market-l1-coverage-gap-s3-checks.sh
source "$SCRIPT_DIR/lib/market-l1-coverage-gap-s3-checks.sh"
# shellcheck source=scripts/lib/market-l1-coverage-gap-output.sh
source "$SCRIPT_DIR/lib/market-l1-coverage-gap-output.sh"

PLAN_FILE="${RESEARCH_RETEST_HORIZON_PLAN_FILE:-${1:-}}"
REPORT_FILE="${RESEARCH_REPORT_FILE:-${2:-}}"
REPLAY_RUN_FILE="${RESEARCH_REPLAY_RUN_FILE:-${3:-}}"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
MARKET_L1_BUCKET="${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
CHECK_S3="${RESEARCH_MARKET_L1_COVERAGE_CHECK_S3:-false}"
CHECK_SYMBOLS="${RESEARCH_MARKET_L1_COVERAGE_CHECK_SYMBOLS:-false}"
MAX_S3_WINDOWS="${RESEARCH_MARKET_L1_COVERAGE_MAX_S3_WINDOWS:-200}"
MARKET_L1_REPLAY_WINDOW_MS="${RESEARCH_MARKET_L1_REPLAY_WINDOW_MS:-900000}"

require_command date
require_command jq
require_absolute_file "RESEARCH_RETEST_HORIZON_PLAN_FILE or first argument" "$PLAN_FILE"
require_jq_programs
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
  verify_aws_access >&2
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

plan_gaps_json="${tmp_dir}/plan-gaps.json"
aggregate_gaps_json="${tmp_dir}/aggregate-gaps.json"
current_missing_json="${tmp_dir}/current-missing-replay-windows.json"
s3_window_plan_json="${tmp_dir}/s3-window-plan.json"
s3_checks_jsonl="${tmp_dir}/s3-checks.jsonl"

jq -f "$PLAN_GAPS_JQ" "$PLAN_FILE" > "$plan_gaps_json"

if [[ -n "$REPORT_FILE" ]]; then
  jq -f "$AGGREGATE_GAPS_JQ" "$REPORT_FILE" > "$aggregate_gaps_json"
else
  jq -n '{count:0, rows:[]}' > "$aggregate_gaps_json"
fi

if [[ -n "$REPLAY_RUN_FILE" ]]; then
  jq -s --argjson window_ms "$MARKET_L1_REPLAY_WINDOW_MS" \
    -f "$CURRENT_MISSING_JQ" "$REPLAY_RUN_FILE" > "$current_missing_json"
else
  jq -n '{count:0, rows:[]}' > "$current_missing_json"
fi

jq -f "$S3_WINDOW_PLAN_JQ" "$current_missing_json" > "$s3_window_plan_json"

if [[ "$check_s3_normalized" == "true" ]]; then
  write_market_l1_s3_checks "$s3_checks_jsonl" "$s3_window_plan_json"
else
  : > "$s3_checks_jsonl"
fi

emit_market_l1_coverage_gap_diagnosis

echo "market L1 coverage gap diagnosis completed" >&2
