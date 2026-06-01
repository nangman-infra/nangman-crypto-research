#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
JQ_PROGRAM="$SCRIPT_DIR/jq/shadow-sample-gap-manifest.jq"
OBSERVATION_PLAN_FILE="${RESEARCH_SHADOW_OBSERVATION_PLAN_FILE:-${1:-}}"

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

require_absolute_file "RESEARCH_SHADOW_OBSERVATION_PLAN_FILE or first argument" "$OBSERVATION_PLAN_FILE"
require_absolute_file "shadow sample gap manifest jq program" "$JQ_PROGRAM"

jq \
  -L "$SCRIPT_DIR/jq" \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson generated_at_ms "$(date -u +%s)000" \
  --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
  -f "$JQ_PROGRAM" \
  "$OBSERVATION_PLAN_FILE"

echo "research shadow sample gap manifest completed" >&2
