#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck source=scripts/lib/runtime-common.sh
source "$SCRIPT_DIR/lib/runtime-common.sh"
# shellcheck source=scripts/lib/research-dispatch-shards-runtime.sh
source "$SCRIPT_DIR/lib/research-dispatch-shards-runtime.sh"
# shellcheck source=scripts/lib/research-dispatch-shards-output.sh
source "$SCRIPT_DIR/lib/research-dispatch-shards-output.sh"

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
  load_dispatch_runtime_config
else
  ECS_CLUSTER=""
  RESEARCH_BUCKET=""
fi

run_dispatch_shards
collect_post_dispatch_reports

write_dispatch_shard_driver_summary

echo
echo "dispatch_shard_summary=$SUMMARY_OUTPUT"
print_dispatch_shard_driver_summary | redact

echo "research current-approved dispatch shard driver completed"
