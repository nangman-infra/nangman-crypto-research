#!/usr/bin/env bash

research_batch_tmp_files=()

cleanup_research_batch_manifest_tempfiles() {
  if [[ "${#research_batch_tmp_files[@]}" -gt 0 ]]; then
    rm -f "${research_batch_tmp_files[@]}"
  fi
}

prepare_research_batch_manifest_tempfiles() {
  lambda_json="$(mktemp)"
  task_json="$(mktemp)"
  candidate_p0_json="$(mktemp)"
  candidate_p1_json="$(mktemp)"
  candidate_p2_json="$(mktemp)"
  candidate_objects_json="$(mktemp)"
  candidate_records_json="$(mktemp)"
  selected_candidates_json="$(mktemp)"
  historical_index_objects_json="$(mktemp)"
  universe_object_json="$(mktemp)"
  universe_summary_json="$(mktemp)"
  all_candidates_json="$(mktemp)"

  research_batch_tmp_files=(
    "$lambda_json"
    "$task_json"
    "$candidate_p0_json"
    "$candidate_p1_json"
    "$candidate_p2_json"
    "$candidate_objects_json"
    "$candidate_records_json"
    "$selected_candidates_json"
    "$historical_index_objects_json"
    "$universe_object_json"
    "$universe_summary_json"
    "$all_candidates_json"
  )
  trap cleanup_research_batch_manifest_tempfiles EXIT
}

discover_research_batch_runtime_config() {
  aws_cmd lambda get-function-configuration \
    --function-name "$DISPATCHER_FUNCTION" \
    --output json > "$lambda_json"

  aws_cmd ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --output json > "$task_json"

  dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
  candidate_bucket="${RESEARCH_CANDIDATE_S3_BUCKET:-$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKET // ""' "$lambda_json")}"
  if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
    candidate_bucket="$(first_csv_value_containing "$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKETS // ""' "$lambda_json")" "intel-candidate")"
  fi
  output_bucket="$(task_env_value RESEARCH_OUTPUT_S3_BUCKET)"
  market_l1_bucket="$(task_env_value RESEARCH_MARKET_L1_S3_BUCKET)"

  if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
    echo "candidate bucket is not discoverable; set RESEARCH_CANDIDATE_S3_BUCKET" >&2
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
}

print_research_batch_runtime_config() {
  {
    echo "dispatcher_mode=$dispatch_mode"
    echo "candidate_bucket=$candidate_bucket"
    echo "market_l1_bucket=$market_l1_bucket"
    echo "research_output_bucket=$output_bucket"
  } | redact
}

run_research_batch_manifest_build() {
  write_latest_research_batch_universe_summary
  print_latest_research_batch_universe_summary
  collect_research_batch_candidate_objects
  collect_research_batch_candidate_records
  write_research_batch_candidate_pool
  select_research_batch_candidates

  selected_candidate_count="$(jq 'length' "$selected_candidates_json")"
  if [[ "$selected_candidate_count" == "0" ]]; then
    write_empty_research_batch_outputs
    print_empty_research_batch_summary | redact
    echo "no candidate bundles matched RESEARCH_BATCH_UNIVERSE_MODE=$UNIVERSE_MODE" >&2
    exit 1
  fi

  write_historical_replay_run_index_refs
  write_research_batch_manifest
  write_research_batch_summary
  print_research_batch_summary | redact

  echo "research batch manifest build completed"
}
