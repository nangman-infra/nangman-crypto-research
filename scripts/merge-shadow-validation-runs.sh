#!/usr/bin/env bash
set -euo pipefail

OUTPUT_FILE="${RESEARCH_SHADOW_MERGE_OUTPUT:-${1:-}}"
if [[ -n "${RESEARCH_SHADOW_MERGE_SUMMARY_OUTPUT:-}" ]]; then
  SUMMARY_OUTPUT="$RESEARCH_SHADOW_MERGE_SUMMARY_OUTPUT"
elif [[ "$OUTPUT_FILE" == *.jsonl ]]; then
  SUMMARY_OUTPUT="${OUTPUT_FILE%.jsonl}.summary.json"
else
  SUMMARY_OUTPUT="${OUTPUT_FILE}.summary.json"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JQ_PROGRAM="$SCRIPT_DIR/jq/merge-shadow-validation-runs.jq"

# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/shadow-validation-merge-runtime.sh
source "$SCRIPT_DIR/lib/shadow-validation-merge-runtime.sh"

validate_shadow_validation_merge_inputs "$@"
prepare_shadow_validation_merge_outputs
prepare_shadow_validation_merge_tmp_files
write_shadow_validation_merge_input_file_list
run_shadow_validation_merge
write_shadow_validation_merge_outputs
print_shadow_validation_merge_result

echo "shadow validation merge completed" >&2
