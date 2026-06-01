def members: ((.included_symbols // []) + (.excluded_symbols // []));

def status_reason_counts:
  members
  | map(.status_reason // "unknown")
  | group_by(.)
  | map({reason:.[0], count:length})
  | sort_by(.count)
  | reverse;

{
  present:true,
  key:$object.key,
  last_modified:$object.lastModified,
  selection:$object.selection,
  run_start_ms:$object.run_start_ms,
  run_end_ms:$object.run_end_ms,
  run_generated_ms:$object.run_generated_ms,
  schema_version,
  symbol_universe_snapshot_id,
  universe_as_of_ms,
  observed_symbols:[(.liquidity_rank_at_that_time // members)[]?.symbol_canonical],
  approved_symbols:[(.included_symbols // [])[]?.symbol_canonical],
  excluded_symbols:[(.excluded_symbols // [])[]?.symbol_canonical],
  observed_symbol_count:((.liquidity_rank_at_that_time // members) | length),
  approved_symbol_count:((.included_symbols // []) | length),
  excluded_symbol_count:((.excluded_symbols // []) | length),
  status_reason_counts:status_reason_counts
}
