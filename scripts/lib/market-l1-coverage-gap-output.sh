#!/usr/bin/env bash

emit_market_l1_s3_check_row() {
  jq -nc \
    --arg symbol "$symbol" \
    --argjson window_start_ms "$window_start_ms" \
    --arg key "$key" \
    --arg regime_key "$regime_key" \
    --arg index_pointer_key "$index_pointer_key" \
    --arg manifest_key "$manifest_key" \
    --arg manifest_delta_key "$manifest_delta_key" \
    --arg manifest_regime_context_key "$manifest_regime_context_key" \
    --arg discoverable_delta_key "$discoverable_delta_key" \
    --arg discoverable_regime_context_key "$discoverable_regime_context_key" \
    --argjson direct_delta_key_present "$direct_delta_key_present" \
    --argjson direct_regime_context_key_present "$direct_regime_context_key_present" \
    --argjson index_pointer_present "$index_pointer_present" \
    --argjson manifest_present "$manifest_present" \
    --argjson manifest_delta_key_present "$manifest_delta_key_present" \
    --argjson manifest_regime_context_key_present "$manifest_regime_context_key_present" \
    --argjson discoverable_delta_key_present "$discoverable_delta_key_present" \
    --argjson discoverable_regime_context_key_present "$discoverable_regime_context_key_present" \
    --argjson symbol_delta_count "$symbol_delta_count" \
    '{
      symbol:$symbol,
      window_start_ms:$window_start_ms,
      delta_key:(if $key == "" then null else $key end),
      delta_key_present:$direct_delta_key_present,
      regime_context_key:(if $regime_key == "" then null else $regime_key end),
      regime_context_key_present:$direct_regime_context_key_present,
      index_pointer_key:$index_pointer_key,
      index_pointer_present:$index_pointer_present,
      manifest_key:(if $manifest_key == "" then null else $manifest_key end),
      manifest_present:$manifest_present,
      manifest_delta_key:(if $manifest_delta_key == "" then null else $manifest_delta_key end),
      manifest_delta_key_present:$manifest_delta_key_present,
      manifest_regime_context_key:(if $manifest_regime_context_key == "" then null else $manifest_regime_context_key end),
      manifest_regime_context_key_present:$manifest_regime_context_key_present,
      discoverable_delta_key:(if $discoverable_delta_key == "" then null else $discoverable_delta_key end),
      discoverable_delta_key_present:$discoverable_delta_key_present,
      discoverable_regime_context_key:(if $discoverable_regime_context_key == "" then null else $discoverable_regime_context_key end),
      discoverable_regime_context_key_present:$discoverable_regime_context_key_present,
      symbol_delta_count:$symbol_delta_count
    }'
}

emit_market_l1_coverage_gap_diagnosis() {
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg plan_file "$PLAN_FILE" \
    --arg report_file "$REPORT_FILE" \
    --arg replay_run_file "$REPLAY_RUN_FILE" \
    --arg market_l1_bucket "$MARKET_L1_BUCKET" \
    --argjson check_s3 "$([[ "$check_s3_normalized" == "true" ]] && echo true || echo false)" \
    --argjson check_symbols "$([[ "$check_symbols_normalized" == "true" ]] && echo true || echo false)" \
    --argjson max_s3_windows "$MAX_S3_WINDOWS" \
    --slurpfile plan_gaps "$plan_gaps_json" \
    --slurpfile aggregate_gaps "$aggregate_gaps_json" \
    --slurpfile current_missing "$current_missing_json" \
    --slurpfile s3_window_plan "$s3_window_plan_json" \
    --slurpfile s3_checks "$s3_checks_jsonl" \
    '
      ($s3_checks // []) as $checks
      | {
          schema_version:"research_market_l1_coverage_gap_diagnosis_v1",
          generated_at:$generated_at,
          inputs:{
            plan_file:$plan_file,
            report_file:(if $report_file == "" then null else $report_file end),
            replay_run_file:(if $replay_run_file == "" then null else $replay_run_file end),
            market_l1_bucket_set:($market_l1_bucket != ""),
            check_s3:$check_s3,
            check_symbols:$check_symbols,
            max_s3_windows:$max_s3_windows
          },
          plan_gaps:$plan_gaps[0],
          aggregate_gaps:$aggregate_gaps[0],
          current_missing_replay_windows:$current_missing[0],
          s3_window_plan:{
            required_symbol_window_count:($s3_window_plan[0] | length),
            checked_window_count:($checks | length),
            truncated:($check_s3 and (($s3_window_plan[0] | length) > ($checks | length))),
            rows:$s3_window_plan[0]
          },
          s3_checks:{
            checked_window_count:($checks | length),
            missing_delta_key_count:($checks | map(select(.delta_key_present == false)) | length),
            present_delta_key_count:($checks | map(select(.delta_key_present == true)) | length),
            missing_regime_context_key_count:($checks | map(select(.regime_context_key_present == false)) | length),
            present_regime_context_key_count:($checks | map(select(.regime_context_key_present == true)) | length),
            present_index_pointer_count:($checks | map(select(.index_pointer_present == true)) | length),
            present_manifest_count:($checks | map(select(.manifest_present == true)) | length),
            present_manifest_delta_key_count:($checks | map(select(.manifest_delta_key_present == true)) | length),
            present_manifest_regime_context_key_count:($checks | map(select(.manifest_regime_context_key_present == true)) | length),
            missing_discoverable_delta_key_count:(
              $checks
              | map(select(.discoverable_delta_key_present == false))
              | length
            ),
            present_discoverable_delta_key_count:(
              $checks
              | map(select(.discoverable_delta_key_present == true))
              | length
            ),
            missing_discoverable_regime_context_key_count:(
              $checks
              | map(select(.discoverable_regime_context_key_present == false))
              | length
            ),
            present_discoverable_regime_context_key_count:(
              $checks
              | map(select(.discoverable_regime_context_key_present == true))
              | length
            ),
            zero_symbol_delta_count:(
              $checks
              | map(select(.symbol_delta_count != null and .symbol_delta_count == 0))
              | length
            ),
            positive_symbol_delta_count:(
              $checks
              | map(select(.symbol_delta_count != null and .symbol_delta_count > 0))
              | length
            ),
            missing_delta_key_rows:($checks | map(select(.delta_key_present == false))),
            missing_regime_context_key_rows:($checks | map(select(.regime_context_key_present == false))),
            missing_discoverable_delta_key_rows:($checks | map(select(.discoverable_delta_key_present == false))),
            missing_discoverable_regime_context_key_rows:($checks | map(select(.discoverable_regime_context_key_present == false))),
            zero_symbol_delta_rows:($checks | map(select(.symbol_delta_count != null and .symbol_delta_count == 0))),
            sample_rows:($checks[0:20])
          }
        }
    '
}
