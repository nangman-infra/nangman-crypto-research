def symbol_names:
  (.normalized_symbols // .symbols // [])
  | if type == "array" then
      map(
        if type == "string" then .
        else (.symbol_canonical // .symbol // .asset // empty)
        end
      )
    elif type == "string" then [.]
    else []
    end;

. as $record
| if type == "array" then .[] else . end
| select(type == "object")
| {
    source_key:$key,
    candidate_id:(.candidate_id // null),
    candidate_class:(.candidate_class // null),
    research_priority:(.research_priority // null),
    symbols:symbol_names,
    allowed_horizons:(.allowed_horizons // []),
    approved_universe_symbol:(.approved_universe_symbol // null),
    symbol_universe_snapshot_id:(.symbol_universe_snapshot_id // null)
  }
