#!/usr/bin/env bash

RESEARCH_CURRENT_BATCH_OUTPUT_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RESEARCH_CURRENT_BATCH_OUTPUT_JQ_DIR="$(cd -- "$RESEARCH_CURRENT_BATCH_OUTPUT_LIB_DIR/../jq" && pwd -P)"

research_current_batch_output_jq() {
  local name="$1"
  local path="$RESEARCH_CURRENT_BATCH_OUTPUT_JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing research current batch jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

write_current_approved_batch_driver_summary() {
  local report_file="$1"
  local registry_file="$2"

  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg run_id "$RUN_ID" \
    --arg run_dir "$RUN_DIR" \
    --arg manifest_file "$MANIFEST_OUTPUT" \
    --arg manifest_summary_file "$MANIFEST_SUMMARY_OUTPUT" \
    --arg research_output_dir "$RESEARCH_OUTPUT_DIR" \
    --arg report_file "$report_file" \
    --arg registry_file "$registry_file" \
    --arg report_summary_file "$REPORT_SUMMARY_OUTPUT" \
    --arg retest_horizon_plan_file "$RETEST_HORIZON_PLAN_OUTPUT" \
    --arg retest_horizon_status_file "$RETEST_HORIZON_STATUS_OUTPUT" \
    --slurpfile manifest_summary_file_input "$MANIFEST_SUMMARY_OUTPUT" \
    --slurpfile report_summary_file_input "$REPORT_SUMMARY_OUTPUT" \
    --slurpfile retest_horizon_plan_file_input "$RETEST_HORIZON_PLAN_OUTPUT" \
    -f "$(research_current_batch_output_jq research-current-batch-driver-summary.jq)" \
    > "$DRIVER_SUMMARY_OUTPUT"
}

print_current_approved_report_summary() {
  jq -r '
    "report_status=\(.report.research_run_status)",
    "source_candidate_count=\(.report.source_candidate_count)",
    "replay_run_count=\(.report.replay_run_count)",
    "retest_candidate_count=\(.report.retest_candidate_count)",
    "surviving_candidate_count=\(.report.surviving_candidate_count)",
    "shadow_validation_count=\(.report.shadow_validation_count)",
    "paper_trade_candidate_count=\(.report.paper_trade_candidate_count)",
    "promotion_passed=\(.stage_state.promotion_passed)",
    "shadow_created=\(.stage_state.shadow_created)",
    "paper_created=\(.stage_state.paper_created)"
  ' "$REPORT_SUMMARY_OUTPUT" | redact
}

print_current_approved_retest_horizon_plan_summary() {
  jq -r '
    "horizon_count=\(.summary.horizon_count)",
    "ready_for_replay_count=\(.summary.ready_for_replay_count)",
    "waiting_for_market_l1_count=\(.summary.waiting_for_market_l1_count)",
    "market_l1_coverage_extension_count=\(.summary.market_l1_coverage_extension_count)",
    "sample_accumulation_count=\(.summary.sample_accumulation_count)",
    "promotion_ready_for_review_count=\(.summary.promotion_ready_for_review_count)",
    "next_action_counts=\(.summary.next_action_counts | map(.next_action + ":" + (.count|tostring)) | join(","))"
  ' "$RETEST_HORIZON_PLAN_OUTPUT" | redact
}

print_current_approved_retest_horizon_status_summary() {
  jq -r '
    "horizon_status_verdict=\(.next_decision.verdict)",
    "major50_observed_symbol_count=\(.major50_state.observed_symbol_count)",
    "major50_approved_symbol_count=\(.major50_state.approved_symbol_count)",
    "research_factory_blocking_stage=\(.research_factory_gap_summary.blocking_stage)",
    "approved_symbols_without_candidate_count=\(.research_factory_gap_summary.gap_counts.approved_symbols_without_candidate)",
    "approved_symbols_without_selected_candidate_count=\(.research_factory_gap_summary.gap_counts.approved_symbols_without_selected_candidate)",
    "unselected_eligible_candidate_symbol_count=\(.research_factory_gap_summary.gap_counts.unselected_eligible_candidate_symbols)",
    "candidate_count=\(.horizon_summary.candidate_count)",
    "horizon_count=\(.horizon_summary.horizon_count)",
    "symbols=\(.horizon_summary.symbols | join(","))",
    "market_l1_coverage_extension_count=\(.horizon_summary.market_l1_coverage_extension_count)",
    "next_action_counts=\(.horizon_summary.next_action_counts | map(.next_action + ":" + (.count|tostring)) | join(","))",
    "blocked_actions=\(.next_decision.blocked_actions | join(","))"
  ' "$RETEST_HORIZON_STATUS_OUTPUT" | redact
}

print_current_approved_batch_driver_result() {
  {
    echo "batch_driver_summary=$DRIVER_SUMMARY_OUTPUT"
    echo "retest_horizon_status=$RETEST_HORIZON_STATUS_OUTPUT"
    jq -r '
      "selected_candidate_count=\(.manifest.selected_candidate_count)",
      "eligible_candidate_pool_count=\(.manifest.eligible_candidate_pool_count)",
      "selected_candidate_limit_reached=\(.manifest.selected_candidate_limit_reached)",
      "unselected_eligible_candidate_count=\(.manifest.unselected_eligible_candidate_count)",
      "current_approved_candidate_count=\(.manifest.current_approved_candidate_count)",
      "horizon_contract_invalid_candidate_count=\(.manifest.horizon_contract_invalid_candidate_count)",
      "distinct_candidate_symbols=\(.manifest.distinct_candidate_symbols | join(","))",
      "research_replay_completed=\(.stage_state.research_replay_completed)",
      "promotion_passed=\(.stage_state.promotion_passed)",
      "shadow_created=\(.stage_state.shadow_created)",
      "paper_created=\(.stage_state.paper_created)",
      "live_enabled=\(.stage_state.live_enabled)",
      "promotion_ready_for_review_count=\(.retest_horizon_plan_summary.promotion_ready_for_review_count)"
    ' "$DRIVER_SUMMARY_OUTPUT"
    jq -r '
      "research_factory_blocking_stage=\(.research_factory_gap_summary.blocking_stage)",
      "approved_symbols_without_candidate_count=\(.research_factory_gap_summary.gap_counts.approved_symbols_without_candidate)",
      "approved_symbols_without_selected_candidate_count=\(.research_factory_gap_summary.gap_counts.approved_symbols_without_selected_candidate)",
      "unselected_eligible_candidate_symbol_count=\(.research_factory_gap_summary.gap_counts.unselected_eligible_candidate_symbols)",
      "candidate_ids_without_replay_count=\(.research_factory_gap_summary.gap_counts.candidate_ids_without_replay)"
    ' "$RETEST_HORIZON_STATUS_OUTPUT"
    echo "research current-approved batch driver completed"
  } | redact
}
