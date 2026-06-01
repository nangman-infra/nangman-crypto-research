#!/usr/bin/env bash

prepare_research_loop_state_tmp_files() {
  lambda_json="$(mktemp)"
  task_json="$(mktemp)"
  candidate_p0_json="$(mktemp)"
  candidate_p1_json="$(mktemp)"
  candidate_p2_json="$(mktemp)"
  candidate_objects_json="$(mktemp)"
  candidate_records_json="$(mktemp)"
  report_objects_json="$(mktemp)"
  report_records_json="$(mktemp)"
  trap cleanup_research_loop_state_tmp_files EXIT
}

cleanup_research_loop_state_tmp_files() {
  rm -f \
    "${lambda_json:-}" \
    "${task_json:-}" \
    "${candidate_p0_json:-}" \
    "${candidate_p1_json:-}" \
    "${candidate_p2_json:-}" \
    "${candidate_objects_json:-}" \
    "${candidate_records_json:-}" \
    "${report_objects_json:-}" \
    "${report_records_json:-}"
}

fetch_research_loop_runtime_documents() {
  aws_cmd lambda get-function-configuration \
    --function-name "$DISPATCHER_FUNCTION" \
    --output json > "$lambda_json"

  aws_cmd ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --output json > "$task_json"
}

resolve_research_loop_runtime_settings() {
  lambda_state="$(jq -r '.State' "$lambda_json")"
  lambda_update_status="$(jq -r '.LastUpdateStatus' "$lambda_json")"
  dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
  candidate_bucket="${RESEARCH_CANDIDATE_S3_BUCKET:-$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKET // ""' "$lambda_json")}"
  if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
    candidate_bucket="$(first_csv_value_containing "$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKETS // ""' "$lambda_json")" "intel-candidate")"
  fi

  task_revision="$(jq -r '.taskDefinition.revision' "$task_json")"
  task_status="$(jq -r '.taskDefinition.status' "$task_json")"
  cpu_arch="$(jq -r '.taskDefinition.runtimePlatform.cpuArchitecture' "$task_json")"
  os_family="$(jq -r '.taskDefinition.runtimePlatform.operatingSystemFamily' "$task_json")"
  readonly_root="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | .readonlyRootFilesystem' "$task_json")"
  output_bucket="$(task_env_value RESEARCH_OUTPUT_S3_BUCKET)"
  market_l1_bucket="$(task_env_value RESEARCH_MARKET_L1_S3_BUCKET)"
}

validate_research_loop_runtime_settings() {
  if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
    echo "candidate bucket is not discoverable; set RESEARCH_CANDIDATE_S3_BUCKET" >&2
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

build_research_loop_runtime_summary() {
  jq -n -c \
    --arg lambda_state "$lambda_state" \
    --arg lambda_update_status "$lambda_update_status" \
    --arg dispatch_mode "$dispatch_mode" \
    --arg task_definition "${TASK_DEFINITION}:${task_revision}" \
    --arg task_status "$task_status" \
    --arg cpu_arch "$cpu_arch" \
    --arg os_family "$os_family" \
    --arg readonly_root "$readonly_root" \
    '{
      dispatcher_lambda_state:$lambda_state,
      dispatcher_update_status:$lambda_update_status,
      dispatcher_mode:$dispatch_mode,
      task_definition:$task_definition,
      task_status:$task_status,
      cpu_architecture:$cpu_arch,
      operating_system_family:$os_family,
      readonly_root_filesystem:($readonly_root == "true"),
      runtime_alive:(
        $lambda_state == "Active"
        and $lambda_update_status == "Successful"
        and $task_status == "ACTIVE"
        and $cpu_arch == "ARM64"
        and $os_family == "LINUX"
        and $readonly_root == "true"
      )
    }'
}
