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

mkdir -p \
  "$(dirname "$MERGED_SHADOW_FILE")" \
  "$(dirname "$OBSERVATION_PLAN_FILE")" \
  "$(dirname "$GAP_MANIFEST_FILE")" \
  "$(dirname "$ACCUMULATION_MANIFEST_FILE")" \
  "$(dirname "$ACCUMULATION_SUMMARY_FILE")" \
  "$(dirname "$CYCLE_SUMMARY_FILE")" \
  "$(dirname "$DECISION_FILE")"

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

if [[ "$accumulation_created" == true ]]; then
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg run_dir "$RUN_DIR" \
    --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
    --arg horizon_status_file "$HORIZON_STATUS_FILE" \
    --arg accumulation_blocked_reason "$accumulation_blocked_reason" \
    --arg merged_shadow_file "$MERGED_SHADOW_FILE" \
    --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
    --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
    --arg accumulation_manifest_file "$ACCUMULATION_MANIFEST_FILE" \
    --arg accumulation_summary_file "$ACCUMULATION_SUMMARY_FILE" \
    --arg cycle_summary_file "$CYCLE_SUMMARY_FILE" \
    --arg decision_file "$DECISION_FILE" \
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
      retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
      shadow_input_files:$shadow_input_files,
      merged_shadow_file:$merged_shadow_file,
      observation_plan_file:$observation_plan_file,
      gap_manifest_file:$gap_manifest_file,
      accumulation_manifest_file:$accumulation_manifest_file,
      accumulation_summary_file:$accumulation_summary_file,
      accumulation_blocked_reason:(if $accumulation_blocked_reason == "" then null else $accumulation_blocked_reason end),
      cycle_summary_file:$cycle_summary_file,
      decision_file:$decision_file,
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
    --arg accumulation_blocked_reason "$accumulation_blocked_reason" \
    --arg merged_shadow_file "$MERGED_SHADOW_FILE" \
    --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
    --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
    --arg cycle_summary_file "$CYCLE_SUMMARY_FILE" \
    --arg decision_file "$DECISION_FILE" \
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
      retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
      shadow_input_files:$shadow_input_files,
      merged_shadow_file:$merged_shadow_file,
      observation_plan_file:$observation_plan_file,
      gap_manifest_file:$gap_manifest_file,
      accumulation_manifest_file:null,
      accumulation_summary_file:null,
      accumulation_blocked_reason:(if $accumulation_blocked_reason == "" then null else $accumulation_blocked_reason end),
      cycle_summary_file:$cycle_summary_file,
      decision_file:$decision_file,
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

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --slurpfile cycle "$CYCLE_SUMMARY_FILE" \
  '
    def scheduler_action($verdict; $focused_research_manifest_file; $accumulation_blocked_reason):
      if $verdict == "DISCOVER_LATEST_MARKET_L1_AS_OF" then "DISCOVER_MARKET_L1_WATERMARK"
      elif $verdict == "WAIT_FOR_TARGET_HOLDING_WINDOW" then "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES"
      elif $verdict == "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION" then "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES"
      elif $verdict == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" and $focused_research_manifest_file != null then "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH"
      elif $verdict == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" and $accumulation_blocked_reason == "missing_retest_horizon_status_file" then "HOLD_FOR_OPERATOR_REVIEW"
      elif $verdict == "REVIEW_SHADOW_COMPLETION_EVIDENCE" then "REVIEW_SHADOW_COMPLETION_EVIDENCE"
      elif $verdict == "NO_SHADOW_SAMPLE_GAP_DETECTED" then "NOOP"
      elif $verdict == "NO_SHADOW_CANDIDATES" then "NOOP"
      else "HOLD_FOR_OPERATOR_REVIEW" end;
    def wait_action($action):
      ($action == "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES"
       or $action == "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES");

    ($cycle[0] // {}) as $summary
    | ($summary.next_decision.verdict // "UNKNOWN") as $verdict
    | scheduler_action($verdict; ($summary.accumulation_manifest_file // null); ($summary.accumulation_blocked_reason // null)) as $action
    | ($summary.next_decision.next_observation_not_before_ms // null) as $not_before_ms
    | {
        schema_version:"research_shadow_cycle_decision_v1",
        generated_at:$generated_at,
        decision_id:(
          "shadow_cycle_decision:"
          + (($summary.run_dir // "unknown") | split("/") | last)
          + ":"
          + $verdict
          + ":"
          + (($not_before_ms // $summary.latest_l1_as_of_ms // $summary.generated_at // $generated_at) | tostring)
        ),
        source_cycle_summary_file:($summary.cycle_summary_file // null),
        run_dir:($summary.run_dir // null),
        scheduler_action:$action,
        source_verdict:$verdict,
        run_not_before_ms:(if wait_action($action) then $not_before_ms else null end),
        run_not_before_at:(if wait_action($action) then ($summary.next_decision.next_observation_not_before_at // null) else null end),
        run_not_before_source:(if wait_action($action) then ($summary.next_decision.next_observation_not_before_source // null) else null end),
        focused_research_manifest_file:(
          if $action == "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH" then $summary.accumulation_manifest_file
          else null
          end
        ),
        focused_research_summary_file:(
          if $action == "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH" then $summary.accumulation_summary_file
          else null
          end
        ),
        latest_l1_as_of_ms:($summary.latest_l1_as_of_ms // null),
        shadow_sample_state:{
          shadow_validation_count:($summary.observation_summary.shadow_validation_count // 0),
          target_window_materialized_count:($summary.observation_summary.target_window_materialized_count // 0),
          candidate_lifecycle_count:($summary.gap_summary.candidate_lifecycle_count // 0),
          partially_materialized_candidate_count:($summary.gap_summary.partially_materialized_candidate_count // 0),
          pending_target_window_candidate_count:($summary.gap_summary.pending_target_window_candidate_count // 0),
          total_sample_deficit:($summary.gap_summary.total_sample_deficit // 0),
          symbols:($summary.gap_summary.symbols // [])
        },
        safe_next_actions:($summary.next_decision.safe_next_actions // []),
        blocked_actions:($summary.next_decision.blocked_actions // []),
        safety:{
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          local_decision_only:true,
          shadow_status_mutated:false,
          paper_live_enabled:false,
          live_enabled:false,
          order_execution_enabled:false
        }
      }
  ' > "$DECISION_FILE"

jq -r '
  "cycle_summary=\(.cycle_summary_file)",
  "decision_file=\(.decision_file)",
  "shadow_input_file_count=\(.shadow_input_files | length)",
  "merged_record_count=\(.merge_summary.merged_record_count)",
  "target_window_materialized_count=\(.observation_summary.target_window_materialized_count)",
  "total_sample_deficit=\(.gap_summary.total_sample_deficit)",
  "next_verdict=\(.next_decision.verdict)",
  "next_observation_not_before_ms=\(.next_decision.next_observation_not_before_ms // "none")",
  "accumulation_manifest_file=\(.accumulation_manifest_file // "not_created")",
  "blocked_actions=\(.next_decision.blocked_actions | join(","))",
  "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),shadow_status_mutated:\(.safety.shadow_status_mutated),paper_live_enabled:\(.safety.paper_live_enabled)"
' "$CYCLE_SUMMARY_FILE"

jq -r '
  "scheduler_action=\(.scheduler_action)",
  "run_not_before_ms=\(.run_not_before_ms // "none")",
  "focused_research_manifest_file=\(.focused_research_manifest_file // "not_created")"
' "$DECISION_FILE"

echo "shadow sample accumulation cycle completed" >&2
