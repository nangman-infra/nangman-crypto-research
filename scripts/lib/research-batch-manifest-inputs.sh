#!/usr/bin/env bash

RESEARCH_BATCH_INPUTS_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RESEARCH_BATCH_INPUTS_JQ_DIR="$(cd -- "$RESEARCH_BATCH_INPUTS_LIB_DIR/../jq" && pwd -P)"

research_batch_inputs_jq() {
  local name="$1"
  local path="$RESEARCH_BATCH_INPUTS_JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing research batch input jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

write_latest_research_batch_universe_summary() {
  local universe_key

  latest_universe_snapshot_object_json \
    "$market_l1_bucket" \
    "symbol_universe_snapshot/run_id=" > "$universe_object_json"

  universe_key="$(jq -r '.key // empty' "$universe_object_json")"
  if [[ -n "$universe_key" ]]; then
    aws_cmd s3 cp "s3://${market_l1_bucket}/${universe_key}" - \
    | jq -c \
        --argjson object "$(cat "$universe_object_json")" \
        -f "$(research_batch_inputs_jq research-batch-universe-summary.jq)" > "$universe_summary_json"
  else
    jq -n -c -f "$(research_batch_inputs_jq research-batch-empty-universe-summary.jq)" > "$universe_summary_json"
  fi
}

print_latest_research_batch_universe_summary() {
  {
    jq -r '
      "latest_universe_selection=\(.selection // "absent")",
      "latest_universe_key=\(.key // "absent")",
      "latest_universe_observed_count=\(.observed_symbol_count)",
      "latest_universe_approved_count=\(.approved_symbol_count)"
    ' "$universe_summary_json"
  } | redact
}

collect_research_batch_candidate_objects() {
  list_latest_objects "$candidate_bucket" "candidate-evidence-bundle/priority=p0/" "$CANDIDATE_READ_LIMIT" > "$candidate_p0_json"
  list_latest_objects "$candidate_bucket" "candidate-evidence-bundle/priority=p1/" "$CANDIDATE_READ_LIMIT" > "$candidate_p1_json"
  list_latest_objects "$candidate_bucket" "candidate-evidence-bundle/priority=p2/" "$CANDIDATE_READ_LIMIT" > "$candidate_p2_json"

  jq -s -c \
    --argjson limit "$CANDIDATE_READ_LIMIT" \
    -f "$(research_batch_inputs_jq research-batch-candidate-objects.jq)" \
    "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" > "$candidate_objects_json"
}

append_research_batch_candidate_record() {
  local object="$1"
  local key
  local last_modified
  local size

  key="$(jq -r '.Key' <<<"$object")"
  last_modified="$(jq -r '.LastModified' <<<"$object")"
  size="$(jq -r '.Size' <<<"$object")"
  [[ -z "$key" || "$key" == "null" ]] && return 0

  aws_cmd s3 cp "s3://${candidate_bucket}/${key}" - \
  | jq -c \
      --arg bucket "$candidate_bucket" \
      --arg key "$key" \
      --arg uri "s3://${candidate_bucket}/${key}" \
      --arg last_modified "$last_modified" \
      --argjson size "$size" \
      -f "$(research_batch_inputs_jq research-batch-candidate-record.jq)" >> "$candidate_records_json"
}

collect_research_batch_candidate_records() {
  : > "$candidate_records_json"
  while IFS= read -r object; do
    append_research_batch_candidate_record "$object"
  done < <(jq -c '.[]' "$candidate_objects_json")
}

write_research_batch_candidate_pool() {
  jq -s -c \
    --arg mode "$UNIVERSE_MODE" \
    --argjson universe "$(cat "$universe_summary_json")" \
    -f "$(research_batch_inputs_jq research-batch-candidate-pool.jq)" "$candidate_records_json" > "$all_candidates_json"
}

select_research_batch_candidates() {
  jq -c \
    --arg mode "$UNIVERSE_MODE" \
    --argjson max "$MAX_CANDIDATE_BUNDLE_COUNT" \
    -f "$(research_batch_inputs_jq research-batch-selected-candidates.jq)" "$all_candidates_json" > "$selected_candidates_json"
}

write_historical_replay_run_index_refs() {
  list_latest_objects "$output_bucket" "replay-run-index/" "$HISTORICAL_INDEX_READ_LIMIT" \
  | jq -c --arg bucket "$output_bucket" \
      -f "$(research_batch_inputs_jq research-batch-historical-index-objects.jq)" > "$historical_index_objects_json"
}
