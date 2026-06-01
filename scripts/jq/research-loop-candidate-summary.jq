{
  candidate_read_limit:$read_limit,
  recent_bundle_object_count:$object_count,
  recent_candidate_record_count:length,
  distinct_candidate_symbols:([.[].symbols[]?] | unique | sort),
  distinct_candidate_symbol_count:([.[].symbols[]?] | unique | length),
  latest_candidates:[.[0:10][] | {
    candidate_id,
    candidate_class,
    research_priority,
    symbols,
    allowed_horizons,
    approved_universe_symbol
  }]
}
