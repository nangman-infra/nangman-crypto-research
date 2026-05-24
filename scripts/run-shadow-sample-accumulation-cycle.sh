#!/usr/bin/env bash
set -euo pipefail

RUN_DIR="${RESEARCH_SHADOW_CYCLE_RUN_DIR:-${1:-}}"
if [[ $# -gt 0 ]]; then
  shift
fi

SOURCE_MANIFEST_FILE="${RESEARCH_SHADOW_CYCLE_SOURCE_MANIFEST_FILE:-${RUN_DIR%/}/research-input-manifest.json}"
HORIZON_STATUS_FILE="${RESEARCH_SHADOW_CYCLE_RETEST_HORIZON_STATUS_FILE:-${RUN_DIR%/}/retest-horizon-status.json}"
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
LATEST_L1_AS_OF_MS="${RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS:-}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

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

positive_or_empty_integer_arg() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" ]]; then
    return
  fi
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

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
require_absolute_file "RESEARCH_SHADOW_CYCLE_RETEST_HORIZON_STATUS_FILE" "$HORIZON_STATUS_FILE"
require_absolute_path "RESEARCH_SHADOW_CYCLE_MERGED_SHADOW_FILE" "$MERGED_SHADOW_FILE"
require_absolute_path "RESEARCH_SHADOW_CYCLE_OBSERVATION_PLAN_FILE" "$OBSERVATION_PLAN_FILE"
require_absolute_path "RESEARCH_SHADOW_CYCLE_GAP_MANIFEST_FILE" "$GAP_MANIFEST_FILE"
require_absolute_path "RESEARCH_SHADOW_CYCLE_ACCUMULATION_MANIFEST_FILE" "$ACCUMULATION_MANIFEST_FILE"
require_absolute_path "RESEARCH_SHADOW_CYCLE_ACCUMULATION_SUMMARY_FILE" "$ACCUMULATION_SUMMARY_FILE"
require_absolute_path "RESEARCH_SHADOW_CYCLE_SUMMARY_FILE" "$CYCLE_SUMMARY_FILE"
positive_or_empty_integer_arg "RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS" "$LATEST_L1_AS_OF_MS"

mkdir -p \
  "$(dirname "$MERGED_SHADOW_FILE")" \
  "$(dirname "$OBSERVATION_PLAN_FILE")" \
  "$(dirname "$GAP_MANIFEST_FILE")" \
  "$(dirname "$ACCUMULATION_MANIFEST_FILE")" \
  "$(dirname "$ACCUMULATION_SUMMARY_FILE")" \
  "$(dirname "$CYCLE_SUMMARY_FILE")"

shadow_files_tmp="$(mktemp)"
shadow_files_json_tmp="$(mktemp)"
trap 'rm -f "$shadow_files_tmp" "$shadow_files_json_tmp"' EXIT

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

shadow_inputs=()
while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  shadow_inputs+=("$path")
done < "$shadow_files_tmp"

"${script_dir}/merge-shadow-validation-runs.sh" \
  "$MERGED_SHADOW_FILE" \
  "${shadow_inputs[@]}" >&2

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

"${script_dir}/build-shadow-sample-gap-manifest.sh" \
  "$OBSERVATION_PLAN_FILE" \
  > "$GAP_MANIFEST_FILE"

gap_verdict="$(jq -r '.next_decision.verdict // "UNKNOWN"' "$GAP_MANIFEST_FILE")"
accumulation_created=false
if [[ "$gap_verdict" == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" ]]; then
  RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT="$ACCUMULATION_MANIFEST_FILE" \
  RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT="$ACCUMULATION_SUMMARY_FILE" \
    "${script_dir}/build-shadow-sample-accumulation-manifest.sh" \
      "$GAP_MANIFEST_FILE" \
      "$HORIZON_STATUS_FILE" \
      "$SOURCE_MANIFEST_FILE" >&2
  accumulation_created=true
fi

if [[ "$accumulation_created" == true ]]; then
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg run_dir "$RUN_DIR" \
    --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
    --arg horizon_status_file "$HORIZON_STATUS_FILE" \
    --arg merged_shadow_file "$MERGED_SHADOW_FILE" \
    --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
    --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
    --arg accumulation_manifest_file "$ACCUMULATION_MANIFEST_FILE" \
    --arg accumulation_summary_file "$ACCUMULATION_SUMMARY_FILE" \
    --arg cycle_summary_file "$CYCLE_SUMMARY_FILE" \
    --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
    --argjson shadow_input_files "$(cat "$shadow_files_json_tmp")" \
    --slurpfile merge_summary "$MERGED_SHADOW_SUMMARY_FILE" \
    --slurpfile observation "$OBSERVATION_PLAN_FILE" \
    --slurpfile gap "$GAP_MANIFEST_FILE" \
    --slurpfile accumulation "$ACCUMULATION_SUMMARY_FILE" \
    '{
      schema_version:"research_shadow_sample_accumulation_cycle_summary_v1",
      generated_at:$generated_at,
      run_dir:$run_dir,
      source_manifest_file:$source_manifest_file,
      retest_horizon_status_file:$horizon_status_file,
      shadow_input_files:$shadow_input_files,
      merged_shadow_file:$merged_shadow_file,
      observation_plan_file:$observation_plan_file,
      gap_manifest_file:$gap_manifest_file,
      accumulation_manifest_file:$accumulation_manifest_file,
      accumulation_summary_file:$accumulation_summary_file,
      cycle_summary_file:$cycle_summary_file,
      latest_l1_as_of_ms:(if $latest_l1_as_of_ms == "" then null else ($latest_l1_as_of_ms | tonumber) end),
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_cycle_only:true,
        shadow_status_mutated:false,
        paper_live_enabled:false
      },
      merge_summary:($merge_summary[0] // null),
      observation_summary:($observation[0].observation_summary // null),
      gap_summary:($gap[0].shadow_sample_gap_summary // null),
      accumulation_summary:($accumulation[0].backlog_summary // null),
      next_decision:{
        verdict:($gap[0].next_decision.verdict // null),
        safe_next_actions:($gap[0].next_decision.safe_next_actions // []),
        next_observation_not_before_ms:($gap[0].next_decision.next_observation_not_before_ms // null),
        next_observation_not_before_at:($gap[0].next_decision.next_observation_not_before_at // null),
        next_observation_not_before_source:($gap[0].next_decision.next_observation_not_before_source // null),
        blocked_actions:(
          (($gap[0].next_decision.blocked_actions // [])
          + ($accumulation[0].next_decision.blocked_actions // []))
          | unique
          | sort
        )
      }
    }' > "$CYCLE_SUMMARY_FILE"
else
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg run_dir "$RUN_DIR" \
    --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
    --arg horizon_status_file "$HORIZON_STATUS_FILE" \
    --arg merged_shadow_file "$MERGED_SHADOW_FILE" \
    --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
    --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
    --arg cycle_summary_file "$CYCLE_SUMMARY_FILE" \
    --arg latest_l1_as_of_ms "$LATEST_L1_AS_OF_MS" \
    --argjson shadow_input_files "$(cat "$shadow_files_json_tmp")" \
    --slurpfile merge_summary "$MERGED_SHADOW_SUMMARY_FILE" \
    --slurpfile observation "$OBSERVATION_PLAN_FILE" \
    --slurpfile gap "$GAP_MANIFEST_FILE" \
    '{
      schema_version:"research_shadow_sample_accumulation_cycle_summary_v1",
      generated_at:$generated_at,
      run_dir:$run_dir,
      source_manifest_file:$source_manifest_file,
      retest_horizon_status_file:$horizon_status_file,
      shadow_input_files:$shadow_input_files,
      merged_shadow_file:$merged_shadow_file,
      observation_plan_file:$observation_plan_file,
      gap_manifest_file:$gap_manifest_file,
      accumulation_manifest_file:null,
      accumulation_summary_file:null,
      cycle_summary_file:$cycle_summary_file,
      latest_l1_as_of_ms:(if $latest_l1_as_of_ms == "" then null else ($latest_l1_as_of_ms | tonumber) end),
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_cycle_only:true,
        shadow_status_mutated:false,
        paper_live_enabled:false
      },
      merge_summary:($merge_summary[0] // null),
      observation_summary:($observation[0].observation_summary // null),
      gap_summary:($gap[0].shadow_sample_gap_summary // null),
      accumulation_summary:null,
      next_decision:{
        verdict:($gap[0].next_decision.verdict // null),
        safe_next_actions:($gap[0].next_decision.safe_next_actions // []),
        next_observation_not_before_ms:($gap[0].next_decision.next_observation_not_before_ms // null),
        next_observation_not_before_at:($gap[0].next_decision.next_observation_not_before_at // null),
        next_observation_not_before_source:($gap[0].next_decision.next_observation_not_before_source // null),
        blocked_actions:($gap[0].next_decision.blocked_actions // [])
      }
    }' > "$CYCLE_SUMMARY_FILE"
fi

jq -r '
  "cycle_summary=\(.cycle_summary_file)",
  "shadow_input_file_count=\(.shadow_input_files | length)",
  "merged_record_count=\(.merge_summary.merged_record_count)",
  "target_window_materialized_count=\(.observation_summary.target_window_materialized_count)",
  "total_sample_deficit=\(.gap_summary.total_sample_deficit)",
  "next_verdict=\(.next_decision.verdict)",
  "accumulation_manifest_file=\(.accumulation_manifest_file // "not_created")",
  "blocked_actions=\(.next_decision.blocked_actions | join(","))",
  "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),shadow_status_mutated:\(.safety.shadow_status_mutated),paper_live_enabled:\(.safety.paper_live_enabled)"
' "$CYCLE_SUMMARY_FILE"

echo "shadow sample accumulation cycle completed" >&2
