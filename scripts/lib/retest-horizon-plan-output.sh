#!/usr/bin/env bash

write_retest_horizon_plan() {
  local report_file="$1"
  local bundles_json_file="$2"
  local jq_program="$SCRIPT_DIR/jq/retest-horizon-plan.jq"

  jq \
    -L "$SCRIPT_DIR/jq" \
    --arg manifest_file "$MANIFEST_FILE" \
    --arg report_file "$REPORT_FILE" \
    --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
    --slurpfile bundles_file "$bundles_json_file" \
    -f "$jq_program" "$report_file"
}
