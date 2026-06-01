#!/usr/bin/env bash

activation_readiness_jq() {
  local name="$1"
  local path="$JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing activation readiness jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

print_activation_readiness_header() {
  echo "== ${APP_NAME} activation readiness =="
  echo "region=$REGION"
  echo "dispatcher=$DISPATCHER_FUNCTION"
  echo "cluster=$CLUSTER_NAME"
  echo "task_definition=$TASK_DEFINITION"
  echo
}

load_dispatcher_configuration() {
  aws_cmd lambda get-function-configuration \
    --function-name "$DISPATCHER_FUNCTION" \
    --output json > "$lambda_json"

  lambda_state="$(jq -r '.State' "$lambda_json")"
  lambda_update_status="$(jq -r '.LastUpdateStatus' "$lambda_json")"
  dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
  lambda_task_definition="$(jq -r '.Environment.Variables.ECS_TASK_DEFINITION // ""' "$lambda_json")"
  lambda_container="$(jq -r '.Environment.Variables.ECS_CONTAINER_NAME // ""' "$lambda_json")"
}

validate_dispatcher_configuration() {
  if [[ "$lambda_state" != "Active" ]]; then
    echo "dispatcher Lambda is not Active: $lambda_state" >&2
    exit 1
  fi
  if [[ "$lambda_update_status" != "Successful" ]]; then
    echo "dispatcher Lambda update status is not Successful: $lambda_update_status" >&2
    exit 1
  fi
  if [[ "$dispatch_mode" != "$EXPECTED_DISPATCH_MODE" ]]; then
    echo "dispatcher mode mismatch: expected=$EXPECTED_DISPATCH_MODE actual=$dispatch_mode" >&2
    exit 1
  fi
  if [[ "$lambda_task_definition" != "$TASK_DEFINITION" ]]; then
    echo "dispatcher task definition mismatch: expected=$TASK_DEFINITION actual=$lambda_task_definition" >&2
    exit 1
  fi
  if [[ "$lambda_container" != "$CONTAINER_NAME" ]]; then
    echo "dispatcher container mismatch: expected=$CONTAINER_NAME actual=$lambda_container" >&2
    exit 1
  fi
}

load_task_definition() {
  aws_cmd ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --output json > "$task_json"

  task_revision="$(jq -r '.taskDefinition.revision' "$task_json")"
  task_status="$(jq -r '.taskDefinition.status' "$task_json")"
  cpu_arch="$(jq -r '.taskDefinition.runtimePlatform.cpuArchitecture' "$task_json")"
  os_family="$(jq -r '.taskDefinition.runtimePlatform.operatingSystemFamily' "$task_json")"
  readonly_root="$(task_container_value '.readonlyRootFilesystem')"
  image="$(task_container_value '.image')"
  output_bucket="$(task_env_value RESEARCH_OUTPUT_S3_BUCKET)"
  market_l1_bucket="$(task_env_value RESEARCH_MARKET_L1_S3_BUCKET)"
  history_index_bucket="$(task_env_value RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET)"
  history_index_prefix="$(task_env_value RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX)"
}

task_container_value() {
  local expression="$1"
  jq -r \
    --arg name "$CONTAINER_NAME" \
    ".taskDefinition.containerDefinitions[] | select(.name == \$name) | $expression" \
    "$task_json"
}

validate_task_definition() {
  if [[ "$task_status" != "ACTIVE" ]]; then
    echo "task definition is not ACTIVE: $task_status" >&2
    exit 1
  fi
  if [[ "$cpu_arch" != "ARM64" || "$os_family" != "LINUX" ]]; then
    echo "task runtime platform mismatch: ${cpu_arch}/${os_family}" >&2
    exit 1
  fi
  if [[ "$readonly_root" != "true" ]]; then
    echo "container readonlyRootFilesystem is not true" >&2
    exit 1
  fi
  if [[ -z "$output_bucket" || "$output_bucket" == "null" ]]; then
    echo "RESEARCH_OUTPUT_S3_BUCKET is missing from task definition" >&2
    exit 1
  fi
  if [[ -z "$market_l1_bucket" || "$market_l1_bucket" == "null" ]]; then
    echo "RESEARCH_MARKET_L1_S3_BUCKET is missing from task definition" >&2
    exit 1
  fi
}

print_task_definition_summary() {
  {
    echo "task ok: ${TASK_DEFINITION}:${task_revision} ${cpu_arch}/${os_family} readonly=${readonly_root}"
    echo "image=$image"
    echo "output_bucket=$output_bucket"
    echo "market_l1_bucket=$market_l1_bucket"
    echo "historical_replay_run_index_bucket=${history_index_bucket:-not-configured}"
    echo "historical_replay_run_index_prefix=${history_index_prefix:-not-configured}"
  } | redact
}
