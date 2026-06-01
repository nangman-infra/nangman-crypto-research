#!/usr/bin/env bash

RESEARCH_BATCH_OUTPUT_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RESEARCH_BATCH_OUTPUT_JQ_DIR="$(cd -- "$RESEARCH_BATCH_OUTPUT_LIB_DIR/../jq" && pwd -P)"

research_batch_output_jq() {
  local name="$1"
  local path="$RESEARCH_BATCH_OUTPUT_JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing research batch output jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

write_empty_research_batch_outputs() {
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg manifest_output "$MANIFEST_OUTPUT" \
    --arg summary_output "$SUMMARY_OUTPUT" \
    --arg region "$REGION" \
    --arg dispatch_mode "$dispatch_mode" \
    --arg universe_mode "$UNIVERSE_MODE" \
    --argjson candidate_read_limit "$CANDIDATE_READ_LIMIT" \
    --argjson max_candidate_bundle_count "$MAX_CANDIDATE_BUNDLE_COUNT" \
    --argjson universe "$(cat "$universe_summary_json")" \
    --argjson candidates "$(cat "$all_candidates_json")" \
    -f "$(research_batch_output_jq research-batch-empty-summary.jq)" > "$SUMMARY_OUTPUT"

  jq -n \
    --arg research_packet_id "$RESEARCH_PACKET_ID" \
    --arg run_scope "$RUN_SCOPE" \
    --argjson max_candidates "$MAX_CANDIDATE_BUNDLE_COUNT" \
    --argjson max_history "$MAX_HISTORICAL_REPLAY_RUN_REF_COUNT" \
    --argjson max_replay "$MAX_REPLAY_RUN_COUNT" \
    -f "$(research_batch_output_jq research-batch-empty-manifest.jq)" > "$MANIFEST_OUTPUT"
}

print_empty_research_batch_summary() {
  echo "manifest_output=$MANIFEST_OUTPUT"
  echo "summary_output=$SUMMARY_OUTPUT"
  echo "selected_candidate_count=0"
  jq -r '
    "scanned_research_eligible_candidate_count=\(.scanned_research_eligible_candidate_count)",
    "current_observed_candidate_count=\(.current_observed_candidate_count)",
    "current_approved_candidate_count=\(.current_approved_candidate_count)",
    "legacy_bundle_approved_candidate_count=\(.legacy_bundle_approved_candidate_count)",
    "horizon_contract_invalid_candidate_count=\(.horizon_contract_invalid_candidate_count)",
    "blocked_reason=\(.blocked_reason)"
  ' "$SUMMARY_OUTPUT"
}

write_research_batch_manifest() {
  jq -n \
    --arg research_packet_id "$RESEARCH_PACKET_ID" \
    --arg run_scope "$RUN_SCOPE" \
    --slurpfile candidates_input "$selected_candidates_json" \
    --slurpfile indexes_input "$historical_index_objects_json" \
    --argjson max_candidates "$MAX_CANDIDATE_BUNDLE_COUNT" \
    --argjson max_history "$MAX_HISTORICAL_REPLAY_RUN_REF_COUNT" \
    --argjson max_replay "$MAX_REPLAY_RUN_COUNT" \
    -f "$(research_batch_output_jq research-batch-manifest.jq)" > "$MANIFEST_OUTPUT"
}

write_research_batch_summary() {
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg manifest_output "$MANIFEST_OUTPUT" \
    --arg summary_output "$SUMMARY_OUTPUT" \
    --arg region "$REGION" \
    --arg dispatch_mode "$dispatch_mode" \
    --arg universe_mode "$UNIVERSE_MODE" \
    --arg run_scope "$RUN_SCOPE" \
    --arg research_packet_id "$RESEARCH_PACKET_ID" \
    --argjson candidate_read_limit "$CANDIDATE_READ_LIMIT" \
    --argjson max_candidate_bundle_count "$MAX_CANDIDATE_BUNDLE_COUNT" \
    --slurpfile universe_input "$universe_summary_json" \
    --slurpfile scanned_candidates_input "$all_candidates_json" \
    --slurpfile candidates_input "$selected_candidates_json" \
    --slurpfile indexes_input "$historical_index_objects_json" \
    -f "$(research_batch_output_jq research-batch-summary.jq)" > "$SUMMARY_OUTPUT"
}

print_research_batch_summary() {
  echo "manifest_output=$MANIFEST_OUTPUT"
  echo "summary_output=$SUMMARY_OUTPUT"
  jq -r '
    "selected_candidate_count=\(.selected_candidate_count)",
    "eligible_candidate_pool_count=\(.eligible_candidate_pool_count)",
    "selected_candidate_limit_reached=\(.selected_candidate_limit_reached)",
    "unselected_eligible_candidate_count=\(.unselected_eligible_candidate_count)",
    "distinct_candidate_symbols=\(.distinct_candidate_symbols | join(","))",
    "allowed_horizons=\(.allowed_horizons | join(","))",
    "universe_mode=\(.universe_mode)",
    "current_observed_candidate_count=\(.current_observed_candidate_count)",
    "current_approved_candidate_count=\(.current_approved_candidate_count)",
    "horizon_contract_invalid_candidate_count=\(.horizon_contract_invalid_candidate_count)",
    "selected_current_approved_candidate_count=\(.selected_current_approved_candidate_count)",
    "selected_horizon_contract_valid_count=\(.selected_horizon_contract_valid_count)",
    "historical_replay_run_index_ref_count=\(.historical_replay_run_index_ref_count)",
    "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed)"
  ' "$SUMMARY_OUTPUT"
  echo
  echo "local validation command:"
  printf 'AWS_PROFILE=<sso-profile> AWS_REGION=%q cargo run -- --input-manifest-file %q --market-l1-s3-bucket %q --output-dir /absolute/path/to/local-research-output\n' \
    "$REGION" "$MANIFEST_OUTPUT" "$market_l1_bucket"
}
