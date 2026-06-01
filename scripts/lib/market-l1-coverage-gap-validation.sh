#!/usr/bin/env bash

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

positive_integer() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

normalize_bool() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

require_jq_programs() {
  require_absolute_file "plan gap jq program" "$PLAN_GAPS_JQ"
  require_absolute_file "aggregate gap jq program" "$AGGREGATE_GAPS_JQ"
  require_absolute_file "current missing replay jq program" "$CURRENT_MISSING_JQ"
  require_absolute_file "S3 window plan jq program" "$S3_WINDOW_PLAN_JQ"
}
