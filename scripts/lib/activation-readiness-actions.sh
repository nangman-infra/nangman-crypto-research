#!/usr/bin/env bash

run_activation_dry_run_if_requested() {
  if [[ -z "${RESEARCH_DRY_RUN_BUCKET:-}" && -z "${RESEARCH_DRY_RUN_KEY:-}" ]]; then
    return
  fi
  if [[ -z "${RESEARCH_DRY_RUN_BUCKET:-}" || -z "${RESEARCH_DRY_RUN_KEY:-}" ]]; then
    echo "RESEARCH_DRY_RUN_BUCKET and RESEARCH_DRY_RUN_KEY must be set together" >&2
    exit 1
  fi
  if [[ "$dispatch_mode" != "dry_run" ]]; then
    echo "refusing dry-run Lambda invocation while dispatch mode is $dispatch_mode" >&2
    exit 1
  fi

  invoke_payload="$(mktemp)"
  invoke_output="$(mktemp)"
  jq -n \
    --arg bucket "$RESEARCH_DRY_RUN_BUCKET" \
    --arg key "$RESEARCH_DRY_RUN_KEY" \
    -f "$(activation_readiness_jq activation-readiness-dry-run-payload.jq)" > "$invoke_payload"

  aws_cmd lambda invoke \
    --function-name "$DISPATCHER_FUNCTION" \
    --payload "fileb://$invoke_payload" \
    "$invoke_output" >/dev/null

  invoke_status="$(jq -r '.status' "$invoke_output")"
  dry_run_task_count="$(jq -r '.dryRunTaskCount // 0' "$invoke_output")"
  dispatched_task_count="$(jq -r '.dispatchedTaskCount // 0' "$invoke_output")"
  if [[ "$invoke_status" != "dry_run" || "$dry_run_task_count" -lt 1 || "$dispatched_task_count" -ne 0 ]]; then
    echo "unexpected dry-run invocation response:" >&2
    cat "$invoke_output" | redact >&2
    exit 1
  fi
  echo "dry-run invoke ok: dryRunTaskCount=$dry_run_task_count dispatchedTaskCount=$dispatched_task_count"
}

assert_no_dispatcher_tasks() {
  local desired_status
  local task_count
  for desired_status in RUNNING PENDING; do
    task_count="$(aws_cmd ecs list-tasks \
      --cluster "$CLUSTER_NAME" \
      --desired-status "$desired_status" \
      --started-by research-s3-dispatcher \
      --query 'length(taskArns)' \
      --output text)"
    if [[ "$task_count" != "0" ]]; then
      echo "unexpected research-s3-dispatcher task count for $desired_status: $task_count" >&2
      exit 1
    fi
  done
  echo "dispatcher side effect check ok: no RUNNING/PENDING started-by research-s3-dispatcher tasks"
}

print_latest_research_objects() {
  local prefix
  echo "latest research bucket objects:"
  for prefix in \
    research-run-report/ \
    replay-run/ \
    replay-run-index/ \
    shadow-validation-run/ \
    paper-trade-run/
  do
    aws_cmd s3api list-objects-v2 \
      --bucket "$output_bucket" \
      --prefix "$prefix" \
      --query 'sort_by(Contents || `[]`, &LastModified)[-1].{prefix:`'"$prefix"'`,lastModified:LastModified,size:Size,key:Key}' \
      --output json \
    | jq -c -f "$(activation_readiness_jq activation-readiness-latest-object-display.jq)" \
    | redact
  done
}
