#!/usr/bin/env bash

validate_current_approved_batch_driver_inputs() {
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
}

prepare_current_approved_batch_run() {
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
}

build_current_approved_batch_manifest() {
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
}

prepare_current_approved_market_l1_bucket() {
  market_l1_bucket="$(discover_market_l1_bucket)"
  if [[ -z "$market_l1_bucket" || "$market_l1_bucket" == "null" ]]; then
    echo "RESEARCH_MARKET_L1_S3_BUCKET is not set and could not be discovered from the task definition" >&2
    exit 1
  fi
  export RESEARCH_MARKET_L1_S3_BUCKET="$market_l1_bucket"
  prepare_aws_sdk_credentials
}

run_current_approved_local_research_replay() {
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
}

summarize_current_approved_research_report() {
  echo
  echo "== local research report summary =="
  if [[ -n "$registry_file" ]]; then
    "${script_dir}/summarize-research-report.sh" "$report_file" "$registry_file" > "$REPORT_SUMMARY_OUTPUT"
  else
    "${script_dir}/summarize-research-report.sh" "$report_file" > "$REPORT_SUMMARY_OUTPUT"
  fi
  require_absolute_file "research report summary output" "$REPORT_SUMMARY_OUTPUT"
  print_current_approved_report_summary
}

build_current_approved_retest_horizon_plan() {
  echo
  echo "== retest horizon plan =="
  "${script_dir}/build-retest-horizon-plan.sh" "$MANIFEST_OUTPUT" "$report_file" > "$RETEST_HORIZON_PLAN_OUTPUT"
  require_absolute_file "retest horizon plan output" "$RETEST_HORIZON_PLAN_OUTPUT"
  print_current_approved_retest_horizon_plan_summary
}

build_current_approved_retest_horizon_status() {
  echo
  echo "== retest horizon status checkpoint =="
  "${script_dir}/summarize-retest-horizon-status.sh" "$RETEST_HORIZON_PLAN_OUTPUT" "$DRIVER_SUMMARY_OUTPUT" > "$RETEST_HORIZON_STATUS_OUTPUT"
  require_absolute_file "retest horizon status output" "$RETEST_HORIZON_STATUS_OUTPUT"
  print_current_approved_retest_horizon_status_summary
}

finalize_current_approved_batch_driver_summary() {
  write_current_approved_batch_driver_summary "$report_file" "$registry_file"
  require_absolute_file "batch driver summary output" "$DRIVER_SUMMARY_OUTPUT"
}
