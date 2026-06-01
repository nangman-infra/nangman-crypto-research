#!/usr/bin/env bash

research_batch_positive_integer_arg() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

research_batch_absolute_output_path() {
  local name="$1"
  local value="$2"
  case "$value" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $value" >&2
      exit 1
      ;;
  esac
}

validate_research_batch_manifest_config() {
  require_command aws
  require_command jq
  require_command sed
  require_command mktemp
  require_command date

  research_batch_positive_integer_arg "RESEARCH_BATCH_CANDIDATE_READ_LIMIT" "$CANDIDATE_READ_LIMIT"
  research_batch_positive_integer_arg "RESEARCH_BATCH_MAX_CANDIDATE_BUNDLE_COUNT" "$MAX_CANDIDATE_BUNDLE_COUNT"
  research_batch_positive_integer_arg "RESEARCH_BATCH_HISTORICAL_INDEX_READ_LIMIT" "$HISTORICAL_INDEX_READ_LIMIT"
  research_batch_positive_integer_arg "RESEARCH_BATCH_MAX_HISTORICAL_REPLAY_RUN_REF_COUNT" "$MAX_HISTORICAL_REPLAY_RUN_REF_COUNT"
  research_batch_positive_integer_arg "RESEARCH_BATCH_MAX_REPLAY_RUN_COUNT" "$MAX_REPLAY_RUN_COUNT"

  case "$UNIVERSE_MODE" in
    current_approved | current_observed | legacy_retest) ;;
    *)
      echo "RESEARCH_BATCH_UNIVERSE_MODE must be current_approved, current_observed, or legacy_retest; got $UNIVERSE_MODE" >&2
      exit 1
      ;;
  esac
}

prepare_research_batch_manifest_outputs() {
  if [[ -n "${RESEARCH_BATCH_MANIFEST_OUTPUT:-}" ]]; then
    MANIFEST_OUTPUT="$RESEARCH_BATCH_MANIFEST_OUTPUT"
  else
    local tmp_root
    tmp_root="${TMPDIR:-/tmp}"
    tmp_root="${tmp_root%/}"
    MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-input-manifest.XXXXXX")"
  fi
  SUMMARY_OUTPUT="${RESEARCH_BATCH_SUMMARY_OUTPUT:-${MANIFEST_OUTPUT}.summary.json}"

  research_batch_absolute_output_path "RESEARCH_BATCH_MANIFEST_OUTPUT" "$MANIFEST_OUTPUT"
  research_batch_absolute_output_path "RESEARCH_BATCH_SUMMARY_OUTPUT" "$SUMMARY_OUTPUT"
}

print_research_batch_manifest_header() {
  echo "== ${APP_NAME} batch manifest builder =="
  echo "region=$REGION"
  echo "dispatcher=$DISPATCHER_FUNCTION"
  echo "task_definition=$TASK_DEFINITION"
  echo "candidate_read_limit=$CANDIDATE_READ_LIMIT"
  echo "max_candidate_bundle_count=$MAX_CANDIDATE_BUNDLE_COUNT"
  echo "historical_index_read_limit=$HISTORICAL_INDEX_READ_LIMIT"
  echo "universe_mode=$UNIVERSE_MODE"
  echo
}
