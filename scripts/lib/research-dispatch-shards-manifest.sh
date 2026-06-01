# shellcheck shell=bash

build_or_copy_base_manifest() {
  if [[ -n "$SOURCE_MANIFEST_FILE" ]]; then
    require_absolute_file "RESEARCH_DISPATCH_SOURCE_MANIFEST_FILE" "$SOURCE_MANIFEST_FILE"
    cp "$SOURCE_MANIFEST_FILE" "$BASE_MANIFEST_OUTPUT"
    if [[ -n "$SOURCE_MANIFEST_SUMMARY_FILE" ]]; then
      require_absolute_file "RESEARCH_DISPATCH_SOURCE_MANIFEST_SUMMARY_FILE" "$SOURCE_MANIFEST_SUMMARY_FILE"
      cp "$SOURCE_MANIFEST_SUMMARY_FILE" "$BASE_MANIFEST_SUMMARY_OUTPUT"
    else
      write_dispatch_source_manifest_summary
    fi
    return
  fi

  export RESEARCH_BATCH_UNIVERSE_MODE="$UNIVERSE_MODE"
  export RESEARCH_BATCH_MANIFEST_OUTPUT="$BASE_MANIFEST_OUTPUT"
  export RESEARCH_BATCH_SUMMARY_OUTPUT="$BASE_MANIFEST_SUMMARY_OUTPUT"
  "${SCRIPT_DIR}/build-research-batch-manifest.sh" 2>&1 \
    | redact \
    | tee "${RUN_DIR}/build-research-batch-manifest.log"
}

write_shard_manifest() {
  local shard_id="$1"
  local start="$2"
  local size="$3"
  local output_file="$4"

  jq \
    --arg id "$shard_id" \
    --arg scope "current_approved_auto_research_validation_shard" \
    --argjson start "$start" \
    --argjson size "$size" \
    '.research_packet_id = $id
      | .run_scope = $scope
      | .candidate_bundle_refs = (.candidate_bundle_refs[$start:($start + $size)])
      | .runtime_budget_policy.max_candidate_bundle_count = $size' \
    "$BASE_MANIFEST_OUTPUT" > "$output_file"
}
