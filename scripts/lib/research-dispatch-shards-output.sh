#!/usr/bin/env bash

write_dispatch_source_manifest_summary() {
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
    --arg copied_manifest_file "$BASE_MANIFEST_OUTPUT" \
    --argjson selected_candidate_count "$(jq '.candidate_bundle_refs | length' "$BASE_MANIFEST_OUTPUT")" \
    '{
      schema_version:"research_dispatch_source_manifest_summary_v1",
      generated_at:$generated_at,
      source_manifest_file:$source_manifest_file,
      copied_manifest_file:$copied_manifest_file,
      selected_candidate_count:$selected_candidate_count
    }' > "$BASE_MANIFEST_SUMMARY_OUTPUT"
}

append_dry_run_task_summary() {
  jq -n -c \
    --arg shard_id "$shard_id" \
    --arg manifest_file "$shard_manifest" \
    --arg s3_key "$shard_key" \
    --argjson candidate_count "$shard_candidate_count" \
    '{
      shard_id:$shard_id,
      manifest_file:$manifest_file,
      manifest_s3_key:$s3_key,
      candidate_count:$candidate_count,
      dry_run:true,
      task_arn:null,
      exit_code:null
    }' >> "$task_summary_jsonl"
}

write_dispatch_shard_driver_summary() {
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg run_id "$RUN_ID" \
    --arg run_dir "$RUN_DIR" \
    --arg base_manifest_file "$BASE_MANIFEST_OUTPUT" \
    --arg base_manifest_summary_file "$BASE_MANIFEST_SUMMARY_OUTPUT" \
    --arg task_summary_file "$task_summary_jsonl" \
    --arg report_summary_file "$report_summary_jsonl" \
    --argjson shard_size "$SHARD_SIZE" \
    --argjson shard_count "$shard_count" \
    --argjson candidate_count "$total_candidate_count" \
    --argjson dry_run "$(bool_is_true "$DRY_RUN" && echo true || echo false)" \
    --slurpfile manifest_summary_input "$BASE_MANIFEST_SUMMARY_OUTPUT" \
    --slurpfile task_summary_input "$task_summary_jsonl" \
    --slurpfile report_summary_input "$report_summary_jsonl" \
    '($manifest_summary_input[0] // {}) as $manifest_summary
    | ($task_summary_input // []) as $tasks
    | ($report_summary_input // []) as $reports
    | {
        schema_version:"research_dispatch_shard_driver_summary_v1",
        generated_at:$generated_at,
        run_id:$run_id,
        run_dir:$run_dir,
        base_manifest_file:$base_manifest_file,
        base_manifest_summary_file:$base_manifest_summary_file,
        task_summary_file:$task_summary_file,
        report_summary_file:$report_summary_file,
        shard_size:$shard_size,
        shard_count:$shard_count,
        candidate_count:$candidate_count,
        dry_run:$dry_run,
        safety:{
          current_approved_required:true,
          dispatcher_mode_changed:false,
          live_enabled:false,
          paper_live_enabled:false,
          order_execution_enabled:false
        },
        manifest:{
          universe_mode:($manifest_summary.universe_mode // null),
          selected_candidate_count:($manifest_summary.selected_candidate_count // $candidate_count),
          current_approved_candidate_count:($manifest_summary.current_approved_candidate_count // null),
          latest_universe:($manifest_summary.latest_universe // null)
        },
        tasks:{
          total:($tasks | length),
          succeeded:($tasks | map(select(.exit_code == 0 or .dry_run == true)) | length),
          failed:($tasks | map(select(.dry_run != true and .exit_code != 0)) | length),
          exit_codes:($tasks | map(.exit_code) | unique)
        },
        reports:{
          total:($reports | length),
          statuses:($reports | map(.research_run_status) | unique),
          total_source_candidates:($reports | map(.source_candidate_count) | add // 0),
          total_replay_runs:($reports | map(.replay_run_count) | add // 0),
          symbols:($reports | map(.partition_symbols[]?) | unique | sort),
          symbol_count:($reports | map(.partition_symbols[]?) | unique | length),
          gate_biases:($reports | map(.gate_biases[]?) | unique | sort),
          shadow_validation_total:($reports | map(.shadow_validation_count) | add // 0),
          paper_trade_candidate_total:($reports | map(.paper_trade_candidate_count) | add // 0)
        }
      }' > "$SUMMARY_OUTPUT"
}

print_dispatch_shard_driver_summary() {
  jq -r '
    "candidate_count=\(.candidate_count)",
    "shard_size=\(.shard_size)",
    "shard_count=\(.shard_count)",
    "task_succeeded=\(.tasks.succeeded)",
    "task_failed=\(.tasks.failed)",
    "report_count=\(.reports.total)",
    "total_source_candidates=\(.reports.total_source_candidates)",
    "total_replay_runs=\(.reports.total_replay_runs)",
    "symbol_count=\(.reports.symbol_count)",
    "gate_biases=\(.reports.gate_biases | join(","))",
    "shadow_validation_total=\(.reports.shadow_validation_total)",
    "paper_trade_candidate_total=\(.reports.paper_trade_candidate_total)"
  ' "$SUMMARY_OUTPUT"
}
