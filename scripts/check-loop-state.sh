#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/research-loop-state-runtime.sh
source "$SCRIPT_DIR/lib/research-loop-state-runtime.sh"
# shellcheck source=scripts/lib/research-loop-state-s3.sh
source "$SCRIPT_DIR/lib/research-loop-state-s3.sh"
# shellcheck source=scripts/lib/research-loop-state-output.sh
source "$SCRIPT_DIR/lib/research-loop-state-output.sh"

REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
CANDIDATE_READ_LIMIT="${RESEARCH_LOOP_STATE_CANDIDATE_READ_LIMIT:-1000}"
REPORT_READ_LIMIT="${RESEARCH_LOOP_STATE_REPORT_READ_LIMIT:-100}"
EXPECTED_MAJOR_UNIVERSE_SIZE="${RESEARCH_EXPECTED_MAJOR_UNIVERSE_SIZE:-50}"

require_command aws
require_command jq
require_command sed
require_command mktemp

echo "== research loop state =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "task_definition=$TASK_DEFINITION"
echo "candidate_read_limit=$CANDIDATE_READ_LIMIT"
echo "report_read_limit=$REPORT_READ_LIMIT"
echo

verify_aws_access

prepare_research_loop_state_tmp_files
fetch_research_loop_runtime_documents
resolve_research_loop_runtime_settings
validate_research_loop_runtime_settings
runtime_summary="$(build_research_loop_runtime_summary)"

universe_summary="$(
  build_research_loop_universe_summary \
    "$market_l1_bucket" \
    "$EXPECTED_MAJOR_UNIVERSE_SIZE"
)"

candidate_summary="$(
  build_research_loop_candidate_summary \
    "$candidate_bucket" \
    "$CANDIDATE_READ_LIMIT" \
    "$candidate_p0_json" \
    "$candidate_p1_json" \
    "$candidate_p2_json" \
    "$candidate_objects_json" \
    "$candidate_records_json"
)"

report_object="$(latest_object_json "$output_bucket" "research-run-report/")"
replay_object="$(latest_object_json "$output_bucket" "replay-run/")"
index_object="$(latest_object_json "$output_bucket" "replay-run-index/")"
shadow_object="$(latest_object_json "$output_bucket" "shadow-validation-run/")"
paper_object="$(latest_object_json "$output_bucket" "paper-trade-run/")"

report_summary="$(
  build_research_loop_latest_report_summary \
    "$output_bucket" \
    "$report_object"
)"

collect_research_loop_recent_report_records \
  "$output_bucket" \
  "$REPORT_READ_LIMIT" \
  "$report_objects_json" \
  "$report_records_json"

current_approved_shard_batch_summary="$(
  select_current_approved_shard_batch_summary "$report_records_json"
)"
recent_research_report_coverage_summary="$(
  summarize_recent_research_report_coverage "$report_records_json"
)"
research_evidence_summary="$(
  select_research_evidence_summary \
    "$report_summary" \
    "$current_approved_shard_batch_summary"
)"
prefix_summary="$(
  build_research_loop_prefix_summary \
    "$report_object" \
    "$replay_object" \
    "$index_object" \
    "$shadow_object" \
    "$paper_object"
)"

emit_research_loop_state_report | redact

echo "research loop state check completed"
