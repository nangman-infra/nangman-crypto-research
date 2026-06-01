#!/usr/bin/env bash
set -euo pipefail

STATUS_FILE="${RESEARCH_HORIZON_STATUS_FILE:-${1:-}}"
SOURCE_MANIFEST_FILE="${RESEARCH_SOURCE_MANIFEST_FILE:-${2:-}}"
FOCUS_NEXT_ACTIONS="${RESEARCH_FOCUS_NEXT_ACTIONS:-run_research_replay_for_horizon,accumulate_completed_native_replay_samples,materialize_completed_native_replay_sample}"
FOCUS_PACKET_ID="${RESEARCH_FOCUS_PACKET_ID:-research_focus_$(date -u +%Y%m%dT%H%M%SZ)}"
FOCUS_RUN_SCOPE="${RESEARCH_FOCUS_RUN_SCOPE:-focused_retest_local_validation}"
INCLUDE_HISTORICAL_INDEX_REFS="${RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS:-auto}"
INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED="$(printf '%s' "$INCLUDE_HISTORICAL_INDEX_REFS" | tr '[:upper:]' '[:lower:]')"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
JQ_PROGRAM="$SCRIPT_DIR/jq/build-focused-retest-manifest.jq"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_absolute_file() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute file path" >&2
    exit 1
  fi
  if [[ ! -f "$path" ]]; then
    echo "$name does not exist: $path" >&2
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

require_command date
require_command jq
require_command mktemp
require_absolute_file "focused retest jq program" "$JQ_PROGRAM"
require_absolute_file "RESEARCH_HORIZON_STATUS_FILE or first argument" "$STATUS_FILE"
require_absolute_file "RESEARCH_SOURCE_MANIFEST_FILE or second argument" "$SOURCE_MANIFEST_FILE"
case "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" in
  auto | true | false) ;;
  *)
    echo "RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS must be auto, true, or false; got $INCLUDE_HISTORICAL_INDEX_REFS" >&2
    exit 1
    ;;
esac

if [[ -n "${RESEARCH_FOCUS_MANIFEST_OUTPUT:-}" ]]; then
  FOCUS_MANIFEST_OUTPUT="$RESEARCH_FOCUS_MANIFEST_OUTPUT"
else
  tmp_root="${TMPDIR:-/tmp}"
  tmp_root="${tmp_root%/}"
  FOCUS_MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-focused-input-manifest.XXXXXX")"
fi
FOCUS_SUMMARY_OUTPUT="${RESEARCH_FOCUS_SUMMARY_OUTPUT:-${FOCUS_MANIFEST_OUTPUT}.summary.json}"
require_absolute_path "RESEARCH_FOCUS_MANIFEST_OUTPUT" "$FOCUS_MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_FOCUS_SUMMARY_OUTPUT" "$FOCUS_SUMMARY_OUTPUT"

summary_tmp="$(mktemp)"
trap 'rm -f "$summary_tmp"' EXIT

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg status_file "$STATUS_FILE" \
  --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
  --arg focus_manifest_output "$FOCUS_MANIFEST_OUTPUT" \
  --arg focus_summary_output "$FOCUS_SUMMARY_OUTPUT" \
  --arg focus_next_actions "$FOCUS_NEXT_ACTIONS" \
  --arg focus_packet_id "$FOCUS_PACKET_ID" \
  --arg focus_run_scope "$FOCUS_RUN_SCOPE" \
  --arg include_historical_index_refs "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" \
  --slurpfile status "$STATUS_FILE" \
  --slurpfile source "$SOURCE_MANIFEST_FILE" \
  -L "$SCRIPT_DIR/jq" \
  -f "$JQ_PROGRAM" > "$summary_tmp"

jq '.summary' "$summary_tmp" > "$FOCUS_SUMMARY_OUTPUT"
jq '.manifest' "$summary_tmp" > "$FOCUS_MANIFEST_OUTPUT"

selected_count="$(jq -r '.focused.selected_candidate_bundle_ref_count' "$FOCUS_SUMMARY_OUTPUT")"
if [[ "$selected_count" == "0" ]]; then
  jq -r '
    "focus_horizon_count=\(.focused.focus_horizon_count)",
    "focus_candidate_count=\(.focused.focus_candidate_count)",
    "selected_candidate_bundle_ref_count=\(.focused.selected_candidate_bundle_ref_count)",
    "missing_candidate_ref_ids=\(.focused.missing_candidate_ref_ids | join(","))"
  ' "$FOCUS_SUMMARY_OUTPUT"
  echo "no focused candidate bundle refs were selected" >&2
  exit 1
fi

jq -r '
  "focus_manifest_output=\(.focus_manifest_output)",
  "focus_summary_output=\(.focus_summary_output)",
  "focus_next_actions=\(.focus_next_actions | join(","))",
  "focus_horizon_count=\(.focused.focus_horizon_count)",
  "focus_candidate_count=\(.focused.focus_candidate_count)",
  "selected_candidate_bundle_ref_count=\(.focused.selected_candidate_bundle_ref_count)",
  "selected_historical_replay_run_index_ref_count=\(.focused.selected_historical_replay_run_index_ref_count)",
  "symbols=\(.focused.symbols | join(","))",
  "horizons=\(.focused.horizons | map(.horizon + ":" + (.count|tostring)) | join(","))",
  "next_action_counts=\(.focused.next_action_counts | map(.next_action + ":" + (.count|tostring)) | join(","))",
  "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),historical_replay_run_index_refs_carried:\(.safety.historical_replay_run_index_refs_carried)"
' "$FOCUS_SUMMARY_OUTPUT"

echo "focused retest manifest build completed"
