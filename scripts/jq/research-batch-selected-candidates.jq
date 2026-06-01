map(select(
  .batch_horizon_contract_valid == true
  and if $mode == "current_approved" then .current_universe_approved == true
  elif $mode == "current_observed" then .current_universe_observed == true
  elif $mode == "legacy_retest" then .research_eligible == true
  else false
  end
))
| sort_by(.last_modified, .key)
| reverse
| reduce .[] as $candidate ({};
    if has($candidate.candidate_id) then .
    else .[$candidate.candidate_id] = $candidate
    end
  )
| [.[]]
| sort_by(.last_modified, .key)
| reverse
| .[0:$max]
