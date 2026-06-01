#!/usr/bin/env bash

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

registry_summary_json() {
  local registry_file="$1"
  if [[ -z "$registry_file" ]]; then
    jq -n '{
      present:false,
      aggregate_count:0,
      symbol_count:0,
      symbols:[],
      strongest_positive_retest:[]
    }'
    return
  fi

  require_absolute_file "RESEARCH_AGGREGATE_REGISTRY_FILE or second argument" "$registry_file"
  jq -s -f "$SCRIPT_DIR/jq/research-report-registry-summary.jq" "$registry_file"
}
