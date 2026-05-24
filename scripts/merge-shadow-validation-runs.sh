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

require_command date
require_command jq
require_command mkdir
require_command mktemp

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

mkdir -p "$(dirname "$OUTPUT_FILE")" "$(dirname "$SUMMARY_OUTPUT")"

merged_tmp="$(mktemp)"
summary_tmp="$(mktemp)"
input_files_tmp="$(mktemp)"
trap 'rm -f "$merged_tmp" "$summary_tmp" "$input_files_tmp"' EXIT

printf '%s\n' "${INPUT_FILES[@]}" | jq -R . | jq -s . > "$input_files_tmp"

jq -s \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson generated_at_ms "$(date -u +%s)000" \
  --arg output_file "$OUTPUT_FILE" \
  --arg summary_output "$SUMMARY_OUTPUT" \
  --slurpfile input_files "$input_files_tmp" \
  '
    def unique_sorted: unique | sort;
    def records:
      map(if type == "array" then .[] else . end)
      | map(select(type == "object"));
    def status_value: (.status // "pending");
    def counts_by(expr):
      map(expr)
      | sort
      | group_by(.)
      | map({value:.[0], count:length});

    records as $runs
    | (
        $runs
        | map(select((.shadow_validation_run_id // "") != ""))
        | group_by(.shadow_validation_run_id)
        | map(last)
        | sort_by(.candidate_lifecycle_key // "", .symbol_canonical // "", .shadow_validation_run_id // "")
      ) as $merged
    | (
        $runs
        | map(select((.shadow_validation_run_id // "") != ""))
        | group_by(.shadow_validation_run_id)
        | map(select(length > 1) | {
            shadow_validation_run_id:.[0].shadow_validation_run_id,
            duplicate_count:length,
            statuses:(map(status_value) | unique_sorted),
            passed_values:(map(.passed // false) | unique | sort)
          })
      ) as $duplicates
    | {
        summary:{
          schema_version:"research_shadow_validation_merge_summary_v1",
          generated_at:$generated_at,
          generated_at_ms:$generated_at_ms,
          output_file:$output_file,
          summary_output:$summary_output,
          input_files:($input_files[0] // []),
          safety:{
            s3_write:false,
            ecs_task_started:false,
            dispatcher_mode_changed:false,
            local_merge_only:true,
            shadow_status_mutated:false,
            paper_live_enabled:false
          },
          input_record_count:($runs | length),
          merged_record_count:($merged | length),
          duplicate_record_count:(($runs | length) - ($merged | length)),
          duplicate_shadow_validation_run_count:($duplicates | length),
          schema_versions:($merged | map(.schema_version // "unknown") | unique_sorted),
          status_counts:($merged | counts_by(status_value)),
          symbol_count:($merged | map(.symbol_canonical // empty) | unique | length),
          symbols:($merged | map(.symbol_canonical // empty) | unique_sorted),
          candidate_lifecycle_count:($merged | map(.candidate_lifecycle_key // empty) | unique | length),
          duplicate_shadow_validation_runs:$duplicates,
          blocked_actions:[
            "do_not_mark_pending_shadow_passed_from_merge",
            "do_not_create_paper_without_completed_passed_shadow",
            "do_not_enable_live_from_shadow_merge"
          ]
        },
        records:$merged
      }
  ' "${INPUT_FILES[@]}" > "$summary_tmp"

jq -c '.records[]' "$summary_tmp" > "$merged_tmp"
jq '.summary' "$summary_tmp" > "$SUMMARY_OUTPUT"
mv "$merged_tmp" "$OUTPUT_FILE"

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

echo "shadow validation merge completed" >&2
