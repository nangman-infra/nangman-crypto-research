#!/usr/bin/env bash
set -euo pipefail

RUN_DIR="${RESEARCH_SHADOW_CYCLE_RUN_DIR:-${1:-}}"
if [[ $# -gt 0 ]]; then
  shift
fi

SOURCE_MANIFEST_FILE="${RESEARCH_SHADOW_CYCLE_SOURCE_MANIFEST_FILE:-${RUN_DIR%/}/research-input-manifest.json}"
DEFAULT_HORIZON_STATUS_FILE="${RUN_DIR%/}/retest-horizon-status.json"
HORIZON_STATUS_FILE="${RESEARCH_SHADOW_CYCLE_RETEST_HORIZON_STATUS_FILE:-}"
if [[ -z "$HORIZON_STATUS_FILE" && -f "$DEFAULT_HORIZON_STATUS_FILE" ]]; then
  HORIZON_STATUS_FILE="$DEFAULT_HORIZON_STATUS_FILE"
fi
MERGED_SHADOW_FILE="${RESEARCH_SHADOW_CYCLE_MERGED_SHADOW_FILE:-${RUN_DIR%/}/shadow-validation-merged.jsonl}"
if [[ "$MERGED_SHADOW_FILE" == *.jsonl ]]; then
  MERGED_SHADOW_SUMMARY_FILE="${MERGED_SHADOW_FILE%.jsonl}.summary.json"
else
  MERGED_SHADOW_SUMMARY_FILE="${MERGED_SHADOW_FILE}.summary.json"
fi
OBSERVATION_PLAN_FILE="${RESEARCH_SHADOW_CYCLE_OBSERVATION_PLAN_FILE:-${RUN_DIR%/}/shadow-observation-plan.cycle.json}"
GAP_MANIFEST_FILE="${RESEARCH_SHADOW_CYCLE_GAP_MANIFEST_FILE:-${RUN_DIR%/}/shadow-sample-gap-manifest.cycle.json}"
ACCUMULATION_MANIFEST_FILE="${RESEARCH_SHADOW_CYCLE_ACCUMULATION_MANIFEST_FILE:-${RUN_DIR%/}/shadow-accumulation-input-manifest.next.json}"
ACCUMULATION_SUMMARY_FILE="${RESEARCH_SHADOW_CYCLE_ACCUMULATION_SUMMARY_FILE:-${RUN_DIR%/}/shadow-accumulation-input-manifest.next.summary.json}"
CYCLE_SUMMARY_FILE="${RESEARCH_SHADOW_CYCLE_SUMMARY_FILE:-${RUN_DIR%/}/shadow-sample-accumulation-cycle-summary.json}"
DECISION_FILE="${RESEARCH_SHADOW_CYCLE_DECISION_FILE:-${RUN_DIR%/}/shadow-cycle-decision.json}"
LATEST_L1_AS_OF_MS="${RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS:-}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/shadow-sample-accumulation-cycle-output.sh
source "$script_dir/lib/shadow-sample-accumulation-cycle-output.sh"
# shellcheck source=scripts/lib/shadow-sample-accumulation-cycle-runtime.sh
source "$script_dir/lib/shadow-sample-accumulation-cycle-runtime.sh"
# shellcheck source=scripts/lib/shadow-sample-accumulation-cycle-validation.sh
source "$script_dir/lib/shadow-sample-accumulation-cycle-validation.sh"

validate_shadow_sample_accumulation_cycle_inputs
prepare_shadow_sample_accumulation_cycle_outputs
prepare_shadow_sample_accumulation_cycle_tmp_files
collect_shadow_sample_accumulation_inputs "$@"
load_shadow_sample_accumulation_input_array
merge_shadow_sample_accumulation_inputs
build_shadow_sample_accumulation_observation_plan
build_shadow_sample_accumulation_gap_manifest
maybe_build_shadow_sample_accumulation_manifest
write_shadow_sample_accumulation_cycle_outputs
print_shadow_sample_accumulation_cycle_result

echo "shadow sample accumulation cycle completed" >&2
