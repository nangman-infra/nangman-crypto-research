#!/usr/bin/env bash
set -euo pipefail

APP_NAME="research-app"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
UNIVERSE_MODE="${RESEARCH_BATCH_UNIVERSE_MODE:-current_approved}"
RUN_ID="${RESEARCH_BATCH_DRIVER_RUN_ID:-research_batch_$(date -u +%Y%m%dT%H%M%SZ)}"
DRIVER_ROOT="${RESEARCH_BATCH_DRIVER_ROOT:-/tmp/nangman-crypto/research-current-approved-batch}"
RUN_DIR="${RESEARCH_BATCH_DRIVER_RUN_DIR:-${DRIVER_ROOT%/}/${RUN_ID}}"
MANIFEST_OUTPUT="${RESEARCH_BATCH_MANIFEST_OUTPUT:-${RUN_DIR}/research-input-manifest.json}"
MANIFEST_SUMMARY_OUTPUT="${RESEARCH_BATCH_SUMMARY_OUTPUT:-${RUN_DIR}/research-input-manifest.summary.json}"
RESEARCH_OUTPUT_DIR="${RESEARCH_BATCH_DRIVER_OUTPUT_DIR:-${RUN_DIR}/research-output}"
REPORT_SUMMARY_OUTPUT="${RESEARCH_BATCH_DRIVER_REPORT_SUMMARY_OUTPUT:-${RUN_DIR}/research-report-summary.json}"
RETEST_HORIZON_PLAN_OUTPUT="${RESEARCH_BATCH_DRIVER_RETEST_HORIZON_PLAN_OUTPUT:-${RUN_DIR}/retest-horizon-plan.json}"
RETEST_HORIZON_STATUS_OUTPUT="${RESEARCH_BATCH_DRIVER_RETEST_HORIZON_STATUS_OUTPUT:-${RUN_DIR}/retest-horizon-status.json}"
DRIVER_SUMMARY_OUTPUT="${RESEARCH_BATCH_DRIVER_SUMMARY_OUTPUT:-${RUN_DIR}/batch-driver-summary.json}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd "${script_dir}/.." && pwd)"
# shellcheck source=scripts/lib/runtime-common.sh
source "$script_dir/lib/runtime-common.sh"
# shellcheck source=scripts/lib/research-current-batch-runtime.sh
source "$script_dir/lib/research-current-batch-runtime.sh"
# shellcheck source=scripts/lib/research-current-batch-credentials.sh
source "$script_dir/lib/research-current-batch-credentials.sh"
# shellcheck source=scripts/lib/research-current-batch-artifacts.sh
source "$script_dir/lib/research-current-batch-artifacts.sh"
# shellcheck source=scripts/lib/research-current-batch-output.sh
source "$script_dir/lib/research-current-batch-output.sh"
# shellcheck source=scripts/lib/research-current-batch-driver.sh
source "$script_dir/lib/research-current-batch-driver.sh"

if [[ "${RESEARCH_BATCH_DRIVER_SELF_TEST:-false}" == "true" ]]; then
  credential_loader_self_test
  exit 0
fi

validate_current_approved_batch_driver_inputs
prepare_current_approved_batch_run
build_current_approved_batch_manifest
prepare_current_approved_market_l1_bucket
run_current_approved_local_research_replay
summarize_current_approved_research_report
build_current_approved_retest_horizon_plan
finalize_current_approved_batch_driver_summary
build_current_approved_retest_horizon_status

echo
print_current_approved_batch_driver_result
