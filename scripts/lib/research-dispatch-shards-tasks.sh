# shellcheck shell=bash

append_completed_task_summary() {
  local task_result="$1"
  jq -c \
    --arg shard_id "$shard_id" \
    --arg manifest_file "$shard_manifest" \
    --arg s3_key "$shard_key" \
    --argjson candidate_count "$shard_candidate_count" \
    '.tasks[0]
      | {
          shard_id:$shard_id,
          manifest_file:$manifest_file,
          manifest_s3_key:$s3_key,
          candidate_count:$candidate_count,
          dry_run:false,
          task_arn:.taskArn,
          task_definition:.taskDefinitionArn,
          started_by:.startedBy,
          last_status:.lastStatus,
          stop_code:.stopCode,
          stopped_reason:.stoppedReason,
          exit_code:.containers[0].exitCode,
          reason:.containers[0].reason,
          image:.containers[0].image,
          image_digest:.containers[0].imageDigest
        }' <<<"$task_result" >> "$task_summary_jsonl"
}

print_task_result_status() {
  local task_result="$1"
  jq -r \
    '.tasks[0]
      | "exit_code=\(.containers[0].exitCode) reason=\(.containers[0].reason // "none") image_digest=\(.containers[0].imageDigest)"' \
    <<<"$task_result" \
    | redact
}

run_dispatch_shards() {
  dispatch_started_at="$(date -u +%Y-%m-%dT%H:%M:%S+00:00)"
  task_summary_jsonl="${RUN_DIR}/dispatch-tasks.jsonl"
  : > "$task_summary_jsonl"

  for ((i = 0; i < shard_count; i++)); do
    shard_num=$((i + 1))
    start=$((i * SHARD_SIZE))
    shard_id="${RUN_ID}_shard$(printf '%02d' "$shard_num")of$(printf '%02d' "$shard_count")"
    shard_dir="${SHARD_ROOT}/${shard_id}"
    shard_manifest="${shard_dir}/manifest.json"
    mkdir -p "$shard_dir"
    write_shard_manifest "$shard_id" "$start" "$SHARD_SIZE" "$shard_manifest"
    shard_candidate_count="$(jq '.candidate_bundle_refs | length' "$shard_manifest")"
    shard_key="${MANIFEST_S3_PREFIX%/}/run_id=${shard_id}/manifest.json"

    echo
    echo "shard=${shard_num}/${shard_count} id=$shard_id candidate_refs=$shard_candidate_count"
    echo "manifest=$shard_manifest"
    echo "s3_key=$shard_key"

    if bool_is_true "$DRY_RUN"; then
      append_dry_run_task_summary
      continue
    fi

    before_tasks="${shard_dir}/dispatcher-tasks.before.json"
    after_tasks="${shard_dir}/dispatcher-tasks.after.json"
    dispatcher_tasks > "$before_tasks"
    aws_cmd s3 cp "$shard_manifest" "s3://${RESEARCH_BUCKET}/${shard_key}" --only-show-errors
    task_arn="$(wait_for_new_dispatcher_task "$before_tasks" "$after_tasks")"
    echo "task=$task_arn"
    aws_cmd ecs wait tasks-stopped --cluster "$ECS_CLUSTER" --tasks "$task_arn"
    task_result="$(aws_cmd ecs describe-tasks --cluster "$ECS_CLUSTER" --tasks "$task_arn" --output json)"
    append_completed_task_summary "$task_result"
    print_task_result_status "$task_result"
    exit_code="$(jq -r '.tasks[0].containers[0].exitCode' <<<"$task_result")"
    if [[ "$exit_code" != "0" ]]; then
      echo "shard $shard_id failed with exit_code=$exit_code" >&2
      exit 1
    fi
  done
}
