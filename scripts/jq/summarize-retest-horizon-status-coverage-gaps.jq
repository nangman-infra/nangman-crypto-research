def research_factory_coverage_gaps($coverage; $stage):
  {
    approved_symbols_without_candidate:$coverage.approved_symbols_without_eligible_candidate,
    approved_symbols_without_selected_candidate:$coverage.approved_symbols_without_selected_candidate,
    approved_symbols_without_eligible_candidate:$coverage.approved_symbols_without_eligible_candidate,
    unselected_eligible_candidate_symbols:$coverage.unselected_eligible_candidate_symbols,
    candidate_symbols_without_replay:$coverage.candidate_symbols_without_replay,
    candidate_ids_without_replay:$coverage.candidate_ids_without_replay,
    replayed_symbols_without_promotion_ready:$coverage.replayed_symbols_without_promotion_ready,
    replayed_symbols_without_promotion:$coverage.replayed_symbols_without_promotion,
    replayed_candidate_ids_without_promotion_ready:$coverage.replayed_candidate_ids_without_promotion_ready,
    replayed_candidate_ids_without_promotion:$coverage.replayed_candidate_ids_without_promotion,
    promotion_ready_symbols_without_shadow:(
      if (($stage.shadow_created // false) == true) then []
      else $coverage.promotion_ready_symbols
      end
    ),
    promotion_ready_candidate_ids_without_shadow:(
      if (($stage.shadow_created // false) == true) then []
      else $coverage.promotion_ready_candidate_ids
      end
    ),
    promoted_symbols_without_shadow:(
      if (($stage.shadow_created // false) == true) then []
      else $coverage.promoted_symbols
      end
    ),
    promoted_candidate_ids_without_shadow:(
      if (($stage.shadow_created // false) == true) then []
      else $coverage.promoted_candidate_ids
      end
    )
  };

def research_factory_safe_next_actions($coverage; $driver; $next_decision):
  [
    if ($coverage.approved_symbols_without_eligible_candidate | length) > 0
      then "increase_candidate_generation_for_approved_major50_symbols"
      else empty end,
    if (($driver.manifest.selected_candidate_limit_reached // false) == true and ($coverage.unselected_eligible_candidate_symbols | length) > 0)
      then "increase_research_batch_selection_limit_or_run_focused_manifest"
      else empty end,
    if ($coverage.candidate_ids_without_replay | length) > 0
      then "build_focused_research_manifest_for_unreplayed_candidate_symbols"
      else empty end,
    if ($next_decision.safe_next_actions // [] | index("extend_market_l1_horizon_coverage") != null)
      then "extend_market_l1_horizon_coverage_for_current_retest_symbols"
      else empty end,
    if ($next_decision.safe_next_actions // [] | index("wait_for_market_l1_horizon_materialization") != null)
      then "wait_for_market_l1_horizon_materialization"
      else empty end,
    if ($next_decision.safe_next_actions // [] | index("keep_accumulating_completed_native_replay_samples") != null)
      then "keep_accumulating_completed_native_replay_samples"
      else empty end
  ]
  | unique;

def research_factory_gap_summary($coverage; $driver; $latest_universe; $next_decision):
  {
    blocking_stage:$coverage.blocking_stage,
    stage_counts:{
      major50_observed:($latest_universe.observed_symbol_count // ($coverage.observed_symbols | length)),
      major50_approved:($latest_universe.approved_symbol_count // ($coverage.approved_symbols | length)),
      candidate_generated:($coverage.candidate_symbols | length),
      candidate_generated_candidates:($coverage.candidate_ids | length),
      research_replayed:($coverage.research_replayed_symbols | length),
      research_replayed_candidates:($coverage.research_replayed_candidate_ids | length),
      promotion_ready:($coverage.promotion_ready_symbols | length),
      promotion_ready_candidates:($coverage.promotion_ready_candidate_ids | length),
      promoted:($coverage.promoted_symbols | length),
      promoted_candidates:($coverage.promoted_candidate_ids | length)
    },
    gap_counts:{
      approved_symbols_without_candidate:($coverage.approved_symbols_without_eligible_candidate | length),
      approved_symbols_without_selected_candidate:($coverage.approved_symbols_without_selected_candidate | length),
      approved_symbols_without_eligible_candidate:($coverage.approved_symbols_without_eligible_candidate | length),
      unselected_eligible_candidate_symbols:($coverage.unselected_eligible_candidate_symbols | length),
      candidate_symbols_without_replay:($coverage.candidate_symbols_without_replay | length),
      candidate_ids_without_replay:($coverage.candidate_ids_without_replay | length),
      replayed_symbols_without_promotion_ready:($coverage.replayed_symbols_without_promotion_ready | length),
      replayed_symbols_without_promotion:($coverage.replayed_symbols_without_promotion | length),
      replayed_candidate_ids_without_promotion_ready:($coverage.replayed_candidate_ids_without_promotion_ready | length),
      replayed_candidate_ids_without_promotion:($coverage.replayed_candidate_ids_without_promotion | length)
    },
    safe_next_actions:research_factory_safe_next_actions($coverage; $driver; $next_decision)
  };
