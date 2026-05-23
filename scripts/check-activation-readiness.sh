#!/usr/bin/env bash
set -euo pipefail

APP_NAME="research-app"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
CLUSTER_NAME="${RESEARCH_ECS_CLUSTER:-ecs-nangman-dev-invest-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
EXPECTED_DISPATCH_MODE="${RESEARCH_EXPECTED_DISPATCH_MODE:-dry_run}"

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

require_command aws
require_command jq
require_command sed
require_command mktemp

echo "== ${APP_NAME} activation readiness =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "cluster=$CLUSTER_NAME"
echo "task_definition=$TASK_DEFINITION"
echo

lambda_json="$(mktemp)"
task_json="$(mktemp)"
invoke_payload=""
invoke_output=""
trap 'rm -f "$lambda_json" "$task_json" ${invoke_payload:+"$invoke_payload"} ${invoke_output:+"$invoke_output"}' EXIT

aws_cmd lambda get-function-configuration \
  --function-name "$DISPATCHER_FUNCTION" \
  --output json > "$lambda_json"

lambda_state="$(jq -r '.State' "$lambda_json")"
lambda_update_status="$(jq -r '.LastUpdateStatus' "$lambda_json")"
dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
lambda_task_definition="$(jq -r '.Environment.Variables.ECS_TASK_DEFINITION // ""' "$lambda_json")"
lambda_container="$(jq -r '.Environment.Variables.ECS_CONTAINER_NAME // ""' "$lambda_json")"

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

echo "dispatcher ok: state=$lambda_state update=$lambda_update_status mode=$dispatch_mode"

aws_cmd ecs describe-task-definition \
  --task-definition "$TASK_DEFINITION" \
  --output json > "$task_json"

task_revision="$(jq -r '.taskDefinition.revision' "$task_json")"
task_status="$(jq -r '.taskDefinition.status' "$task_json")"
cpu_arch="$(jq -r '.taskDefinition.runtimePlatform.cpuArchitecture' "$task_json")"
os_family="$(jq -r '.taskDefinition.runtimePlatform.operatingSystemFamily' "$task_json")"
readonly_root="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | .readonlyRootFilesystem' "$task_json")"
image="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | .image' "$task_json")"
output_bucket="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | (.environment // [])[]? | select(.name == "RESEARCH_OUTPUT_S3_BUCKET") | .value' "$task_json")"
market_l1_bucket="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | (.environment // [])[]? | select(.name == "RESEARCH_MARKET_L1_S3_BUCKET") | .value' "$task_json")"

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

{
  echo "task ok: ${TASK_DEFINITION}:${task_revision} ${cpu_arch}/${os_family} readonly=${readonly_root}"
  echo "image=$image"
  echo "output_bucket=$output_bucket"
  echo "market_l1_bucket=$market_l1_bucket"
} | redact

if [[ -n "${RESEARCH_DRY_RUN_BUCKET:-}" || -n "${RESEARCH_DRY_RUN_KEY:-}" ]]; then
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
    '{
      Records: [{
        eventSource: "aws:s3",
        eventName: "ObjectCreated:Put",
        eventTime: "2026-05-23T00:00:00.000Z",
        s3: {
          bucket: { name: $bucket },
          object: {
            key: $key,
            eTag: "activation-readiness",
            sequencer: "0000000000000001"
          }
        }
      }]
    }' > "$invoke_payload"

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
fi

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
  | jq -c 'if .key == null then {prefix:.prefix,lastModified:null,size:null,key:null} else {prefix:.prefix,lastModified:.lastModified,size:.size,key:(.key | split("/") | .[0:4] | join("/") + "/...")} end' \
  | redact
done

echo "activation readiness check completed"
