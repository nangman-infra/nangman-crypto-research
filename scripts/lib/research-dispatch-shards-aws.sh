# shellcheck shell=bash

dispatch_task_env_value() {
  local task_json="$1"
  local name="$2"
  jq -r \
    --arg container "$CONTAINER_NAME" \
    --arg name "$name" \
    '[.taskDefinition.containerDefinitions[]
        | select(.name == $container)
        | (.environment // [])[]?
        | select(.name == $name)
        | .value][0] // ""' "$task_json"
}

load_dispatch_runtime_config() {
  require_command aws
  lambda_json="${RUN_DIR}/lambda-config.json"
  task_json="${RUN_DIR}/task-definition.json"
  aws_cmd lambda get-function-configuration \
    --function-name "$DISPATCHER_FUNCTION" \
    --output json > "$lambda_json"
  aws_cmd ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --output json > "$task_json"

  dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
  if [[ "$dispatch_mode" != "run_task" ]]; then
    echo "dispatcher must be in run_task mode for dispatch shards; got $dispatch_mode" >&2
    exit 1
  fi
  ECS_CLUSTER="$(jq -r '.Environment.Variables.ECS_CLUSTER_ARN // empty' "$lambda_json")"
  if [[ -z "$ECS_CLUSTER" ]]; then
    echo "ECS_CLUSTER_ARN is missing from dispatcher environment" >&2
    exit 1
  fi
  RESEARCH_BUCKET="$(dispatch_task_env_value "$task_json" "RESEARCH_OUTPUT_S3_BUCKET")"
  if [[ -z "$RESEARCH_BUCKET" || "$RESEARCH_BUCKET" == "null" ]]; then
    echo "RESEARCH_OUTPUT_S3_BUCKET is missing from task definition" >&2
    exit 1
  fi
}

dispatcher_tasks() {
  local running stopped
  running="$(aws_cmd ecs list-tasks \
    --cluster "$ECS_CLUSTER" \
    --started-by research-s3-dispatcher \
    --desired-status RUNNING \
    --query 'taskArns' \
    --output json)"
  stopped="$(aws_cmd ecs list-tasks \
    --cluster "$ECS_CLUSTER" \
    --started-by research-s3-dispatcher \
    --desired-status STOPPED \
    --query 'taskArns' \
    --output json)"
  jq -n --argjson running "$running" --argjson stopped "$stopped" \
    '$running + $stopped | unique'
}

latest_new_dispatcher_task() {
  local before_file="$1"
  local current_file="$2"
  local new_tasks=()
  while IFS= read -r task_arn; do
    [[ -n "$task_arn" ]] && new_tasks+=("$task_arn")
  done < <(jq -n \
    --slurpfile before "$before_file" \
    --slurpfile current "$current_file" \
    '($current[0] - $before[0])[]?' \
    | jq -r .)
  if [[ "${#new_tasks[@]}" -eq 0 ]]; then
    return 1
  fi
  aws_cmd ecs describe-tasks \
    --cluster "$ECS_CLUSTER" \
    --tasks "${new_tasks[@]}" \
    --output json \
  | jq -r '.tasks | sort_by(.createdAt) | last | .taskArn'
}

wait_for_new_dispatcher_task() {
  local before_file="$1"
  local current_file="$2"
  local elapsed=0
  local task_arn
  while (( elapsed < TASK_POLL_SECONDS )); do
    dispatcher_tasks > "$current_file"
    if task_arn="$(latest_new_dispatcher_task "$before_file" "$current_file")"; then
      printf '%s\n' "$task_arn"
      return
    fi
    sleep 2
    elapsed=$((elapsed + 2))
  done
  echo "timed out waiting for research-s3-dispatcher ECS task" >&2
  exit 1
}
