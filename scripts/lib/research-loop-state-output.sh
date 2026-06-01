#!/usr/bin/env bash

RESEARCH_LOOP_STATE_OUTPUT_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RESEARCH_LOOP_STATE_OUTPUT_JQ_DIR="$(cd -- "$RESEARCH_LOOP_STATE_OUTPUT_LIB_DIR/../jq" && pwd -P)"

research_loop_state_output_jq() {
  local name="$1"
  local path="$RESEARCH_LOOP_STATE_OUTPUT_JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing research loop state output jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

select_current_approved_shard_batch_summary() {
  local report_records_file="$1"
  jq -s -c -f "$(research_loop_state_output_jq research-loop-current-approved-shard-batch-summary.jq)" "$report_records_file"
}

summarize_recent_research_report_coverage() {
  local report_records_file="$1"
  jq -s -c -f "$(research_loop_state_output_jq research-loop-recent-report-coverage-summary.jq)" "$report_records_file"
}

select_research_evidence_summary() {
  local latest_report_summary="$1"
  local current_approved_shard_batch_summary="$2"
  jq -n -c \
    --argjson latest "$latest_report_summary" \
    --argjson shard_batch "$current_approved_shard_batch_summary" \
    -f "$(research_loop_state_output_jq research-loop-research-evidence-summary.jq)"
}

build_research_loop_prefix_summary() {
  local report_object="$1"
  local replay_object="$2"
  local index_object="$3"
  local shadow_object="$4"
  local paper_object="$5"
  jq -n -c \
    --argjson report "$report_object" \
    --argjson replay "$replay_object" \
    --argjson index "$index_object" \
    --argjson shadow "$shadow_object" \
    --argjson paper "$paper_object" \
    -f "$(research_loop_state_output_jq research-loop-prefix-summary.jq)"
}

emit_research_loop_state_report() {
  jq -n \
    --arg region "$REGION" \
    --arg candidate_bucket "$candidate_bucket" \
    --arg market_l1_bucket "$market_l1_bucket" \
    --arg output_bucket "$output_bucket" \
    --argjson runtime "$runtime_summary" \
    --argjson universe "$universe_summary" \
    --argjson candidates "$candidate_summary" \
    --argjson report "$report_summary" \
    --argjson current_approved_shard_batch "$current_approved_shard_batch_summary" \
    --argjson recent_report_coverage "$recent_research_report_coverage_summary" \
    --argjson research "$research_evidence_summary" \
    --argjson prefixes "$prefix_summary" \
    -f "$(research_loop_state_output_jq research-loop-state-report.jq)"
}
