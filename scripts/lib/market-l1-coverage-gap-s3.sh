#!/usr/bin/env bash

latest_direct_key_for_window() {
  local window_start_ms="$1"
  local family_prefix="$2"
  local file_suffix="$3"
  aws_cmd s3api list-objects-v2 \
    --bucket "$MARKET_L1_BUCKET" \
    --prefix "${family_prefix}/run_id=l1_${window_start_ms}_" \
    --output json \
  | jq -r --arg file_suffix "$file_suffix" '
      (.Contents // [])
      | map(select(.Key | endswith($file_suffix)))
      | sort_by(.Key)
      | last
      | .Key // empty
    '
}

latest_delta_key_for_window() {
  local window_start_ms="$1"
  latest_direct_key_for_window "$window_start_ms" "market_feature_delta" "/delta.json"
}

latest_regime_context_key_for_window() {
  local window_start_ms="$1"
  latest_direct_key_for_window "$window_start_ms" "market_regime_context" "/context.json"
}

s3_object_exists() {
  local key="$1"
  aws_cmd s3api head-object --bucket "$MARKET_L1_BUCKET" --key "$key" >/dev/null 2>&1
}

normalize_s3_key() {
  local value="$1"
  value="${value#/}"
  if [[ "$value" == s3://* ]]; then
    value="${value#s3://}"
    value="${value#*/}"
  fi
  printf '%s' "$value"
}

l1_index_pointer_key_for_window() {
  local window_start_ms="$1"
  local event_date
  local hour
  IFS=$'\t' read -r event_date hour < <(
    jq -nr --argjson window_start_ms "$window_start_ms" '
      (($window_start_ms / 1000) | floor | gmtime)
      | [strftime("%Y-%m-%d"), strftime("%H")]
      | @tsv
    '
  )
  printf 'l1_index/window_ms=1000/event_date=%s/hour=%s/window_start_ms=%s.json' \
    "$event_date" "$hour" "$window_start_ms"
}

manifest_key_from_l1_index_pointer() {
  local pointer_key="$1"
  aws_cmd s3 cp "s3://${MARKET_L1_BUCKET}/${pointer_key}" - \
  | jq -r '
      select(.schema_version == "l1_index_pointer_v1")
      | select((.status // "" | ascii_downcase) == "success")
      | (.canonical_manifest_key // .manifest_key // empty)
    ' \
  | while IFS= read -r key; do
      normalize_s3_key "$key"
    done
}

artifact_key_from_l1_manifest() {
  local manifest_key="$1"
  local manifest_field="$2"
  aws_cmd s3 cp "s3://${MARKET_L1_BUCKET}/${manifest_key}" - \
  | jq -r --arg manifest_field "$manifest_field" '
      select(.schema_version == "l1_manifest_v1")
      | select((.status // "" | ascii_downcase) == "success")
      | (.[$manifest_field] // empty)
    ' \
  | while IFS= read -r key; do
      normalize_s3_key "$key"
    done
}

feature_delta_key_from_l1_manifest() {
  local manifest_key="$1"
  artifact_key_from_l1_manifest "$manifest_key" "market_feature_delta_key"
}

regime_context_key_from_l1_manifest() {
  local manifest_key="$1"
  artifact_key_from_l1_manifest "$manifest_key" "market_regime_context_key"
}

symbol_delta_count_for_key() {
  local key="$1"
  local symbol="$2"
  aws_cmd s3 cp "s3://${MARKET_L1_BUCKET}/${key}" - \
  | jq -s --arg symbol "$symbol" '
      def rows:
        if length == 1 and (.[0] | type) == "array" then .[0] else . end;
      [rows[]? | select(.symbol_canonical == $symbol)] | length
    '
}
