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

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
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

task_env_value() {
  local name="$1"
  aws_cmd ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --output json \
  | jq -r \
      --arg container "$CONTAINER_NAME" \
      --arg name "$name" \
      '.taskDefinition.containerDefinitions[]
        | select(.name == $container)
        | (.environment // [])[]?
        | select(.name == $name)
        | .value' \
  | head -n 1
}

discover_market_l1_bucket() {
  if [[ -n "${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}" ]]; then
    printf '%s\n' "${RESEARCH_MARKET_L1_S3_BUCKET:-${MARKET_L1_BUCKET:-}}"
    return
  fi
  task_env_value "RESEARCH_MARKET_L1_S3_BUCKET"
}

prepare_aws_sdk_credentials() {
  if [[ -n "${AWS_ACCESS_KEY_ID:-}" && -n "${AWS_SECRET_ACCESS_KEY:-}" ]]; then
    return
  fi
  if [[ -z "${AWS_PROFILE:-}" ]]; then
    return
  fi

  local credential_env_file
  credential_env_file="${RUN_DIR}/aws-exported-credentials.env"
  rm -f "$credential_env_file"
  if ! aws configure export-credentials \
    --profile "$AWS_PROFILE" \
    --format env-no-export > "$credential_env_file"; then
    rm -f "$credential_env_file"
    echo "failed to export AWS CLI credentials for AWS_PROFILE=$AWS_PROFILE" >&2
    exit 1
  fi
  chmod 600 "$credential_env_file"
  set -a
  # shellcheck source=/dev/null
  source "$credential_env_file"
  set +a
  rm -f "$credential_env_file"
  export AWS_ACCESS_KEY_ID
  export AWS_SECRET_ACCESS_KEY
  export AWS_SESSION_TOKEN
  export AWS_CREDENTIAL_EXPIRATION
}

find_latest_report_file() {
  if [[ ! -d "$RESEARCH_OUTPUT_DIR/research-run-report" ]]; then
    return 0
  fi
  find "$RESEARCH_OUTPUT_DIR/research-run-report" \
    -type f \
    -name "report.json" \
    -print 2>/dev/null \
  | sort \
  | tail -n 1
}

find_latest_registry_file() {
  if [[ ! -d "$RESEARCH_OUTPUT_DIR/research-aggregate-registry" ]]; then
    return 0
  fi
  find "$RESEARCH_OUTPUT_DIR/research-aggregate-registry" \
    -type f \
    -name "part-000001.jsonl" \
    -print 2>/dev/null \
  | sort \
  | tail -n 1
}

require_command aws
require_command cargo
require_command date
require_command find
require_command jq
require_command mkdir
require_command sed
require_command tee

require_absolute_path "RESEARCH_BATCH_DRIVER_ROOT" "$DRIVER_ROOT"
require_absolute_path "RESEARCH_BATCH_DRIVER_RUN_DIR" "$RUN_DIR"
require_absolute_path "RESEARCH_BATCH_MANIFEST_OUTPUT" "$MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_BATCH_SUMMARY_OUTPUT" "$MANIFEST_SUMMARY_OUTPUT"
require_absolute_path "RESEARCH_BATCH_DRIVER_OUTPUT_DIR" "$RESEARCH_OUTPUT_DIR"
require_absolute_path "RESEARCH_BATCH_DRIVER_REPORT_SUMMARY_OUTPUT" "$REPORT_SUMMARY_OUTPUT"
require_absolute_path "RESEARCH_BATCH_DRIVER_RETEST_HORIZON_PLAN_OUTPUT" "$RETEST_HORIZON_PLAN_OUTPUT"
require_absolute_path "RESEARCH_BATCH_DRIVER_RETEST_HORIZON_STATUS_OUTPUT" "$RETEST_HORIZON_STATUS_OUTPUT"
require_absolute_path "RESEARCH_BATCH_DRIVER_SUMMARY_OUTPUT" "$DRIVER_SUMMARY_OUTPUT"

if [[ "$UNIVERSE_MODE" != "current_approved" && "${RESEARCH_BATCH_DRIVER_ALLOW_NON_APPROVED_UNIVERSE:-false}" != "true" ]]; then
  echo "RESEARCH_BATCH_UNIVERSE_MODE must be current_approved for promotion-safe batch evidence; got $UNIVERSE_MODE" >&2
  echo "Set RESEARCH_BATCH_DRIVER_ALLOW_NON_APPROVED_UNIVERSE=true only for diagnostics." >&2
  exit 1
fi

mkdir -p "$RUN_DIR" "$RESEARCH_OUTPUT_DIR"
export RESEARCH_BATCH_UNIVERSE_MODE="$UNIVERSE_MODE"
export RESEARCH_BATCH_MANIFEST_OUTPUT="$MANIFEST_OUTPUT"
export RESEARCH_BATCH_SUMMARY_OUTPUT="$MANIFEST_SUMMARY_OUTPUT"

echo "== ${APP_NAME} current-approved research batch driver =="
echo "region=$REGION"
echo "universe_mode=$UNIVERSE_MODE"
echo "run_dir=$RUN_DIR"
echo "research_output_dir=$RESEARCH_OUTPUT_DIR"
echo "safety=s3_write:false,ecs_task_started:false,dispatcher_mode_changed:false,shadow_paper_live_enabled:false"
echo

"${script_dir}/build-research-batch-manifest.sh" 2>&1 \
| redact \
| tee "${RUN_DIR}/build-research-batch-manifest.log"

require_absolute_file "manifest output" "$MANIFEST_OUTPUT"
require_absolute_file "manifest summary output" "$MANIFEST_SUMMARY_OUTPUT"

selected_candidate_count="$(jq -r '.selected_candidate_count // 0' "$MANIFEST_SUMMARY_OUTPUT")"
if [[ "$selected_candidate_count" == "0" ]]; then
  echo "selected_candidate_count=0; no local research run was started" >&2
  exit 1
fi

market_l1_bucket="$(discover_market_l1_bucket)"
if [[ -z "$market_l1_bucket" || "$market_l1_bucket" == "null" ]]; then
  echo "RESEARCH_MARKET_L1_S3_BUCKET is not set and could not be discovered from the task definition" >&2
  exit 1
fi
export RESEARCH_MARKET_L1_S3_BUCKET="$market_l1_bucket"
prepare_aws_sdk_credentials

echo
echo "== local research replay run =="
(
  cd "$repo_dir"
  cargo run -- \
    --input-manifest-file "$MANIFEST_OUTPUT" \
    --market-l1-s3-bucket "$market_l1_bucket" \
    --output-dir "$RESEARCH_OUTPUT_DIR"
) 2>&1 \
| redact \
| tee "${RUN_DIR}/cargo-research-run.log"

report_file="$(find_latest_report_file)"
if [[ -z "$report_file" ]]; then
  echo "research report was not created under $RESEARCH_OUTPUT_DIR" >&2
  exit 1
fi
registry_file="$(find_latest_registry_file)"

echo
echo "== local research report summary =="
if [[ -n "$registry_file" ]]; then
  "${script_dir}/summarize-research-report.sh" "$report_file" "$registry_file" > "$REPORT_SUMMARY_OUTPUT"
else
  "${script_dir}/summarize-research-report.sh" "$report_file" > "$REPORT_SUMMARY_OUTPUT"
fi
require_absolute_file "research report summary output" "$REPORT_SUMMARY_OUTPUT"
jq -r '
  "report_status=\(.report.research_run_status)",
  "source_candidate_count=\(.report.source_candidate_count)",
  "replay_run_count=\(.report.replay_run_count)",
  "retest_candidate_count=\(.report.retest_candidate_count)",
  "surviving_candidate_count=\(.report.surviving_candidate_count)",
  "shadow_validation_count=\(.report.shadow_validation_count)",
  "paper_trade_candidate_count=\(.report.paper_trade_candidate_count)",
  "promotion_passed=\(.stage_state.promotion_passed)",
  "shadow_created=\(.stage_state.shadow_created)",
  "paper_created=\(.stage_state.paper_created)"
' "$REPORT_SUMMARY_OUTPUT" | redact

echo
echo "== retest horizon plan =="
"${script_dir}/build-retest-horizon-plan.sh" "$MANIFEST_OUTPUT" "$report_file" > "$RETEST_HORIZON_PLAN_OUTPUT"
require_absolute_file "retest horizon plan output" "$RETEST_HORIZON_PLAN_OUTPUT"
jq -r '
  "horizon_count=\(.summary.horizon_count)",
  "ready_for_replay_count=\(.summary.ready_for_replay_count)",
  "waiting_for_market_l1_count=\(.summary.waiting_for_market_l1_count)",
  "market_l1_coverage_extension_count=\(.summary.market_l1_coverage_extension_count)",
  "sample_accumulation_count=\(.summary.sample_accumulation_count)",
  "promotion_ready_for_review_count=\(.summary.promotion_ready_for_review_count)",
  "next_action_counts=\(.summary.next_action_counts | map(.next_action + ":" + (.count|tostring)) | join(","))"
' "$RETEST_HORIZON_PLAN_OUTPUT" | redact

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg run_id "$RUN_ID" \
  --arg run_dir "$RUN_DIR" \
  --arg manifest_file "$MANIFEST_OUTPUT" \
  --arg manifest_summary_file "$MANIFEST_SUMMARY_OUTPUT" \
  --arg research_output_dir "$RESEARCH_OUTPUT_DIR" \
  --arg report_file "$report_file" \
  --arg registry_file "$registry_file" \
  --arg report_summary_file "$REPORT_SUMMARY_OUTPUT" \
  --arg retest_horizon_plan_file "$RETEST_HORIZON_PLAN_OUTPUT" \
  --arg retest_horizon_status_file "$RETEST_HORIZON_STATUS_OUTPUT" \
  --slurpfile manifest_summary_file_input "$MANIFEST_SUMMARY_OUTPUT" \
  --slurpfile report_summary_file_input "$REPORT_SUMMARY_OUTPUT" \
  --slurpfile retest_horizon_plan_file_input "$RETEST_HORIZON_PLAN_OUTPUT" \
  '($manifest_summary_file_input[0] // {}) as $manifest_summary
  | ($report_summary_file_input[0] // {}) as $report_summary
  | ($retest_horizon_plan_file_input[0] // {}) as $retest_horizon_plan
  | {
    schema_version:"research_current_approved_batch_driver_summary_v1",
    generated_at:$generated_at,
    run_id:$run_id,
    run_dir:$run_dir,
    manifest_file:$manifest_file,
    manifest_summary_file:$manifest_summary_file,
    research_output_dir:$research_output_dir,
    report_file:$report_file,
    registry_file:(if $registry_file == "" then null else $registry_file end),
    report_summary_file:$report_summary_file,
    retest_horizon_plan_file:$retest_horizon_plan_file,
    retest_horizon_status_file:$retest_horizon_status_file,
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_research_output_only:true,
      shadow_paper_live_enabled:false,
      selected_candidates_require_current_universe:($manifest_summary.safety.selected_candidates_require_current_universe // true)
    },
    stage_state:{
      runtime_alive:null,
      artifact_created:true,
      candidate_generated:(($manifest_summary.selected_candidate_count // 0) > 0),
      research_replay_completed:($report_summary.stage_state.research_replay_completed // false),
      promotion_passed:($report_summary.stage_state.promotion_passed // false),
      shadow_created:($report_summary.stage_state.shadow_created // false),
      paper_created:($report_summary.stage_state.paper_created // false),
      live_enabled:false
    },
    manifest:{
      universe_mode:$manifest_summary.universe_mode,
      dispatch_mode:$manifest_summary.dispatch_mode,
      latest_universe:$manifest_summary.latest_universe,
      scanned_research_eligible_candidate_count:$manifest_summary.scanned_research_eligible_candidate_count,
      current_observed_candidate_count:$manifest_summary.current_observed_candidate_count,
      current_approved_candidate_count:$manifest_summary.current_approved_candidate_count,
      horizon_contract_valid_candidate_count:$manifest_summary.horizon_contract_valid_candidate_count,
      horizon_contract_invalid_candidate_count:$manifest_summary.horizon_contract_invalid_candidate_count,
      excluded_horizon_contract_violations:$manifest_summary.excluded_horizon_contract_violations,
      selected_candidate_count:$manifest_summary.selected_candidate_count,
      distinct_candidate_symbols:$manifest_summary.distinct_candidate_symbols,
      allowed_horizons:$manifest_summary.allowed_horizons,
      selected_current_approved_candidate_count:$manifest_summary.selected_current_approved_candidate_count,
      selected_horizon_contract_valid_count:$manifest_summary.selected_horizon_contract_valid_count,
      historical_replay_run_index_ref_count:$manifest_summary.historical_replay_run_index_ref_count
    },
    report:$report_summary.report,
    bias_counts:$report_summary.bias_counts,
    reason_counts:$report_summary.reason_counts,
    top_blockers:$report_summary.top_blockers,
    next_research_needs:$report_summary.next_research_needs,
    retest_horizon_plan_summary:$retest_horizon_plan.summary
  }' > "$DRIVER_SUMMARY_OUTPUT"

require_absolute_file "batch driver summary output" "$DRIVER_SUMMARY_OUTPUT"

echo
echo "== retest horizon status checkpoint =="
"${script_dir}/summarize-retest-horizon-status.sh" "$RETEST_HORIZON_PLAN_OUTPUT" "$DRIVER_SUMMARY_OUTPUT" > "$RETEST_HORIZON_STATUS_OUTPUT"
require_absolute_file "retest horizon status output" "$RETEST_HORIZON_STATUS_OUTPUT"
jq -r '
  "horizon_status_verdict=\(.next_decision.verdict)",
  "major50_observed_symbol_count=\(.major50_state.observed_symbol_count)",
  "major50_approved_symbol_count=\(.major50_state.approved_symbol_count)",
  "research_factory_blocking_stage=\(.research_factory_gap_summary.blocking_stage)",
  "approved_symbols_without_candidate_count=\(.research_factory_gap_summary.gap_counts.approved_symbols_without_candidate)",
  "candidate_count=\(.horizon_summary.candidate_count)",
  "horizon_count=\(.horizon_summary.horizon_count)",
  "symbols=\(.horizon_summary.symbols | join(","))",
  "market_l1_coverage_extension_count=\(.horizon_summary.market_l1_coverage_extension_count)",
  "next_action_counts=\(.horizon_summary.next_action_counts | map(.next_action + ":" + (.count|tostring)) | join(","))",
  "blocked_actions=\(.next_decision.blocked_actions | join(","))"
' "$RETEST_HORIZON_STATUS_OUTPUT" | redact

echo
{
  echo "batch_driver_summary=$DRIVER_SUMMARY_OUTPUT"
  echo "retest_horizon_status=$RETEST_HORIZON_STATUS_OUTPUT"
  jq -r '
    "selected_candidate_count=\(.manifest.selected_candidate_count)",
    "current_approved_candidate_count=\(.manifest.current_approved_candidate_count)",
    "horizon_contract_invalid_candidate_count=\(.manifest.horizon_contract_invalid_candidate_count)",
    "distinct_candidate_symbols=\(.manifest.distinct_candidate_symbols | join(","))",
    "research_replay_completed=\(.stage_state.research_replay_completed)",
    "promotion_passed=\(.stage_state.promotion_passed)",
    "shadow_created=\(.stage_state.shadow_created)",
    "paper_created=\(.stage_state.paper_created)",
    "live_enabled=\(.stage_state.live_enabled)",
    "promotion_ready_for_review_count=\(.retest_horizon_plan_summary.promotion_ready_for_review_count)"
  ' "$DRIVER_SUMMARY_OUTPUT"
  jq -r '
    "research_factory_blocking_stage=\(.research_factory_gap_summary.blocking_stage)",
    "approved_symbols_without_candidate_count=\(.research_factory_gap_summary.gap_counts.approved_symbols_without_candidate)",
    "candidate_ids_without_replay_count=\(.research_factory_gap_summary.gap_counts.candidate_ids_without_replay)"
  ' "$RETEST_HORIZON_STATUS_OUTPUT"
  echo "research current-approved batch driver completed"
} | redact
