def research_factory_major50_state($coverage; $driver; $latest_universe):
  {
    universe_mode:($driver.manifest.universe_mode // null),
    latest_universe_present:($latest_universe.present // null),
    observed_symbol_count:($latest_universe.observed_symbol_count // ($coverage.observed_symbols | length)),
    approved_symbol_count:($latest_universe.approved_symbol_count // ($coverage.approved_symbols | length)),
    excluded_symbol_count:($latest_universe.excluded_symbol_count // null),
    candidate_symbol_count:($coverage.candidate_symbols | length),
    candidate_symbols:$coverage.candidate_symbols,
    eligible_candidate_pool_count:($driver.manifest.eligible_candidate_pool_count // null),
    selected_candidate_limit_reached:($driver.manifest.selected_candidate_limit_reached // null),
    unselected_eligible_candidate_count:($driver.manifest.unselected_eligible_candidate_count // null),
    eligible_candidate_symbols:$coverage.eligible_candidate_symbols,
    unselected_eligible_candidate_symbols:$coverage.unselected_eligible_candidate_symbols,
    candidate_symbols_in_approved_universe:$coverage.candidate_symbols_in_approved_universe,
    eligible_candidate_symbols_in_approved_universe:$coverage.eligible_candidate_symbols_in_approved_universe,
    approved_symbols_without_selected_candidate:$coverage.approved_symbols_without_selected_candidate,
    approved_symbols_without_eligible_candidate:$coverage.approved_symbols_without_eligible_candidate,
    selected_symbols_not_in_approved_universe:(
      if ($coverage.approved_symbols | length) == 0 then []
      else ($coverage.candidate_symbols - $coverage.approved_symbols)
      end
    ),
    candidate_symbol_coverage_of_approved_universe:(
      if ($coverage.approved_symbols | length) == 0 then null
      else (($coverage.candidate_symbols_in_approved_universe | length) / ($coverage.approved_symbols | length))
      end
    ),
    eligible_candidate_symbol_coverage_of_approved_universe:(
      if ($coverage.approved_symbols | length) == 0 then null
      else (($coverage.eligible_candidate_symbols_in_approved_universe | length) / ($coverage.approved_symbols | length))
      end
    )
  };

def research_factory_progression($coverage; $latest_universe; $stage):
  {
    major50_observed_symbol_count:($latest_universe.observed_symbol_count // ($coverage.observed_symbols | length)),
    major50_approved_symbol_count:($latest_universe.approved_symbol_count // ($coverage.approved_symbols | length)),
    candidate_generated_symbol_count:($coverage.candidate_symbols | length),
    candidate_generated_candidate_count:($coverage.candidate_ids | length),
    research_replayed_symbol_count:($coverage.research_replayed_symbols | length),
    research_replayed_candidate_count:($coverage.research_replayed_candidate_ids | length),
    promotion_ready_symbol_count:($coverage.promotion_ready_symbols | length),
    promotion_ready_candidate_count:($coverage.promotion_ready_candidate_ids | length),
    promoted_symbol_count:($coverage.promoted_symbols | length),
    promoted_candidate_count:($coverage.promoted_candidate_ids | length),
    shadow_created:(($stage.shadow_created // false) == true),
    paper_created:(($stage.paper_created // false) == true),
    live_enabled:false,
    symbols:{
      candidate_generated:$coverage.candidate_symbols,
      research_replayed:$coverage.research_replayed_symbols,
      promotion_ready:$coverage.promotion_ready_symbols,
      promoted:$coverage.promoted_symbols
    },
    candidates:{
      candidate_generated:$coverage.candidate_ids,
      research_replayed:$coverage.research_replayed_candidate_ids,
      promotion_ready:$coverage.promotion_ready_candidate_ids,
      promoted:$coverage.promoted_candidate_ids
    }
  };
