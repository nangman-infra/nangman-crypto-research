def all_symbols_in($allowed):
  (.symbols | length) > 0
  and all(.symbols[]; . as $symbol | ($allowed | index($symbol)));

def horizon_ms($h):
  if $h == "15m" then 900000
  elif $h == "1h" then 3600000
  elif $h == "4h" then 14400000
  elif $h == "24h" then 86400000
  elif $h == "72h" then 259200000
  elif $h == "7d" then 604800000
  else null
  end;

def absolute_max_horizon_ms: 259200000;

def horizon_contract_valid:
  (.allowed_horizons // []) as $horizons
  | ($horizons | length) > 0
    and all($horizons[]; (horizon_ms(.) != null and horizon_ms(.) <= absolute_max_horizon_ms));

def horizon_contract_reasons:
  (.allowed_horizons // []) as $horizons
  | (
      if ($horizons | length) == 0
      then ["missing_allowed_horizons"]
      else []
      end
    )
    + [
      $horizons[]
      | select(horizon_ms(.) == null)
      | "unsupported_horizon:" + .
    ]
    + [
      $horizons[]
      | select((horizon_ms(.) // 0) > absolute_max_horizon_ms)
      | "holding_horizon_contract_violation:" + .
    ];

map(select(.candidate_id != null and .research_eligible == true))
| map(. + {
    current_universe_snapshot_id:($universe.symbol_universe_snapshot_id // null),
    current_universe_as_of_ms:($universe.universe_as_of_ms // null),
    current_universe_observed:all_symbols_in($universe.observed_symbols // []),
    current_universe_approved:all_symbols_in($universe.approved_symbols // []),
    bundle_current_universe_match:(.symbol_universe_snapshot_id == ($universe.symbol_universe_snapshot_id // null)),
    universe_selection_mode:$mode,
    batch_horizon_contract_valid:horizon_contract_valid,
    batch_horizon_contract_reasons:horizon_contract_reasons
  })
