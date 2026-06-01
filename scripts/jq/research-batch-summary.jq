($universe_input[0] // {}) as $universe
| ($scanned_candidates_input[0] // []) as $scanned_candidates
| ($candidates_input[0] // []) as $candidates
| ($indexes_input[0] // []) as $indexes
|
def eligible_for_universe_mode:
  .batch_horizon_contract_valid == true
  and if $universe_mode == "current_approved" then .current_universe_approved == true
  elif $universe_mode == "current_observed" then .current_universe_observed == true
  elif $universe_mode == "legacy_retest" then .research_eligible == true
  else false
  end;

def latest_unique_by_candidate_id:
  sort_by(.last_modified, .key)
  | reverse
  | reduce .[] as $candidate ({};
      if has($candidate.candidate_id) then .
      else .[$candidate.candidate_id] = $candidate
      end
    )
  | [.[]]
  | sort_by(.last_modified, .key)
  | reverse;

($scanned_candidates | map(select(eligible_for_universe_mode)) | latest_unique_by_candidate_id) as $eligible_candidates
| ([$candidates[].candidate_id] | unique) as $selected_candidate_ids
| ($eligible_candidates | map(select((.candidate_id as $id | $selected_candidate_ids | index($id)) | not))) as $unselected_candidates
| {
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
  research_packet_id:$research_packet_id,
  run_scope:$run_scope,
  scanned_research_eligible_candidate_count:($scanned_candidates | length),
  selected_candidate_count:($candidates | length),
  eligible_candidate_pool_count:($eligible_candidates | length),
  selected_candidate_limit_reached:(
    ($candidates | length) >= $max_candidate_bundle_count
    and ($eligible_candidates | length) > ($candidates | length)
  ),
  unselected_eligible_candidate_count:($unselected_candidates | length),
  unselected_eligible_candidate_symbols:([$unselected_candidates[].symbols[]?] | unique | sort),
  distinct_candidate_symbols:([$candidates[].symbols[]?] | unique | sort),
  eligible_candidate_symbols:([$eligible_candidates[].symbols[]?] | unique | sort),
  candidate_class_counts:(
    [$candidates[].candidate_class]
    | group_by(.)
    | map({candidate_class:.[0], count:length})
  ),
  research_priority_counts:(
    [$candidates[].research_priority]
    | group_by(.)
    | map({research_priority:.[0], count:length})
  ),
  allowed_horizons:([$candidates[].allowed_horizons[]?] | unique | sort),
  approved_bundle_candidate_count:([$candidates[] | select(.approved_universe_symbol == true)] | length),
  current_observed_candidate_count:([$scanned_candidates[] | select(.current_universe_observed == true)] | length),
  current_approved_candidate_count:([$scanned_candidates[] | select(.current_universe_approved == true)] | length),
  horizon_contract_valid_candidate_count:([$scanned_candidates[] | select(.batch_horizon_contract_valid == true)] | length),
  horizon_contract_invalid_candidate_count:([$scanned_candidates[] | select(.batch_horizon_contract_valid != true)] | length),
  excluded_horizon_contract_violations:(
    [$scanned_candidates[] | select(.batch_horizon_contract_valid != true)]
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
  selected_current_observed_candidate_count:([$candidates[] | select(.current_universe_observed == true)] | length),
  selected_current_approved_candidate_count:([$candidates[] | select(.current_universe_approved == true)] | length),
  selected_horizon_contract_valid_count:([$candidates[] | select(.batch_horizon_contract_valid == true)] | length),
  selected_bundle_current_universe_match_count:([$candidates[] | select(.bundle_current_universe_match == true)] | length),
  historical_replay_run_index_ref_count:($indexes | length),
  selected_candidates:($candidates | map({
    candidate_id,
    candidate_class,
    research_priority,
    symbols,
    allowed_horizons,
    approved_universe_symbol,
    current_universe_observed,
    current_universe_approved,
    bundle_current_universe_match,
    batch_horizon_contract_valid,
    last_modified,
    key
  }))
}
