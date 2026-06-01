($manifest_summary_file_input[0] // {}) as $manifest_summary
| ($report_summary_file_input[0] // {}) as $report_summary
| ($retest_horizon_plan_file_input[0] // {}) as $retest_horizon_plan
| {
    schema_version:"research_current_approved_batch_driver_summary_v1",
    generated_at:$generated_at,
    run_id:$run_id,
    run_dir:$run_dir,
    manifest_file:$manifest_file,
    manifest_summary_file:$manifest_summary_file,
    research_output_dir:$research_output_dir,
    report_file:$report_file,
    registry_file:(if $registry_file == "" then null else $registry_file end),
    report_summary_file:$report_summary_file,
    retest_horizon_plan_file:$retest_horizon_plan_file,
    retest_horizon_status_file:$retest_horizon_status_file,
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_research_output_only:true,
      shadow_paper_live_enabled:false,
      selected_candidates_require_current_universe:($manifest_summary.safety.selected_candidates_require_current_universe // true)
    },
    stage_state:{
      runtime_alive:null,
      artifact_created:true,
      candidate_generated:(($manifest_summary.selected_candidate_count // 0) > 0),
      research_replay_completed:($report_summary.stage_state.research_replay_completed // false),
      promotion_passed:($report_summary.stage_state.promotion_passed // false),
      shadow_created:($report_summary.stage_state.shadow_created // false),
      paper_created:($report_summary.stage_state.paper_created // false),
      live_enabled:false
    },
    manifest:{
      universe_mode:$manifest_summary.universe_mode,
      dispatch_mode:$manifest_summary.dispatch_mode,
      latest_universe:$manifest_summary.latest_universe,
      scanned_research_eligible_candidate_count:$manifest_summary.scanned_research_eligible_candidate_count,
      current_observed_candidate_count:$manifest_summary.current_observed_candidate_count,
      current_approved_candidate_count:$manifest_summary.current_approved_candidate_count,
      horizon_contract_valid_candidate_count:$manifest_summary.horizon_contract_valid_candidate_count,
      horizon_contract_invalid_candidate_count:$manifest_summary.horizon_contract_invalid_candidate_count,
      excluded_horizon_contract_violations:$manifest_summary.excluded_horizon_contract_violations,
      selected_candidate_count:$manifest_summary.selected_candidate_count,
      eligible_candidate_pool_count:$manifest_summary.eligible_candidate_pool_count,
      selected_candidate_limit_reached:$manifest_summary.selected_candidate_limit_reached,
      unselected_eligible_candidate_count:$manifest_summary.unselected_eligible_candidate_count,
      distinct_candidate_symbols:$manifest_summary.distinct_candidate_symbols,
      eligible_candidate_symbols:$manifest_summary.eligible_candidate_symbols,
      unselected_eligible_candidate_symbols:$manifest_summary.unselected_eligible_candidate_symbols,
      allowed_horizons:$manifest_summary.allowed_horizons,
      selected_current_approved_candidate_count:$manifest_summary.selected_current_approved_candidate_count,
      selected_horizon_contract_valid_count:$manifest_summary.selected_horizon_contract_valid_count,
      historical_replay_run_index_ref_count:$manifest_summary.historical_replay_run_index_ref_count
    },
    report:$report_summary.report,
    bias_counts:$report_summary.bias_counts,
    reason_counts:$report_summary.reason_counts,
    top_blockers:$report_summary.top_blockers,
    next_research_needs:$report_summary.next_research_needs,
    retest_horizon_plan_summary:$retest_horizon_plan.summary
  }
