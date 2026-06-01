{
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
  partition_symbols:([(.partition_aggregates // [])[].symbol_canonical] | unique | sort),
  gate_biases:([(.partition_aggregates // [])[].gate_bias] | unique | sort),
  shadow_validation_count:((.shadow_validation_runs // []) | length),
  paper_trade_candidate_count:((.paper_trade_candidates // []) | length),
  promotion_bias_count:([
    (.summary_findings // [])[]?
    | select((.bias // "") | startswith("PROMOTE_TO_"))
  ] | length)
}
