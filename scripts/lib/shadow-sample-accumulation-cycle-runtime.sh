#!/usr/bin/env bash

validate_shadow_sample_accumulation_cycle_inputs() {
  require_command date
  require_command find
  require_command jq
  require_command mkdir
  require_command mktemp
  require_command sort

  require_absolute_path "RESEARCH_SHADOW_CYCLE_RUN_DIR or first argument" "$RUN_DIR"
  if [[ ! -d "$RUN_DIR" ]]; then
    echo "RESEARCH_SHADOW_CYCLE_RUN_DIR does not exist: $RUN_DIR" >&2
    exit 1
  fi
  require_absolute_file "RESEARCH_SHADOW_CYCLE_SOURCE_MANIFEST_FILE" "$SOURCE_MANIFEST_FILE"
  if [[ -n "$HORIZON_STATUS_FILE" ]]; then
    require_absolute_file "RESEARCH_SHADOW_CYCLE_RETEST_HORIZON_STATUS_FILE" "$HORIZON_STATUS_FILE"
  fi
  require_absolute_path "RESEARCH_SHADOW_CYCLE_MERGED_SHADOW_FILE" "$MERGED_SHADOW_FILE"
  require_absolute_path "RESEARCH_SHADOW_CYCLE_OBSERVATION_PLAN_FILE" "$OBSERVATION_PLAN_FILE"
  require_absolute_path "RESEARCH_SHADOW_CYCLE_GAP_MANIFEST_FILE" "$GAP_MANIFEST_FILE"
  require_absolute_path "RESEARCH_SHADOW_CYCLE_ACCUMULATION_MANIFEST_FILE" "$ACCUMULATION_MANIFEST_FILE"
  require_absolute_path "RESEARCH_SHADOW_CYCLE_ACCUMULATION_SUMMARY_FILE" "$ACCUMULATION_SUMMARY_FILE"
  require_absolute_path "RESEARCH_SHADOW_CYCLE_SUMMARY_FILE" "$CYCLE_SUMMARY_FILE"
  require_absolute_path "RESEARCH_SHADOW_CYCLE_DECISION_FILE" "$DECISION_FILE"
  positive_or_empty_integer_arg "RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS" "$LATEST_L1_AS_OF_MS"
}

prepare_shadow_sample_accumulation_cycle_outputs() {
  mkdir -p \
    "$(dirname "$MERGED_SHADOW_FILE")" \
    "$(dirname "$OBSERVATION_PLAN_FILE")" \
    "$(dirname "$GAP_MANIFEST_FILE")" \
    "$(dirname "$ACCUMULATION_MANIFEST_FILE")" \
    "$(dirname "$ACCUMULATION_SUMMARY_FILE")" \
    "$(dirname "$CYCLE_SUMMARY_FILE")" \
    "$(dirname "$DECISION_FILE")"
}

prepare_shadow_sample_accumulation_cycle_tmp_files() {
  shadow_files_tmp="$(mktemp)"
  shadow_files_json_tmp="$(mktemp)"
  trap cleanup_shadow_sample_accumulation_cycle_tmp_files EXIT
}

cleanup_shadow_sample_accumulation_cycle_tmp_files() {
  rm -f "${shadow_files_tmp:-}" "${shadow_files_json_tmp:-}"
}

collect_shadow_sample_accumulation_inputs() {
  if [[ $# -gt 0 ]]; then
    for path in "$@"; do
      require_absolute_file "shadow validation input file" "$path"
      printf '%s\n' "$path" >> "$shadow_files_tmp"
    done
  else
    find "$RUN_DIR" \
      -path "*/shadow-validation-run/schema=shadow_validation_run_v1/*/part-000001.jsonl" \
      -type f \
      -print \
    | sort > "$shadow_files_tmp"
  fi

  if [[ ! -s "$shadow_files_tmp" ]]; then
    echo "no shadow validation files found under $RUN_DIR" >&2
    exit 1
  fi

  jq -R . "$shadow_files_tmp" | jq -s . > "$shadow_files_json_tmp"
}

load_shadow_sample_accumulation_input_array() {
  shadow_inputs=()
  while IFS= read -r path; do
    [[ -z "$path" ]] && continue
    shadow_inputs+=("$path")
  done < "$shadow_files_tmp"
}

merge_shadow_sample_accumulation_inputs() {
  "${script_dir}/merge-shadow-validation-runs.sh" \
    "$MERGED_SHADOW_FILE" \
    "${shadow_inputs[@]}" >&2
}

build_shadow_sample_accumulation_observation_plan() {
  if [[ -n "$LATEST_L1_AS_OF_MS" ]]; then
    "${script_dir}/build-shadow-observation-plan.sh" \
      "$MERGED_SHADOW_FILE" \
      "$HORIZON_STATUS_FILE" \
      "$LATEST_L1_AS_OF_MS" \
      > "$OBSERVATION_PLAN_FILE"
  else
    "${script_dir}/build-shadow-observation-plan.sh" \
      "$MERGED_SHADOW_FILE" \
      "$HORIZON_STATUS_FILE" \
      > "$OBSERVATION_PLAN_FILE"
  fi
}

build_shadow_sample_accumulation_gap_manifest() {
  "${script_dir}/build-shadow-sample-gap-manifest.sh" \
    "$OBSERVATION_PLAN_FILE" \
    > "$GAP_MANIFEST_FILE"
}

maybe_build_shadow_sample_accumulation_manifest() {
  gap_verdict="$(jq -r '.next_decision.verdict // "UNKNOWN"' "$GAP_MANIFEST_FILE")"
  accumulation_created=false
  accumulation_blocked_reason=""
  if [[ "$gap_verdict" == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" ]]; then
    if [[ -n "$HORIZON_STATUS_FILE" ]]; then
      RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT="$ACCUMULATION_MANIFEST_FILE" \
      RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT="$ACCUMULATION_SUMMARY_FILE" \
        "${script_dir}/build-shadow-sample-accumulation-manifest.sh" \
          "$GAP_MANIFEST_FILE" \
          "$HORIZON_STATUS_FILE" \
          "$SOURCE_MANIFEST_FILE" >&2
      accumulation_created=true
    else
      accumulation_blocked_reason="missing_retest_horizon_status_file"
    fi
  fi
}

write_shadow_sample_accumulation_cycle_outputs() {
  write_shadow_sample_accumulation_cycle_summary \
    "$CYCLE_SUMMARY_FILE" \
    "$accumulation_created" \
    "$shadow_files_json_tmp"
  write_shadow_cycle_decision "$DECISION_FILE"
}
