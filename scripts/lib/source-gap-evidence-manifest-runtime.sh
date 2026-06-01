#!/usr/bin/env bash

source_gap_tmp_files=()

cleanup_source_gap_evidence_manifest_tempfiles() {
  if [[ "${#source_gap_tmp_files[@]}" -gt 0 ]]; then
    rm -f "${source_gap_tmp_files[@]}"
  fi
}

source_gap_infer_candidate_bucket() {
  jq -r '
    [
      (.candidate_bundle_refs // [])[]?.uri
      | (capture("^s3://(?<bucket>[^/]+)/")? // {})
      | .bucket // empty
    ][0] // ""
  ' "$SOURCE_MANIFEST_FILE"
}

prepare_source_gap_evidence_manifest_inputs() {
  source_manifest_input="$(mktemp)"
  summary_tmp="$(mktemp)"
  source_gap_tmp_files=("$source_manifest_input" "$summary_tmp")
  trap cleanup_source_gap_evidence_manifest_tempfiles EXIT

  if [[ -n "$SOURCE_MANIFEST_FILE" ]]; then
    cp "$SOURCE_MANIFEST_FILE" "$source_manifest_input"
    if [[ -z "$CANDIDATE_BUCKET" ]]; then
      CANDIDATE_BUCKET="$(source_gap_infer_candidate_bucket)"
    fi
  else
    printf '{}\n' > "$source_manifest_input"
  fi
}

write_source_gap_evidence_manifest_files() {
  write_source_gap_evidence_manifest_outputs "$summary_tmp" "$source_manifest_input"
  jq '.summary' "$summary_tmp" > "$SUMMARY_OUTPUT"
  jq '.manifest' "$summary_tmp" > "$MANIFEST_OUTPUT"
}
