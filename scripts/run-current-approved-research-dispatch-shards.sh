#!/usr/bin/env bash
set -euo pipefail

APP_NAME="research-app"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
UNIVERSE_MODE="${RESEARCH_BATCH_UNIVERSE_MODE:-current_approved}"
RUN_ID="${RESEARCH_DISPATCH_DRIVER_RUN_ID:-research_dispatch_$(date -u +%Y%m%dT%H%M%SZ)}"
DRIVER_ROOT="${RESEARCH_DISPATCH_DRIVER_ROOT:-/tmp/nangman-crypto/research-current-approved-dispatch}"
RUN_DIR="${RESEARCH_DISPATCH_DRIVER_RUN_DIR:-${DRIVER_ROOT%/}/${RUN_ID}}"
BASE_MANIFEST_OUTPUT="${RESEARCH_DISPATCH_BASE_MANIFEST_OUTPUT:-${RUN_DIR}/research-input-manifest.json}"
BASE_MANIFEST_SUMMARY_OUTPUT="${RESEARCH_DISPATCH_BASE_MANIFEST_SUMMARY_OUTPUT:-${RUN_DIR}/research-input-manifest.summary.json}"
SHARD_ROOT="${RESEARCH_DISPATCH_SHARD_ROOT:-${RUN_DIR}/shards}"
SUMMARY_OUTPUT="${RESEARCH_DISPATCH_SUMMARY_OUTPUT:-${RUN_DIR}/dispatch-shard-summary.json}"
SHARD_SIZE="${RESEARCH_DISPATCH_SHARD_SIZE:-40}"
DRY_RUN="${RESEARCH_DISPATCH_DRY_RUN:-false}"
SOURCE_MANIFEST_FILE="${RESEARCH_DISPATCH_SOURCE_MANIFEST_FILE:-}"
SOURCE_MANIFEST_SUMMARY_FILE="${RESEARCH_DISPATCH_SOURCE_MANIFEST_SUMMARY_FILE:-}"
MANIFEST_S3_PREFIX="${RESEARCH_DISPATCH_MANIFEST_S3_PREFIX:-research-input-manifest/schema=research_input_manifest_v1}"
TASK_POLL_SECONDS="${RESEARCH_DISPATCH_TASK_POLL_SECONDS:-120}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

positive_integer_arg() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
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

redact() {
  sed -E \
    -e 's/nangman-crypto-dev-[A-Za-z0-9-]+-[0-9]{6}/nangman-crypto-dev-<bucket-family>-<account-suffix>/g' \
    -e 's/[0-9]{12}\.dkr\.ecr/<aws-account-id>.dkr.ecr/g' \
    -e 's/account=[0-9]{12}/account=<aws-account-id>/g' \
    -e 's/"Account"[[:space:]]*:[[:space:]]*"[0-9]{12}"/"Account":"<aws-account-id>"/g' \
    -e 's#arn:aws:iam::[^[:space:]"]+#arn:aws:iam::<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:ecs:[^[:space:]"]+#arn:aws:ecs:<region>:<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:lambda:[^[:space:]"]+#arn:aws:lambda:<region>:<aws-account-id>:<resource>#g' \
    -e 's/subnet-[A-Za-z0-9]+/<subnet-id>/g' \
    -e 's/sg-[A-Za-z0-9]+/<security-group-id>/g'
}

aws_cmd() {
  aws --region "$REGION" "$@"
}

bool_is_true() {
  local lowered
  lowered="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
  case "$lowered" in
    1 | true | yes) return 0 ;;
    *) return 1 ;;
  esac
}

build_or_copy_base_manifest() {
  if [[ -n "$SOURCE_MANIFEST_FILE" ]]; then
    require_absolute_file "RESEARCH_DISPATCH_SOURCE_MANIFEST_FILE" "$SOURCE_MANIFEST_FILE"
    cp "$SOURCE_MANIFEST_FILE" "$BASE_MANIFEST_OUTPUT"
    if [[ -n "$SOURCE_MANIFEST_SUMMARY_FILE" ]]; then
      require_absolute_file "RESEARCH_DISPATCH_SOURCE_MANIFEST_SUMMARY_FILE" "$SOURCE_MANIFEST_SUMMARY_FILE"
      cp "$SOURCE_MANIFEST_SUMMARY_FILE" "$BASE_MANIFEST_SUMMARY_OUTPUT"
    else
      jq -n \
        --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
        --arg copied_manifest_file "$BASE_MANIFEST_OUTPUT" \
        --argjson selected_candidate_count "$(jq '.candidate_bundle_refs | length' "$BASE_MANIFEST_OUTPUT")" \
        '{
          schema_version:"research_dispatch_source_manifest_summary_v1",
          generated_at:$generated_at,
          source_manifest_file:$source_manifest_file,
          copied_manifest_file:$copied_manifest_file,
          selected_candidate_count:$selected_candidate_count
        }' > "$BASE_MANIFEST_SUMMARY_OUTPUT"
    fi
    return
  fi

  export RESEARCH_BATCH_UNIVERSE_MODE="$UNIVERSE_MODE"
  export RESEARCH_BATCH_MANIFEST_OUTPUT="$BASE_MANIFEST_OUTPUT"
  export RESEARCH_BATCH_SUMMARY_OUTPUT="$BASE_MANIFEST_SUMMARY_OUTPUT"
  "${script_dir}/build-research-batch-manifest.sh" 2>&1 \
    | redact \
    | tee "${RUN_DIR}/build-research-batch-manifest.log"
}

write_shard_manifest() {
  local shard_id="$1"
  local start="$2"
  local size="$3"
  local output_file="$4"

  jq \
    --arg id "$shard_id" \
    --arg scope "current_approved_auto_research_validation_shard" \
    --argjson start "$start" \
    --argjson size "$size" \
    '.research_packet_id = $id
      | .run_scope = $scope
      | .candidate_bundle_refs = (.candidate_bundle_refs[$start:($start + $size)])
      | .runtime_budget_policy.max_candidate_bundle_count = $size' \
    "$BASE_MANIFEST_OUTPUT" > "$output_file"
}

task_env_value() {
  local task_json="$1"
  local name="$2"
  jq -r \
    --arg container "$CONTAINER_NAME" \
    --arg name "$name" \
    '.taskDefinition.containerDefinitions[]
      | select(.name == $container)
      | (.environment // [])[]?
      | select(.name == $name)
      | .value' "$task_json" \
    | head -n 1
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

collect_report_summary() {
  local started_at="$1"
  local summary_jsonl="$2"
  : > "$summary_jsonl"
  aws_cmd s3api list-objects-v2 \
    --bucket "$RESEARCH_BUCKET" \
    --prefix "research-run-report/" \
    --output json \
  | jq -r --arg started_at "$started_at" '
      (.Contents // [])
      | map(select(.LastModified >= $started_at))
      | sort_by(.LastModified, .Key)
      | .[].Key
    ' \
  | while IFS= read -r key; do
      aws_cmd s3 cp "s3://${RESEARCH_BUCKET}/${key}" - --only-show-errors \
      | jq -c --arg key "$key" '
          {
            key:$key,
            research_packet_id,
            run_scope,
            research_run_status,
            source_candidate_count:(.source_candidate_ids | length),
            replay_run_count:(.replay_run_ids | length),
            partition_count,
            top_symbols,
            partition_symbols:([.partition_aggregates[].symbol_canonical] | unique),
            gate_biases:([.partition_aggregates[].gate_bias] | unique),
            retest_candidate_count:(.retest_candidate_keys | length),
            surviving_candidate_count:(.surviving_candidate_keys | length),
            shadow_validation_count:(.shadow_validation_runs | length),
            paper_trade_candidate_count:(.paper_trade_candidates | length)
          }
        ' >> "$summary_jsonl"
    done
}

require_command date
require_command jq
require_command mkdir
require_command sed
require_command tee
require_command cp
positive_integer_arg "RESEARCH_DISPATCH_SHARD_SIZE" "$SHARD_SIZE"
positive_integer_arg "RESEARCH_DISPATCH_TASK_POLL_SECONDS" "$TASK_POLL_SECONDS"
require_absolute_path "RESEARCH_DISPATCH_DRIVER_ROOT" "$DRIVER_ROOT"
require_absolute_path "RESEARCH_DISPATCH_DRIVER_RUN_DIR" "$RUN_DIR"
require_absolute_path "RESEARCH_DISPATCH_BASE_MANIFEST_OUTPUT" "$BASE_MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_DISPATCH_BASE_MANIFEST_SUMMARY_OUTPUT" "$BASE_MANIFEST_SUMMARY_OUTPUT"
require_absolute_path "RESEARCH_DISPATCH_SHARD_ROOT" "$SHARD_ROOT"
require_absolute_path "RESEARCH_DISPATCH_SUMMARY_OUTPUT" "$SUMMARY_OUTPUT"

if [[ "$UNIVERSE_MODE" != "current_approved" && "${RESEARCH_DISPATCH_ALLOW_NON_APPROVED_UNIVERSE:-false}" != "true" ]]; then
  echo "RESEARCH_BATCH_UNIVERSE_MODE must be current_approved for promotion-safe dispatch evidence; got $UNIVERSE_MODE" >&2
  echo "Set RESEARCH_DISPATCH_ALLOW_NON_APPROVED_UNIVERSE=true only for diagnostics." >&2
  exit 1
fi

mkdir -p "$RUN_DIR" "$SHARD_ROOT"

echo "== ${APP_NAME} current-approved research dispatch shard driver =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "task_definition=$TASK_DEFINITION"
echo "run_id=$RUN_ID"
echo "run_dir=$RUN_DIR"
echo "shard_size=$SHARD_SIZE"
echo "dry_run=$DRY_RUN"
echo

build_or_copy_base_manifest
require_absolute_file "base manifest" "$BASE_MANIFEST_OUTPUT"
require_absolute_file "base manifest summary" "$BASE_MANIFEST_SUMMARY_OUTPUT"

total_candidate_count="$(jq '.candidate_bundle_refs | length' "$BASE_MANIFEST_OUTPUT")"
if [[ "$total_candidate_count" == "0" ]]; then
  echo "base manifest has no candidate_bundle_refs" >&2
  exit 1
fi
shard_count=$(( (total_candidate_count + SHARD_SIZE - 1) / SHARD_SIZE ))

if ! bool_is_true "$DRY_RUN"; then
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
  RESEARCH_BUCKET="$(task_env_value "$task_json" "RESEARCH_OUTPUT_S3_BUCKET")"
  if [[ -z "$RESEARCH_BUCKET" || "$RESEARCH_BUCKET" == "null" ]]; then
    echo "RESEARCH_OUTPUT_S3_BUCKET is missing from task definition" >&2
    exit 1
  fi
else
  ECS_CLUSTER=""
  RESEARCH_BUCKET=""
fi

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
    jq -n -c \
      --arg shard_id "$shard_id" \
      --arg manifest_file "$shard_manifest" \
      --arg s3_key "$shard_key" \
      --argjson candidate_count "$shard_candidate_count" \
      '{
        shard_id:$shard_id,
        manifest_file:$manifest_file,
        manifest_s3_key:$s3_key,
        candidate_count:$candidate_count,
        dry_run:true,
        task_arn:null,
        exit_code:null
      }' >> "$task_summary_jsonl"
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
  jq -r \
    '.tasks[0]
      | "exit_code=\(.containers[0].exitCode) reason=\(.containers[0].reason // "none") image_digest=\(.containers[0].imageDigest)"' \
    <<<"$task_result" \
    | redact
  exit_code="$(jq -r '.tasks[0].containers[0].exitCode' <<<"$task_result")"
  if [[ "$exit_code" != "0" ]]; then
    echo "shard $shard_id failed with exit_code=$exit_code" >&2
    exit 1
  fi
done

report_summary_jsonl="${RUN_DIR}/research-report-summaries.jsonl"
if bool_is_true "$DRY_RUN"; then
  : > "$report_summary_jsonl"
else
  collect_report_summary "$dispatch_started_at" "$report_summary_jsonl"
fi

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg run_id "$RUN_ID" \
  --arg run_dir "$RUN_DIR" \
  --arg base_manifest_file "$BASE_MANIFEST_OUTPUT" \
  --arg base_manifest_summary_file "$BASE_MANIFEST_SUMMARY_OUTPUT" \
  --arg task_summary_file "$task_summary_jsonl" \
  --arg report_summary_file "$report_summary_jsonl" \
  --argjson shard_size "$SHARD_SIZE" \
  --argjson shard_count "$shard_count" \
  --argjson candidate_count "$total_candidate_count" \
  --argjson dry_run "$(bool_is_true "$DRY_RUN" && echo true || echo false)" \
  --slurpfile manifest_summary_input "$BASE_MANIFEST_SUMMARY_OUTPUT" \
  --slurpfile task_summary_input "$task_summary_jsonl" \
  --slurpfile report_summary_input "$report_summary_jsonl" \
  '($manifest_summary_input[0] // {}) as $manifest_summary
  | ($task_summary_input // []) as $tasks
  | ($report_summary_input // []) as $reports
  | {
      schema_version:"research_dispatch_shard_driver_summary_v1",
      generated_at:$generated_at,
      run_id:$run_id,
      run_dir:$run_dir,
      base_manifest_file:$base_manifest_file,
      base_manifest_summary_file:$base_manifest_summary_file,
      task_summary_file:$task_summary_file,
      report_summary_file:$report_summary_file,
      shard_size:$shard_size,
      shard_count:$shard_count,
      candidate_count:$candidate_count,
      dry_run:$dry_run,
      safety:{
        current_approved_required:true,
        dispatcher_mode_changed:false,
        live_enabled:false,
        paper_live_enabled:false,
        order_execution_enabled:false
      },
      manifest:{
        universe_mode:($manifest_summary.universe_mode // null),
        selected_candidate_count:($manifest_summary.selected_candidate_count // $candidate_count),
        current_approved_candidate_count:($manifest_summary.current_approved_candidate_count // null),
        latest_universe:($manifest_summary.latest_universe // null)
      },
      tasks:{
        total:($tasks | length),
        succeeded:($tasks | map(select(.exit_code == 0 or .dry_run == true)) | length),
        failed:($tasks | map(select(.dry_run != true and .exit_code != 0)) | length),
        exit_codes:($tasks | map(.exit_code) | unique)
      },
      reports:{
        total:($reports | length),
        statuses:($reports | map(.research_run_status) | unique),
        total_source_candidates:($reports | map(.source_candidate_count) | add // 0),
        total_replay_runs:($reports | map(.replay_run_count) | add // 0),
        symbols:($reports | map(.partition_symbols[]?) | unique | sort),
        symbol_count:($reports | map(.partition_symbols[]?) | unique | length),
        gate_biases:($reports | map(.gate_biases[]?) | unique | sort),
        shadow_validation_total:($reports | map(.shadow_validation_count) | add // 0),
        paper_trade_candidate_total:($reports | map(.paper_trade_candidate_count) | add // 0)
      }
    }' > "$SUMMARY_OUTPUT"

echo
echo "dispatch_shard_summary=$SUMMARY_OUTPUT"
jq -r '
  "candidate_count=\(.candidate_count)",
  "shard_size=\(.shard_size)",
  "shard_count=\(.shard_count)",
  "task_succeeded=\(.tasks.succeeded)",
  "task_failed=\(.tasks.failed)",
  "report_count=\(.reports.total)",
  "total_source_candidates=\(.reports.total_source_candidates)",
  "total_replay_runs=\(.reports.total_replay_runs)",
  "symbol_count=\(.reports.symbol_count)",
  "gate_biases=\(.reports.gate_biases | join(","))",
  "shadow_validation_total=\(.reports.shadow_validation_total)",
  "paper_trade_candidate_total=\(.reports.paper_trade_candidate_total)"
' "$SUMMARY_OUTPUT" | redact

echo "research current-approved dispatch shard driver completed"
