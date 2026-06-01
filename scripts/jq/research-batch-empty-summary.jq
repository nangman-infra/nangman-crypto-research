{
  generated_at:$generated_at,
  manifest_output:$manifest_output,
  summary_output:$summary_output,
  region:$region,
  dispatch_mode:$dispatch_mode,
  universe_mode:$universe_mode,
  safety:{
    s3_write:false,
    ecs_task_started:false,
    dispatcher_mode_changed:false,
    local_manifest_only:true,
    selected_candidates_require_current_universe:($universe_mode != "legacy_retest")
  },
  latest_universe:$universe,
  candidate_read_limit:$candidate_read_limit,
  max_candidate_bundle_count:$max_candidate_bundle_count,
  selected_candidate_count:0,
  scanned_research_eligible_candidate_count:($candidates | length),
  current_observed_candidate_count:([$candidates[] | select(.current_universe_observed == true)] | length),
  current_approved_candidate_count:([$candidates[] | select(.current_universe_approved == true)] | length),
  legacy_bundle_approved_candidate_count:([$candidates[] | select(.approved_universe_symbol == true)] | length),
  horizon_contract_valid_candidate_count:([$candidates[] | select(.batch_horizon_contract_valid == true)] | length),
  horizon_contract_invalid_candidate_count:([$candidates[] | select(.batch_horizon_contract_valid != true)] | length),
  excluded_horizon_contract_violations:(
    [$candidates[] | select(.batch_horizon_contract_valid != true)]
    | map({
        candidate_id,
        symbols,
        allowed_horizons,
        reasons:.batch_horizon_contract_reasons,
        last_modified,
        key
      })
    | .[0:20]
  ),
  blocked_reason:"no_candidates_match_universe_mode_or_horizon_contract"
}
