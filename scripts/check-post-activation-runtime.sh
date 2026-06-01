#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
CLUSTER_NAME="${RESEARCH_ECS_CLUSTER:-ecs-nangman-dev-invest-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
EXPECTED_DISPATCH_MODE="${RESEARCH_EXPECTED_DISPATCH_MODE:-run_task}"
VERIFY_FRESH_OUTPUT="${RESEARCH_VERIFY_FRESH_OUTPUT:-true}"
OUTPUT_MIN_LAST_MODIFIED="${RESEARCH_OUTPUT_MIN_LAST_MODIFIED:-}"
JQ_DIR="$SCRIPT_DIR/jq"
# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/post-activation-runtime-checks.sh
source "$SCRIPT_DIR/lib/post-activation-runtime-checks.sh"

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

verify_aws_access

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
if [[ "$history_index_prefix" == "replay-run-index/" ]]; then
  echo "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX uses the broad replay-run-index/ prefix; remove it or set a narrowed prefix" >&2
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
  jq -c -f "$(post_activation_runtime_jq activation-readiness-latest-object-display.jq)" <<<"$object_json" | redact
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
