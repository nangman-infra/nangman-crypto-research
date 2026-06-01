#!/usr/bin/env bash
set -euo pipefail

GAP_MANIFEST_FILE="${RESEARCH_SHADOW_SAMPLE_GAP_MANIFEST_FILE:-${1:-}}"
HORIZON_STATUS_FILE="${RESEARCH_RETEST_HORIZON_STATUS_FILE:-${2:-}}"
SOURCE_MANIFEST_FILE="${RESEARCH_SOURCE_MANIFEST_FILE:-${3:-}}"
ACCUMULATION_PACKET_ID="${RESEARCH_SHADOW_ACCUMULATION_PACKET_ID:-research_shadow_accumulation_$(date -u +%Y%m%dT%H%M%SZ)}"
ACCUMULATION_RUN_SCOPE="${RESEARCH_SHADOW_ACCUMULATION_RUN_SCOPE:-shadow_sample_accumulation_local_validation}"
INCLUDE_HISTORICAL_INDEX_REFS="${RESEARCH_SHADOW_ACCUMULATION_INCLUDE_HISTORICAL_INDEX_REFS:-true}"
INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED="$(printf '%s' "$INCLUDE_HISTORICAL_INDEX_REFS" | tr '[:upper:]' '[:lower:]')"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JQ_PROGRAM="$SCRIPT_DIR/jq/build-shadow-sample-accumulation-manifest.jq"

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

require_absolute_file "shadow accumulation jq program" "$JQ_PROGRAM"
require_absolute_file "RESEARCH_SHADOW_SAMPLE_GAP_MANIFEST_FILE or first argument" "$GAP_MANIFEST_FILE"
require_absolute_file "RESEARCH_RETEST_HORIZON_STATUS_FILE or second argument" "$HORIZON_STATUS_FILE"
require_absolute_file "RESEARCH_SOURCE_MANIFEST_FILE or third argument" "$SOURCE_MANIFEST_FILE"
case "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" in
  true | false) ;;
  *)
    echo "RESEARCH_SHADOW_ACCUMULATION_INCLUDE_HISTORICAL_INDEX_REFS must be true or false; got $INCLUDE_HISTORICAL_INDEX_REFS" >&2
    exit 1
    ;;
esac

if [[ -n "${RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT:-}" ]]; then
  ACCUMULATION_MANIFEST_OUTPUT="$RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT"
else
  tmp_root="${TMPDIR:-/tmp}"
  tmp_root="${tmp_root%/}"
  ACCUMULATION_MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-shadow-accumulation-manifest.XXXXXX")"
fi
ACCUMULATION_SUMMARY_OUTPUT="${RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT:-${ACCUMULATION_MANIFEST_OUTPUT}.summary.json}"
require_absolute_path "RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT" "$ACCUMULATION_MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT" "$ACCUMULATION_SUMMARY_OUTPUT"

summary_tmp="$(mktemp)"
trap 'rm -f "$summary_tmp"' EXIT

jq -n \
  -L "$SCRIPT_DIR/jq" \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg generated_at_ms "$(date -u +%s)000" \
  --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
  --arg horizon_status_file "$HORIZON_STATUS_FILE" \
  --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
  --arg accumulation_manifest_output "$ACCUMULATION_MANIFEST_OUTPUT" \
  --arg accumulation_summary_output "$ACCUMULATION_SUMMARY_OUTPUT" \
  --arg accumulation_packet_id "$ACCUMULATION_PACKET_ID" \
  --arg accumulation_run_scope "$ACCUMULATION_RUN_SCOPE" \
  --arg include_historical_index_refs "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" \
  --slurpfile gap "$GAP_MANIFEST_FILE" \
  --slurpfile status "$HORIZON_STATUS_FILE" \
  --slurpfile source "$SOURCE_MANIFEST_FILE" \
  -f "$JQ_PROGRAM" > "$summary_tmp"

jq '.summary' "$summary_tmp" > "$ACCUMULATION_SUMMARY_OUTPUT"
jq '.manifest' "$summary_tmp" > "$ACCUMULATION_MANIFEST_OUTPUT"

selected_count="$(jq -r '.backlog_summary.selected_candidate_bundle_ref_count' "$ACCUMULATION_SUMMARY_OUTPUT")"
if [[ "$selected_count" == "0" ]]; then
  jq -r '
    "backlog_candidate_lifecycle_count=\(.backlog_summary.backlog_candidate_lifecycle_count)",
    "status_candidate_count=\(.backlog_summary.status_candidate_count)",
    "selected_candidate_bundle_ref_count=\(.backlog_summary.selected_candidate_bundle_ref_count)",
    "missing_candidate_ref_count=\(.backlog_summary.missing_candidate_ref_count)",
    "verdict=\(.next_decision.verdict)"
  ' "$ACCUMULATION_SUMMARY_OUTPUT"
  echo "no shadow accumulation candidate bundle refs were selected" >&2
  exit 1
fi

jq -r '
  "accumulation_manifest_output=\(.accumulation_manifest_output)",
  "accumulation_summary_output=\(.accumulation_summary_output)",
  "verdict=\(.next_decision.verdict)",
  "backlog_candidate_lifecycle_count=\(.backlog_summary.backlog_candidate_lifecycle_count)",
  "backlog_symbols=\(.backlog_summary.backlog_symbols | join(","))",
  "total_sample_deficit=\(.backlog_summary.total_sample_deficit)",
  "status_candidate_count=\(.backlog_summary.status_candidate_count)",
  "selected_candidate_bundle_ref_count=\(.backlog_summary.selected_candidate_bundle_ref_count)",
  "selected_historical_replay_run_index_ref_count=\(.backlog_summary.selected_historical_replay_run_index_ref_count)",
  "missing_candidate_ref_count=\(.backlog_summary.missing_candidate_ref_count)",
  "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),shadow_status_mutated:\(.safety.shadow_status_mutated),paper_live_enabled:\(.safety.paper_live_enabled)"
' "$ACCUMULATION_SUMMARY_OUTPUT"

echo "shadow sample accumulation manifest build completed"
