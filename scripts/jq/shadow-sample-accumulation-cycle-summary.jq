{
  schema_version:"research_shadow_sample_accumulation_cycle_summary_v1",
  generated_at:$generated_at,
  run_dir:$run_dir,
  source_manifest_file:$source_manifest_file,
  retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
  shadow_input_files:$shadow_input_files,
  merged_shadow_file:$merged_shadow_file,
  observation_plan_file:$observation_plan_file,
  gap_manifest_file:$gap_manifest_file,
  accumulation_manifest_file:(if $accumulation_created then $accumulation_manifest_file else null end),
  accumulation_summary_file:(if $accumulation_created then $accumulation_summary_file else null end),
  accumulation_blocked_reason:(if $accumulation_blocked_reason == "" then null else $accumulation_blocked_reason end),
  cycle_summary_file:$cycle_summary_file,
  decision_file:$decision_file,
  latest_l1_as_of_ms:(if $latest_l1_as_of_ms == "" then null else ($latest_l1_as_of_ms | tonumber) end),
  safety:{
    s3_write:false,
    ecs_task_started:false,
    dispatcher_mode_changed:false,
    local_cycle_only:true,
    shadow_status_mutated:false,
    paper_live_enabled:false
  },
  merge_summary:($merge_summary[0] // null),
  observation_summary:($observation[0].observation_summary // null),
  gap_summary:($gap[0].shadow_sample_gap_summary // null),
  accumulation_summary:(if $accumulation_created then ($accumulation_summary.backlog_summary // null) else null end),
  next_decision:{
    verdict:($gap[0].next_decision.verdict // null),
    safe_next_actions:($gap[0].next_decision.safe_next_actions // []),
    next_observation_not_before_ms:($gap[0].next_decision.next_observation_not_before_ms // null),
    next_observation_not_before_at:($gap[0].next_decision.next_observation_not_before_at // null),
    next_observation_not_before_source:($gap[0].next_decision.next_observation_not_before_source // null),
    blocked_actions:(
      if $accumulation_created then
        (($gap[0].next_decision.blocked_actions // [])
        + ($accumulation_summary.next_decision.blocked_actions // []))
        | unique
        | sort
      else
        ($gap[0].next_decision.blocked_actions // [])
      end
    )
  }
}
