#!/usr/bin/env bash
set -euo pipefail

SHADOW_VALIDATION_RUN_FILE="${RESEARCH_SHADOW_VALIDATION_RUN_FILE:-${1:-}}"
HORIZON_STATUS_FILE="${RESEARCH_RETEST_HORIZON_STATUS_FILE:-${2:-}}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JQ_PROGRAM="$SCRIPT_DIR/jq/summarize-shadow-validation-status.jq"

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

require_absolute_file "shadow validation status jq program" "$JQ_PROGRAM"
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
  -L "$SCRIPT_DIR/jq" \
  -f "$JQ_PROGRAM" "$SHADOW_VALIDATION_RUN_FILE"

echo "research shadow validation status summary completed" >&2
