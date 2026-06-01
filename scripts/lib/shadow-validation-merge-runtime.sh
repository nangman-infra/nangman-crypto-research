#!/usr/bin/env bash

require_absolute_path() {
  local name="$1"
  local path="$2"
  case "$path" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $path" >&2
      exit 1
      ;;
  esac
}

require_absolute_file() {
  local name="$1"
  local path="$2"
  require_absolute_path "$name" "$path"
  if [[ ! -f "$path" ]]; then
    echo "$name does not exist: $path" >&2
    exit 1
  fi
}

validate_shadow_validation_merge_inputs() {
  require_command date
  require_command dirname
  require_command jq
  require_command mkdir
  require_command mktemp
  require_command mv

  require_absolute_file "shadow validation merge jq program" "$JQ_PROGRAM"
  require_absolute_path "RESEARCH_SHADOW_MERGE_OUTPUT or first argument" "$OUTPUT_FILE"
  require_absolute_path "RESEARCH_SHADOW_MERGE_SUMMARY_OUTPUT" "$SUMMARY_OUTPUT"

  if [[ $# -lt 2 ]]; then
    echo "usage: $0 /absolute/output.jsonl /absolute/shadow-1.jsonl [/absolute/shadow-2.jsonl ...]" >&2
    exit 1
  fi

  shift
  INPUT_FILES=("$@")
  for path in "${INPUT_FILES[@]}"; do
    require_absolute_file "shadow validation input file" "$path"
  done
}

prepare_shadow_validation_merge_outputs() {
  mkdir -p "$(dirname "$OUTPUT_FILE")" "$(dirname "$SUMMARY_OUTPUT")"
}

prepare_shadow_validation_merge_tmp_files() {
  merged_tmp="$(mktemp)"
  summary_tmp="$(mktemp)"
  input_files_tmp="$(mktemp)"
  trap cleanup_shadow_validation_merge_tmp_files EXIT
}

cleanup_shadow_validation_merge_tmp_files() {
  rm -f "${merged_tmp:-}" "${summary_tmp:-}" "${input_files_tmp:-}"
}

write_shadow_validation_merge_input_file_list() {
  printf '%s\n' "${INPUT_FILES[@]}" | jq -R . | jq -s . > "$input_files_tmp"
}

run_shadow_validation_merge() {
  jq -s \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --argjson generated_at_ms "$(date -u +%s)000" \
    --arg output_file "$OUTPUT_FILE" \
    --arg summary_output "$SUMMARY_OUTPUT" \
    --slurpfile input_files "$input_files_tmp" \
    -f "$JQ_PROGRAM" \
    "${INPUT_FILES[@]}" > "$summary_tmp"
}

write_shadow_validation_merge_outputs() {
  jq -c '.records[]' "$summary_tmp" > "$merged_tmp"
  jq '.summary' "$summary_tmp" > "$SUMMARY_OUTPUT"
  mv "$merged_tmp" "$OUTPUT_FILE"
}

print_shadow_validation_merge_result() {
  jq -r '
    "shadow_merge_output=\(.output_file)",
    "shadow_merge_summary=\(.summary_output)",
    "input_record_count=\(.input_record_count)",
    "merged_record_count=\(.merged_record_count)",
    "duplicate_record_count=\(.duplicate_record_count)",
    "duplicate_shadow_validation_run_count=\(.duplicate_shadow_validation_run_count)",
    "symbols=\(.symbols | join(","))",
    "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),shadow_status_mutated:\(.safety.shadow_status_mutated),paper_live_enabled:\(.safety.paper_live_enabled)"
  ' "$SUMMARY_OUTPUT"
}
