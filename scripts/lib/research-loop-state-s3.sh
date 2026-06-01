#!/usr/bin/env bash

RESEARCH_LOOP_STATE_S3_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RESEARCH_LOOP_STATE_S3_JQ_DIR="$(cd -- "$RESEARCH_LOOP_STATE_S3_LIB_DIR/../jq" && pwd -P)"

research_loop_state_s3_jq() {
  local name="$1"
  local path="$RESEARCH_LOOP_STATE_S3_JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing research loop state jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

build_research_loop_universe_summary() {
  local market_l1_bucket="$1"
  local expected_major_universe_size="$2"
  local universe_object
  local universe_key

  universe_object="$(latest_universe_snapshot_object_json "$market_l1_bucket" "symbol_universe_snapshot/run_id=")"
  universe_key="$(jq -r '.key // empty' <<<"$universe_object")"
  if [[ -z "$universe_key" ]]; then
    jq -n -c \
      --argjson expected "$expected_major_universe_size" \
      -f "$(research_loop_state_s3_jq research-loop-universe-empty.jq)"
    return
  fi

  aws_cmd s3 cp "s3://${market_l1_bucket}/${universe_key}" - \
  | jq -c \
    --argjson expected "$expected_major_universe_size" \
    --argjson object "$universe_object" \
    -f "$(research_loop_state_s3_jq research-loop-universe-summary.jq)"
}

build_research_loop_candidate_summary() {
  local candidate_bucket="$1"
  local candidate_read_limit="$2"
  local candidate_p0_json="$3"
  local candidate_p1_json="$4"
  local candidate_p2_json="$5"
  local candidate_objects_json="$6"
  local candidate_records_json="$7"

  aws_cmd s3api list-objects-v2 \
    --bucket "$candidate_bucket" \
    --prefix "candidate-evidence-bundle/priority=p0/" \
    --output json > "$candidate_p0_json"
  aws_cmd s3api list-objects-v2 \
    --bucket "$candidate_bucket" \
    --prefix "candidate-evidence-bundle/priority=p1/" \
    --output json > "$candidate_p1_json"
  aws_cmd s3api list-objects-v2 \
    --bucket "$candidate_bucket" \
    --prefix "candidate-evidence-bundle/priority=p2/" \
    --output json > "$candidate_p2_json"

  jq -s -c \
    --argjson limit "$candidate_read_limit" \
    -f "$(research_loop_state_s3_jq research-loop-latest-objects.jq)" \
    "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" > "$candidate_objects_json"

  : > "$candidate_records_json"
  while IFS= read -r key; do
    [[ -z "$key" ]] && continue
    aws_cmd s3 cp "s3://${candidate_bucket}/${key}" - \
    | jq -c \
      --arg key "$key" \
      -f "$(research_loop_state_s3_jq research-loop-candidate-record.jq)" >> "$candidate_records_json"
  done < <(jq -r '.[].Key' "$candidate_objects_json")

  jq -s -c \
    --argjson read_limit "$candidate_read_limit" \
    --argjson object_count "$(jq 'length' "$candidate_objects_json")" \
    -f "$(research_loop_state_s3_jq research-loop-candidate-summary.jq)" "$candidate_records_json"
}

build_research_loop_latest_report_summary() {
  local output_bucket="$1"
  local report_object="$2"
  local report_key

  report_key="$(jq -r '.key // empty' <<<"$report_object")"
  if [[ -z "$report_key" ]]; then
    jq -n -c -f "$(research_loop_state_s3_jq research-loop-report-empty.jq)"
    return
  fi

  aws_cmd s3 cp "s3://${output_bucket}/${report_key}" - \
  | jq -c \
    --arg key "$report_key" \
    --arg last_modified "$(jq -r '.lastModified' <<<"$report_object")" \
    -f "$(research_loop_state_s3_jq research-loop-report-summary.jq)"
}

collect_research_loop_recent_report_records() {
  local output_bucket="$1"
  local report_read_limit="$2"
  local report_objects_json="$3"
  local report_records_json="$4"

  aws_cmd s3api list-objects-v2 \
    --bucket "$output_bucket" \
    --prefix "research-run-report/" \
    --output json \
  | jq -c \
      --argjson limit "$report_read_limit" \
      -f "$(research_loop_state_s3_jq research-loop-report-objects.jq)" > "$report_objects_json"

  : > "$report_records_json"
  while IFS= read -r object_json; do
    key="$(jq -r '.Key' <<<"$object_json")"
    last_modified="$(jq -r '.LastModified' <<<"$object_json")"
    [[ -z "$key" || "$key" == "null" ]] && continue
    aws_cmd s3 cp "s3://${output_bucket}/${key}" - \
    | jq -c \
      --arg key "$key" \
      --arg last_modified "$last_modified" \
      -f "$(research_loop_state_s3_jq research-loop-report-record.jq)" >> "$report_records_json"
  done < <(jq -c '.[]' "$report_objects_json")
}
