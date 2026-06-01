# shellcheck shell=bash

collect_report_summary() {
  local started_at="$1"
  local summary_jsonl="$2"
  : > "$summary_jsonl"
  aws_cmd s3api list-objects-v2 \
    --bucket "$RESEARCH_BUCKET" \
    --prefix "research-run-report/" \
    --output json \
  | jq -r --arg started_at "$started_at" '
      (.Contents // [])
      | map(select(.LastModified >= $started_at))
      | sort_by(.LastModified, .Key)
      | .[].Key
    ' \
  | while IFS= read -r key; do
      aws_cmd s3 cp "s3://${RESEARCH_BUCKET}/${key}" - --only-show-errors \
      | jq -c --arg key "$key" '
          {
            key:$key,
            research_packet_id,
            run_scope,
            research_run_status,
            source_candidate_count:(.source_candidate_ids | length),
            replay_run_count:(.replay_run_ids | length),
            partition_count,
            top_symbols,
            partition_symbols:([.partition_aggregates[].symbol_canonical] | unique),
            gate_biases:([.partition_aggregates[].gate_bias] | unique),
            retest_candidate_count:(.retest_candidate_keys | length),
            surviving_candidate_count:(.surviving_candidate_keys | length),
            shadow_validation_count:(.shadow_validation_runs | length),
            paper_trade_candidate_count:(.paper_trade_candidates | length)
          }
        ' >> "$summary_jsonl"
    done
}

collect_post_dispatch_reports() {
  report_summary_jsonl="${RUN_DIR}/research-report-summaries.jsonl"
  if bool_is_true "$DRY_RUN"; then
    : > "$report_summary_jsonl"
  else
    collect_report_summary "$dispatch_started_at" "$report_summary_jsonl"
  fi
}
