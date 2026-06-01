#!/usr/bin/env bash

source_gap_require_absolute_file() {
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

source_gap_require_absolute_path() {
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

validate_source_gap_evidence_manifest_config() {
  require_command date
  require_command jq
  require_command mktemp
  source_gap_require_absolute_file "RESEARCH_SOURCE_GAP_DIAGNOSIS_FILE or first argument" "$DIAGNOSIS_FILE"

  case "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" in
    auto | true | false) ;;
    *)
      echo "RESEARCH_SOURCE_GAP_INCLUDE_HISTORICAL_INDEX_REFS must be auto, true, or false; got $INCLUDE_HISTORICAL_INDEX_REFS" >&2
      exit 1
      ;;
  esac

  if [[ -n "$SOURCE_MANIFEST_FILE" ]]; then
    source_gap_require_absolute_file "RESEARCH_SOURCE_MANIFEST_FILE or second argument" "$SOURCE_MANIFEST_FILE"
  fi
}

prepare_source_gap_evidence_manifest_outputs() {
  if [[ -n "${RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT:-}" ]]; then
    MANIFEST_OUTPUT="$RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT"
  else
    local tmp_root
    tmp_root="${TMPDIR:-/tmp}"
    tmp_root="${tmp_root%/}"
    MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-source-gap-input-manifest.XXXXXX")"
  fi
  SUMMARY_OUTPUT="${RESEARCH_SOURCE_GAP_SUMMARY_OUTPUT:-${MANIFEST_OUTPUT}.summary.json}"
  source_gap_require_absolute_path "RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT" "$MANIFEST_OUTPUT"
  source_gap_require_absolute_path "RESEARCH_SOURCE_GAP_SUMMARY_OUTPUT" "$SUMMARY_OUTPUT"
}
