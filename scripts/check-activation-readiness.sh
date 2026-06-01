#!/usr/bin/env bash
set -euo pipefail

APP_NAME="research-app"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
CLUSTER_NAME="${RESEARCH_ECS_CLUSTER:-ecs-nangman-dev-invest-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
EXPECTED_DISPATCH_MODE="${RESEARCH_EXPECTED_DISPATCH_MODE:-dry_run}"

JQ_DIR="$SCRIPT_DIR/jq"
# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/activation-readiness-checks.sh
source "$SCRIPT_DIR/lib/activation-readiness-checks.sh"
# shellcheck source=scripts/lib/activation-readiness-actions.sh
source "$SCRIPT_DIR/lib/activation-readiness-actions.sh"

require_command aws
require_command jq
require_command sed
require_command mktemp

print_activation_readiness_header
verify_aws_access

lambda_json="$(mktemp)"
task_json="$(mktemp)"
invoke_payload=""
invoke_output=""
trap 'rm -f "$lambda_json" "$task_json" ${invoke_payload:+"$invoke_payload"} ${invoke_output:+"$invoke_output"}' EXIT

load_dispatcher_configuration
validate_dispatcher_configuration
echo "dispatcher ok: state=$lambda_state update=$lambda_update_status mode=$dispatch_mode"

load_task_definition
validate_task_definition
print_task_definition_summary
run_activation_dry_run_if_requested
assert_no_dispatcher_tasks
print_latest_research_objects

echo "activation readiness check completed"
