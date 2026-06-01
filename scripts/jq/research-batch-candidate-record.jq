. as $record
| if type == "array" then .[] else . end
| select(type == "object")
| {
    bucket:$bucket,
    key:$key,
    uri:$uri,
    last_modified:$last_modified,
    size:$size,
    candidate_id:(.candidate_id // null),
    candidate_lifecycle_key:(.candidate_lifecycle_key // null),
    candidate_class:(.candidate_class // null),
    research_priority:(.research_priority // null),
    research_eligible:(.research_eligible // false),
    symbols:(
      (.normalized_symbols // .symbols // [])
      | if type == "array" then
          map(if type == "string" then . else (.symbol_canonical // .symbol // .asset // empty) end)
        elif type == "string" then [.]
        else []
        end
    ),
    allowed_horizons:(.allowed_horizons // []),
    approved_universe_symbol:(.approved_universe_symbol // false),
    forbidden_lookahead_boundary_ms:(.forbidden_lookahead_boundary_ms // null),
    universe_as_of_ms:(.universe_as_of_ms // null),
    symbol_universe_snapshot_id:(.symbol_universe_snapshot_id // null)
  }
