def members: ((.included_symbols // []) + (.excluded_symbols // []));

def top_reasons:
  [(.excluded_symbols // [])[]?.status_reason]
  | group_by(.)
  | map({reason:.[0], count:length})
  | sort_by(.count)
  | reverse
  | .[0:5];

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
  expected_major_universe_size:$expected,
  observed_symbol_count:((.liquidity_rank_at_that_time // members) | length),
  approved_symbol_count:((.included_symbols // []) | length),
  excluded_symbol_count:((.excluded_symbols // []) | length),
  major_coverage_complete:(((.liquidity_rank_at_that_time // members) | length) >= $expected),
  approved_major_coverage_complete:(((.included_symbols // []) | length) >= $expected),
  top_observed_symbols:[(.liquidity_rank_at_that_time // [])[]?.symbol_canonical][0:$expected],
  approved_symbols:[(.included_symbols // [])[]?.symbol_canonical][0:$expected],
  top_exclusion_reasons:top_reasons
}
