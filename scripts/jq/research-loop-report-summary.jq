def bias_counts:
  (.summary_findings // [])
  | group_by(.bias)
  | map({bias:.[0].bias, count:length})
  | sort_by(.bias);

{
  present:true,
  key:$key,
  last_modified:$last_modified,
  schema_version,
  research_packet_id,
  run_scope,
  research_run_status,
  source_candidate_count:((.source_candidate_ids // []) | length),
  replay_run_count:((.replay_run_ids // []) | length),
  partition_count:(.partition_count // ((.partition_aggregates // []) | length)),
  top_symbols:(.top_symbols // []),
  surviving_candidate_count:((.surviving_candidate_keys // []) | length),
  retest_candidate_count:((.retest_candidate_keys // []) | length),
  pruned_candidate_count:((.pruned_candidate_keys // []) | length),
  shadow_validation_count:((.shadow_validation_runs // []) | length),
  paper_trade_candidate_count:((.paper_trade_candidates // []) | length),
  bias_counts:bias_counts,
  gate_biases:([(.partition_aggregates // [])[].gate_bias] | unique | sort),
  promotion_bias_count:([
    (.summary_findings // [])[]?
    | select((.bias // "") | startswith("PROMOTE_TO_"))
  ] | length)
}
