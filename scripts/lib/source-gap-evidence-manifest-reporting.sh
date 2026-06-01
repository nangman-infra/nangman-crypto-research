#!/usr/bin/env bash

assert_source_gap_evidence_manifest_selected() {
  local missing_bucket_ref_count
  local selected_count
  missing_bucket_ref_count="$(jq -r '.blocked.missing_candidate_bucket_ref_count' "$SUMMARY_OUTPUT")"
  selected_count="$(jq -r '.selected.selected_candidate_bundle_ref_count' "$SUMMARY_OUTPUT")"

  if [[ "$missing_bucket_ref_count" != "0" ]]; then
    jq -r '
      "selected_candidate_bundle_ref_count=\(.selected.selected_candidate_bundle_ref_count)",
      "missing_candidate_bucket_ref_count=\(.blocked.missing_candidate_bucket_ref_count)",
      "missing_candidate_bucket_refs=\(.blocked.missing_candidate_bucket_refs | map(.raw_ref) | join(","))"
    ' "$SUMMARY_OUTPUT"
    echo "candidate bucket is required for key-only evidence refs; set RESEARCH_SOURCE_GAP_CANDIDATE_S3_BUCKET or pass a source manifest" >&2
    exit 1
  fi

  if [[ "$selected_count" == "0" ]]; then
    jq -r '
      "focus_statuses=\(.focus_statuses | join(","))",
      "selected_symbol_count=\(.selected.selected_symbol_count)",
      "selected_candidate_bundle_ref_count=\(.selected.selected_candidate_bundle_ref_count)"
    ' "$SUMMARY_OUTPUT"
    echo "no source-gap candidate evidence refs were selected" >&2
    exit 1
  fi
}

print_source_gap_evidence_manifest_summary() {
  jq -r '
    "source_gap_manifest_output=\(.manifest_output)",
    "source_gap_summary_output=\(.summary_output)",
    "focus_statuses=\(.focus_statuses | join(","))",
    "selected_symbol_count=\(.selected.selected_symbol_count)",
    "selected_symbols=\(.selected.selected_symbols | join(","))",
    "selected_candidate_count=\(.selected.selected_candidate_count)",
    "selected_candidate_bundle_ref_count=\(.selected.selected_candidate_bundle_ref_count)",
    "selected_historical_replay_run_index_ref_count=\(.selected.selected_historical_replay_run_index_ref_count)",
    "ref_source_fields=\(.selected.ref_source_fields | join(","))",
    "safety=s3_read:\(.safety.s3_read),s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),historical_replay_run_index_refs_carried:\(.safety.historical_replay_run_index_refs_carried)"
  ' "$SUMMARY_OUTPUT"
}
