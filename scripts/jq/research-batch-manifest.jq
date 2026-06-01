($candidates_input[0] // []) as $candidates
| ($indexes_input[0] // []) as $indexes
| {
  schema_version:"research_input_manifest_v1",
  research_packet_id:$research_packet_id,
  run_scope:$run_scope,
  candidate_bundle_refs:($candidates | map({uri:.uri})),
  historical_replay_run_index_refs:($indexes | map({uri:.uri})),
  runtime_budget_policy:{
    max_candidate_bundle_count:$max_candidates,
    max_market_artifact_ref_count:2000,
    max_shadow_validation_run_ref_count:10000,
    max_hypothesis_harness_result_ref_count:10000,
    max_oss_adapter_run_ref_count:10000,
    max_historical_replay_run_ref_count:$max_history,
    max_replay_run_count:$max_replay
  }
}
