#!/usr/bin/env bash

write_source_gap_evidence_manifest_outputs() {
  local summary_tmp="$1"
  local source_manifest_input="$2"
  local jq_program="${SOURCE_GAP_EVIDENCE_MANIFEST_OUTPUT_JQ:-$SCRIPT_DIR/jq/source-gap-evidence-manifest-output.jq}"
  local jq_dir

  if [[ ! -f "$jq_program" ]]; then
    echo "missing source-gap evidence manifest jq program: $jq_program" >&2
    exit 1
  fi
  jq_dir="$(cd -- "$(dirname -- "$jq_program")" && pwd -P)"

  jq -n \
    -L "$jq_dir" \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg diagnosis_file "$DIAGNOSIS_FILE" \
    --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
    --arg manifest_output "$MANIFEST_OUTPUT" \
    --arg summary_output "$SUMMARY_OUTPUT" \
    --arg focus_statuses "$FOCUS_STATUSES" \
    --arg candidate_bucket "$CANDIDATE_BUCKET" \
    --arg packet_id "$PACKET_ID" \
    --arg run_scope "$RUN_SCOPE" \
    --arg include_historical_index_refs "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" \
    --slurpfile diagnosis "$DIAGNOSIS_FILE" \
    --slurpfile source "$source_manifest_input" \
    -f "$jq_program" > "$summary_tmp"
}
