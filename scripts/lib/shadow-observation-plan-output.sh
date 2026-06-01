#!/usr/bin/env bash

write_shadow_observation_plan() {
  local horizon_status_input="$1"
  local latest_l1_source="$2"
  local jq_program="${SHADOW_OBSERVATION_PLAN_OUTPUT_JQ:-$SCRIPT_DIR/jq/shadow-observation-plan-output.jq}"
  local jq_dir

  if [[ ! -f "$jq_program" ]]; then
    echo "missing shadow observation plan jq program: $jq_program" >&2
    exit 1
  fi
  jq_dir="$(cd -- "$(dirname -- "$jq_program")" && pwd -P)"

  jq -s \
    -L "$jq_dir" \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --argjson generated_at_ms "$(date -u +%s)000" \
    --arg shadow_validation_run_file "$SHADOW_VALIDATION_RUN_FILE" \
    --arg horizon_status_file "$HORIZON_STATUS_FILE" \
    --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
    --arg latest_l1_source "$latest_l1_source" \
    --slurpfile horizon_status_input "$horizon_status_input" \
    -f "$jq_program" \
    "$SHADOW_VALIDATION_RUN_FILE"
}
