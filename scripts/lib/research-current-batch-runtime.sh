#!/usr/bin/env bash

fail() {
  echo "$*" >&2
  exit 1
}

require_absolute_path() {
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

require_absolute_file() {
  local name="$1"
  local value="$2"
  require_absolute_path "$name" "$value"
  if [[ ! -f "$value" ]]; then
    echo "$name does not exist: $value" >&2
    exit 1
  fi
}

task_env_value() {
  local name="$1"
  aws_cmd ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --output json \
  | jq -r \
      --arg container "$CONTAINER_NAME" \
      --arg name "$name" \
      '[.taskDefinition.containerDefinitions[]
        | select(.name == $container)
        | (.environment // [])[]?
        | select(.name == $name)
        | .value][0] // ""'
}

discover_market_l1_bucket() {
  if [[ -n "${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}" ]]; then
    printf '%s\n' "${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
    return
  fi
  task_env_value "RESEARCH_MARKET_L1_S3_BUCKET"
}
