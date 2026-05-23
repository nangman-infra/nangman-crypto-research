#!/usr/bin/env bash
set -euo pipefail

REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
CLUSTER_NAME="${RESEARCH_ECS_CLUSTER:-ecs-nangman-dev-invest-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
EXPECTED_DISPATCH_MODE="${RESEARCH_EXPECTED_DISPATCH_MODE:-run_task}"
VERIFY_FRESH_OUTPUT="${RESEARCH_VERIFY_FRESH_OUTPUT:-true}"
OUTPUT_MIN_LAST_MODIFIED="${RESEARCH_OUTPUT_MIN_LAST_MODIFIED:-}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

redact() {
  sed -E \
    -e 's/[0-9]{12}/<aws-account-id>/g' \
    -e 's/nangman-crypto-dev-[A-Za-z0-9-]+-[0-9]{6}/nangman-crypto-dev-<bucket-family>-<account-suffix>/g' \
    -e 's#arn:aws:iam::[^[:space:]"]+#arn:aws:iam::<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:ecs:[^[:space:]"]+#arn:aws:ecs:<region>:<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:lambda:[^[:space:]"]+#arn:aws:lambda:<region>:<aws-account-id>:<resource>#g' \
    -e 's/subnet-[A-Za-z0-9]+/<subnet-id>/g' \
    -e 's/sg-[A-Za-z0-9]+/<security-group-id>/g'
}

aws_cmd() {
  aws --region "$REGION" "$@"
}

latest_object_json() {
  local bucket="$1"
  local prefix="$2"
  aws_cmd s3api list-objects-v2 \
    --bucket "$bucket" \
    --prefix "$prefix" \
    --query 'sort_by(Contents || `[]`, &LastModified)[-1].{prefix:`'"$prefix"'`,lastModified:LastModified,size:Size,key:Key}' \
    --output json
}

require_fresh_object() {
  local bucket="$1"
  local prefix="$2"
  local object_json="$3"
  local key
  local last_modified
  local size

  key="$(jq -r '.key // empty' <<<"$object_json")"
  last_modified="$(jq -r '.lastModified // empty' <<<"$object_json")"
  size="$(jq -r '.size // 0' <<<"$object_json")"
  if [[ -z "$key" || -z "$last_modified" || "$size" == "0" ]]; then
    echo "missing or empty required output prefix: s3://${bucket}/${prefix}" | redact >&2
    exit 1
  fi
  if [[ -n "$OUTPUT_MIN_LAST_MODIFIED" && "$last_modified" < "$OUTPUT_MIN_LAST_MODIFIED" ]]; then
    echo "stale output prefix: s3://${bucket}/${prefix} latest=${last_modified} min=${OUTPUT_MIN_LAST_MODIFIED}" | redact >&2
    exit 1
  fi
}

validate_report_sample() {
  local bucket="$1"
  local key="$2"
  local report_json
  report_json="$(aws_cmd s3 cp "s3://${bucket}/${key}" -)"

  jq -e '
    .schema_version == "research_run_report_v1"
    and (.research_run_report_id | type == "string" and length > 0)
    and (.source_candidate_ids | type == "array" and length > 0)
    and (.replay_run_ids | type == "array" and length > 0)
    and (.partition_aggregates | type == "array")
    and (.research_gate_policy.policy_version | type == "string" and length > 0)
  ' <<<"$report_json" >/dev/null

  jq -c '{
    schema_version,
    research_run_report_id,
    source_candidate_count:(.source_candidate_ids | length),
    replay_run_count:(.replay_run_ids | length),
    partition_count,
    top_symbols,
    surviving_candidate_count:(.surviving_candidate_keys | length),
    retest_candidate_count:(.retest_candidate_keys | length),
    pruned_candidate_count:(.pruned_candidate_keys | length),
    shadow_validation_count:(.shadow_validation_runs | length),
    paper_trade_candidate_count:(.paper_trade_candidates | length)
  }' <<<"$report_json" | redact
}

require_command aws
require_command jq
require_command sed
require_command mktemp

echo "== research-app post-activation runtime check =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "cluster=$CLUSTER_NAME"
echo "task_definition=$TASK_DEFINITION"
echo "expected_dispatch_mode=$EXPECTED_DISPATCH_MODE"
echo "verify_fresh_output=$VERIFY_FRESH_OUTPUT"
echo

lambda_json="$(mktemp)"
task_json="$(mktemp)"
trap 'rm -f "$lambda_json" "$task_json"' EXIT

aws_cmd lambda get-function-configuration \
  --function-name "$DISPATCHER_FUNCTION" \
  --output json > "$lambda_json"

lambda_state="$(jq -r '.State' "$lambda_json")"
lambda_update_status="$(jq -r '.LastUpdateStatus' "$lambda_json")"
dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
lambda_task_definition="$(jq -r '.Environment.Variables.ECS_TASK_DEFINITION // ""' "$lambda_json")"
lambda_container="$(jq -r '.Environment.Variables.ECS_CONTAINER_NAME // ""' "$lambda_json")"

if [[ "$lambda_state" != "Active" || "$lambda_update_status" != "Successful" ]]; then
  echo "dispatcher Lambda is not ready: state=$lambda_state update=$lambda_update_status" >&2
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
echo "dispatcher ok: state=$lambda_state update=$lambda_update_status mode=$dispatch_mode"

aws_cmd ecs describe-task-definition \
  --task-definition "$TASK_DEFINITION" \
  --output json > "$task_json"

task_revision="$(jq -r '.taskDefinition.revision' "$task_json")"
task_status="$(jq -r '.taskDefinition.status' "$task_json")"
cpu_arch="$(jq -r '.taskDefinition.runtimePlatform.cpuArchitecture' "$task_json")"
os_family="$(jq -r '.taskDefinition.runtimePlatform.operatingSystemFamily' "$task_json")"
readonly_root="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | .readonlyRootFilesystem' "$task_json")"
output_bucket="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | (.environment // [])[]? | select(.name == "RESEARCH_OUTPUT_S3_BUCKET") | .value' "$task_json")"
history_index_prefix="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | (.environment // [])[]? | select(.name == "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX") | .value' "$task_json")"

if [[ "$task_status" != "ACTIVE" || "$cpu_arch" != "ARM64" || "$os_family" != "LINUX" || "$readonly_root" != "true" ]]; then
  echo "task definition runtime contract failed: status=$task_status platform=${cpu_arch}/${os_family} readonly=$readonly_root" >&2
  exit 1
fi
if [[ -z "$output_bucket" || "$output_bucket" == "null" ]]; then
  echo "RESEARCH_OUTPUT_S3_BUCKET is missing from task definition" >&2
  exit 1
fi
if [[ -z "$history_index_prefix" || "$history_index_prefix" == "null" ]]; then
  echo "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX is missing from task definition" >&2
  exit 1
fi
echo "task ok: ${TASK_DEFINITION}:${task_revision} ${cpu_arch}/${os_family} readonly=${readonly_root}" | redact

running_dispatch_tasks="$(aws_cmd ecs list-tasks \
  --cluster "$CLUSTER_NAME" \
  --desired-status RUNNING \
  --started-by research-s3-dispatcher \
  --query 'length(taskArns)' \
  --output text)"
pending_dispatch_tasks="$(aws_cmd ecs list-tasks \
  --cluster "$CLUSTER_NAME" \
  --desired-status PENDING \
  --started-by research-s3-dispatcher \
  --query 'length(taskArns)' \
  --output text)"
echo "dispatcher task counts: running=$running_dispatch_tasks pending=$pending_dispatch_tasks"

report_json="$(latest_object_json "$output_bucket" "research-run-report/")"
replay_json="$(latest_object_json "$output_bucket" "replay-run/")"
index_json="$(latest_object_json "$output_bucket" "replay-run-index/")"
shadow_json="$(latest_object_json "$output_bucket" "shadow-validation-run/")"
paper_json="$(latest_object_json "$output_bucket" "paper-trade-run/")"

echo "latest output prefixes:"
for object_json in "$report_json" "$replay_json" "$index_json" "$shadow_json" "$paper_json"; do
  jq -c 'if .key == null then {prefix:.prefix,lastModified:null,size:null,key:null} else {prefix:.prefix,lastModified:.lastModified,size:.size,key:(.key | split("/") | .[0:4] | join("/") + "/...")} end' <<<"$object_json" | redact
done

if [[ "$VERIFY_FRESH_OUTPUT" == "true" ]]; then
  require_fresh_object "$output_bucket" "research-run-report/" "$report_json"
  require_fresh_object "$output_bucket" "replay-run/" "$replay_json"
  require_fresh_object "$output_bucket" "replay-run-index/" "$index_json"
  report_key="$(jq -r '.key' <<<"$report_json")"
  echo "research report sample:"
  validate_report_sample "$output_bucket" "$report_key"
else
  echo "fresh output verification skipped"
fi

echo "post-activation runtime check completed"
