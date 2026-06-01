#!/usr/bin/env bash

write_shadow_sample_accumulation_cycle_summary() {
  local output_file="$1"
  local accumulation_created="$2"
  local shadow_files_json_file="$3"
  local accumulation_summary_json="null"
  local jq_program="$script_dir/jq/shadow-sample-accumulation-cycle-summary.jq"

  if [[ "$accumulation_created" == true ]]; then
    accumulation_summary_json="$(cat "$ACCUMULATION_SUMMARY_FILE")"
  fi

  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg run_dir "$RUN_DIR" \
    --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
    --arg horizon_status_file "$HORIZON_STATUS_FILE" \
    --arg accumulation_blocked_reason "$accumulation_blocked_reason" \
    --arg merged_shadow_file "$MERGED_SHADOW_FILE" \
    --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
    --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
    --arg accumulation_manifest_file "$ACCUMULATION_MANIFEST_FILE" \
    --arg accumulation_summary_file "$ACCUMULATION_SUMMARY_FILE" \
    --arg cycle_summary_file "$CYCLE_SUMMARY_FILE" \
    --arg decision_file "$DECISION_FILE" \
    --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
    --argjson accumulation_created "$accumulation_created" \
    --argjson accumulation_summary "$accumulation_summary_json" \
    --argjson shadow_input_files "$(cat "$shadow_files_json_file")" \
    --slurpfile merge_summary "$MERGED_SHADOW_SUMMARY_FILE" \
    --slurpfile observation "$OBSERVATION_PLAN_FILE" \
    --slurpfile gap "$GAP_MANIFEST_FILE" \
    -f "$jq_program" > "$output_file"
}

write_shadow_cycle_decision() {
  local output_file="$1"
  local jq_program="$script_dir/jq/shadow-cycle-decision.jq"

  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --slurpfile cycle "$CYCLE_SUMMARY_FILE" \
    -f "$jq_program" > "$output_file"
}

print_shadow_sample_accumulation_cycle_result() {
  jq -r '
    "cycle_summary=\(.cycle_summary_file)",
    "decision_file=\(.decision_file)",
    "shadow_input_file_count=\(.shadow_input_files | length)",
    "merged_record_count=\(.merge_summary.merged_record_count)",
    "target_window_materialized_count=\(.observation_summary.target_window_materialized_count)",
    "total_sample_deficit=\(.gap_summary.total_sample_deficit)",
    "next_verdict=\(.next_decision.verdict)",
    "next_observation_not_before_ms=\(.next_decision.next_observation_not_before_ms // "none")",
    "accumulation_manifest_file=\(.accumulation_manifest_file // "not_created")",
    "blocked_actions=\(.next_decision.blocked_actions | join(","))",
    "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),shadow_status_mutated:\(.safety.shadow_status_mutated),paper_live_enabled:\(.safety.paper_live_enabled)"
  ' "$CYCLE_SUMMARY_FILE"

  jq -r '
    "scheduler_action=\(.scheduler_action)",
    "run_not_before_ms=\(.run_not_before_ms // "none")",
    "focused_research_manifest_file=\(.focused_research_manifest_file // "not_created")"
  ' "$DECISION_FILE"
}
