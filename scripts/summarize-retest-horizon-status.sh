#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
JQ_DIR="$SCRIPT_DIR/jq"
JQ_PROGRAM="$JQ_DIR/summarize-retest-horizon-status.jq"

PLAN_FILE="${RESEARCH_RETEST_HORIZON_PLAN_FILE:-${1:-}}"
DRIVER_SUMMARY_FILE="${RESEARCH_BATCH_DRIVER_SUMMARY_FILE:-${2:-}}"

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

require_command date
require_command jq
require_absolute_file "retest horizon status jq program" "$JQ_PROGRAM"
require_absolute_file "RESEARCH_RETEST_HORIZON_PLAN_FILE or first argument" "$PLAN_FILE"

if [[ -n "$DRIVER_SUMMARY_FILE" ]]; then
  require_absolute_file "RESEARCH_BATCH_DRIVER_SUMMARY_FILE or second argument" "$DRIVER_SUMMARY_FILE"
  driver_manifest_summary_file="$(jq -r '.manifest_summary_file // empty' "$DRIVER_SUMMARY_FILE")"
  if [[ -n "$driver_manifest_summary_file" && -f "$driver_manifest_summary_file" ]]; then
    DRIVER_MANIFEST_SUMMARY_FILE="$driver_manifest_summary_file"
  else
    DRIVER_MANIFEST_SUMMARY_FILE=""
  fi
else
  DRIVER_MANIFEST_SUMMARY_FILE=""
fi

driver_summary_input="$(mktemp)"
driver_manifest_summary_input="$(mktemp)"
trap 'rm -f "$driver_summary_input" "$driver_manifest_summary_input"' EXIT

if [[ -n "$DRIVER_SUMMARY_FILE" ]]; then
  cp "$DRIVER_SUMMARY_FILE" "$driver_summary_input"
else
  printf 'null\n' > "$driver_summary_input"
fi

if [[ -n "$DRIVER_MANIFEST_SUMMARY_FILE" ]]; then
  cp "$DRIVER_MANIFEST_SUMMARY_FILE" "$driver_manifest_summary_input"
else
  printf 'null\n' > "$driver_manifest_summary_input"
fi

jq \
  -L "$JQ_DIR" \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg plan_file "$PLAN_FILE" \
  --arg driver_summary_file "$DRIVER_SUMMARY_FILE" \
  --slurpfile driver_summary_input "$driver_summary_input" \
  --slurpfile driver_manifest_summary_input "$driver_manifest_summary_input" \
  -f "$JQ_PROGRAM" \
  "$PLAN_FILE"

echo "research retest horizon status summary completed" >&2
