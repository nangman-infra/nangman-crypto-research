#!/usr/bin/env bash

write_single_market_l1_s3_check() {
  symbol="$1"
  window_start_ms="$2"

  key="$(latest_delta_key_for_window "$window_start_ms")"
  direct_delta_key_present=false
  if [[ -n "$key" ]]; then
    direct_delta_key_present=true
  fi

  regime_key="$(latest_regime_context_key_for_window "$window_start_ms")"
  direct_regime_context_key_present=false
  if [[ -n "$regime_key" ]]; then
    direct_regime_context_key_present=true
  fi

  index_pointer_key="$(l1_index_pointer_key_for_window "$window_start_ms")"
  index_pointer_present=false
  manifest_key=""
  manifest_present=false
  manifest_delta_key=""
  manifest_delta_key_present=false
  manifest_regime_context_key=""
  manifest_regime_context_key_present=false
  if s3_object_exists "$index_pointer_key"; then
    index_pointer_present=true
    manifest_key="$(manifest_key_from_l1_index_pointer "$index_pointer_key" | sed -n '1p')"
    if [[ -n "$manifest_key" ]] && s3_object_exists "$manifest_key"; then
      manifest_present=true
      manifest_delta_key="$(feature_delta_key_from_l1_manifest "$manifest_key" | sed -n '1p')"
      if [[ -n "$manifest_delta_key" ]] && s3_object_exists "$manifest_delta_key"; then
        manifest_delta_key_present=true
      fi
      manifest_regime_context_key="$(regime_context_key_from_l1_manifest "$manifest_key" | sed -n '1p')"
      if [[ -n "$manifest_regime_context_key" ]] && s3_object_exists "$manifest_regime_context_key"; then
        manifest_regime_context_key_present=true
      fi
    fi
  fi

  discoverable_delta_key="$key"
  discoverable_delta_key_present="$direct_delta_key_present"
  if [[ "$discoverable_delta_key_present" != "true" && "$manifest_delta_key_present" == "true" ]]; then
    discoverable_delta_key="$manifest_delta_key"
    discoverable_delta_key_present=true
  fi

  discoverable_regime_context_key="$regime_key"
  discoverable_regime_context_key_present="$direct_regime_context_key_present"
  if [[ "$discoverable_regime_context_key_present" != "true" && "$manifest_regime_context_key_present" == "true" ]]; then
    discoverable_regime_context_key="$manifest_regime_context_key"
    discoverable_regime_context_key_present=true
  fi

  symbol_delta_count=null
  if [[ -n "$discoverable_delta_key" && "$check_symbols_normalized" == "true" ]]; then
    symbol_delta_count="$(symbol_delta_count_for_key "$discoverable_delta_key" "$symbol")"
  fi

  emit_market_l1_s3_check_row
}

write_market_l1_s3_checks() {
  local output_file="$1"
  local window_plan_file="$2"
  local checked_count=0
  local symbol
  local window_start_ms

  : > "$output_file"
  while IFS=$'\t' read -r symbol window_start_ms; do
    checked_count=$((checked_count + 1))
    if (( checked_count > MAX_S3_WINDOWS )); then
      break
    fi
    write_single_market_l1_s3_check "$symbol" "$window_start_ms" >> "$output_file"
  done < <(jq -r '.[] | [.symbol, .window_start_ms] | @tsv' "$window_plan_file")
}
