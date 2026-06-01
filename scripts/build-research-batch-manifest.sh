#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/research-batch-manifest-inputs.sh
source "$SCRIPT_DIR/lib/research-batch-manifest-inputs.sh"
# shellcheck source=scripts/lib/research-batch-manifest-output.sh
source "$SCRIPT_DIR/lib/research-batch-manifest-output.sh"
# shellcheck source=scripts/lib/research-batch-manifest-validation.sh
source "$SCRIPT_DIR/lib/research-batch-manifest-validation.sh"
# shellcheck source=scripts/lib/research-batch-manifest-runtime.sh
source "$SCRIPT_DIR/lib/research-batch-manifest-runtime.sh"

APP_NAME="research-app"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
CANDIDATE_READ_LIMIT="${RESEARCH_BATCH_CANDIDATE_READ_LIMIT:-1000}"
MAX_CANDIDATE_BUNDLE_COUNT="${RESEARCH_BATCH_MAX_CANDIDATE_BUNDLE_COUNT:-1000}"
HISTORICAL_INDEX_READ_LIMIT="${RESEARCH_BATCH_HISTORICAL_INDEX_READ_LIMIT:-20}"
MAX_HISTORICAL_REPLAY_RUN_REF_COUNT="${RESEARCH_BATCH_MAX_HISTORICAL_REPLAY_RUN_REF_COUNT:-10000}"
MAX_REPLAY_RUN_COUNT="${RESEARCH_BATCH_MAX_REPLAY_RUN_COUNT:-20000}"
UNIVERSE_MODE="${RESEARCH_BATCH_UNIVERSE_MODE:-current_approved}"
RUN_SCOPE="${RESEARCH_BATCH_RUN_SCOPE:-recent_candidate_batch_${UNIVERSE_MODE}_local_validation}"
RESEARCH_PACKET_ID="${RESEARCH_BATCH_PACKET_ID:-research_packet_$(date -u +%Y%m%dT%H%M%SZ)}"

validate_research_batch_manifest_config
prepare_research_batch_manifest_outputs
print_research_batch_manifest_header
verify_aws_access
prepare_research_batch_manifest_tempfiles
discover_research_batch_runtime_config
print_research_batch_runtime_config
run_research_batch_manifest_build
