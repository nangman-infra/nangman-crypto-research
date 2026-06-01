#!/usr/bin/env bash

post_activation_runtime_jq() {
  local name="$1"
  local path="$JQ_DIR/$name"
  if [[ ! -f "$path" ]]; then
    echo "missing post-activation runtime jq program: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

require_fresh_object() {
  local bucket="$1"
  local prefix="$2"
  local object_json="$3"
  local key
  local last_modified
  local size

  key="$(jq -r '.key // empty' <<<"$object_json")"
  last_modified="$(jq -r '.lastModified // empty' <<<"$object_json")"
  size="$(jq -r '.size // 0' <<<"$object_json")"
  if [[ -z "$key" || -z "$last_modified" || "$size" == "0" ]]; then
    echo "missing or empty required output prefix: s3://${bucket}/${prefix}" | redact >&2
    exit 1
  fi
  if [[ -n "$OUTPUT_MIN_LAST_MODIFIED" && "$last_modified" < "$OUTPUT_MIN_LAST_MODIFIED" ]]; then
    echo "stale output prefix: s3://${bucket}/${prefix} latest=${last_modified} min=${OUTPUT_MIN_LAST_MODIFIED}" | redact >&2
    exit 1
  fi
}

validate_report_sample() {
  local bucket="$1"
  local key="$2"
  local report_json
  report_json="$(aws_cmd s3 cp "s3://${bucket}/${key}" -)"

  jq -e -f "$(post_activation_runtime_jq post-activation-report-sample-validation.jq)" <<<"$report_json" >/dev/null
  jq -c -f "$(post_activation_runtime_jq post-activation-report-sample-summary.jq)" <<<"$report_json" | redact
}
