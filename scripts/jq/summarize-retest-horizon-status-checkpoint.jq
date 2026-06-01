include "summarize-retest-horizon-status-lib";
include "summarize-retest-horizon-status-decision";
include "summarize-retest-horizon-status-matrix";
include "summarize-retest-horizon-status-schedule";

def stage_state($rows; $driver; $plan_research_replay_completed):
  {
    candidate_generated:(
      ($driver.stage_state.candidate_generated // false)
      or (($rows | length) > 0)
    ),
    research_replay_completed:(
      $driver.stage_state.research_replay_completed // $plan_research_replay_completed
    ),
    promotion_passed:($driver.stage_state.promotion_passed // false),
    shadow_created:($driver.stage_state.shadow_created // false),
    paper_created:($driver.stage_state.paper_created // false),
    live_enabled:false
  };

def batch_state($driver):
  {
    run_id:($driver.run_id // null),
    universe_mode:($driver.manifest.universe_mode // null),
    dispatch_mode:($driver.manifest.dispatch_mode // null),
    selected_candidate_count:($driver.manifest.selected_candidate_count // null),
    eligible_candidate_pool_count:($driver.manifest.eligible_candidate_pool_count // null),
    selected_candidate_limit_reached:($driver.manifest.selected_candidate_limit_reached // null),
    unselected_eligible_candidate_count:($driver.manifest.unselected_eligible_candidate_count // null),
    selected_current_approved_candidate_count:($driver.manifest.selected_current_approved_candidate_count // null),
    research_report_status:($driver.report.research_run_status // null),
    source_candidate_count:($driver.report.source_candidate_count // null),
    replay_run_count:($driver.report.replay_run_count // null),
    retest_candidate_count:($driver.report.retest_candidate_count // null),
    surviving_candidate_count:($driver.report.surviving_candidate_count // null),
    shadow_validation_count:($driver.report.shadow_validation_count // null),
    paper_trade_candidate_count:($driver.report.paper_trade_candidate_count // null)
  };

def horizon_summary($rows):
  {
    candidate_count:(($rows | map(.candidate_id) | unique) | length),
    horizon_count:($rows | length),
    symbols:($rows | map(.primary_symbol) | unique_sorted),
    next_action_counts:($rows | action_counts),
    ready_for_replay_count:(
      ($rows | count_action("run_research_replay_for_horizon"))
      + ($rows | count_action("materialize_completed_native_replay_sample"))
    ),
    waiting_for_market_l1_count:($rows | count_action("wait_for_market_l1_horizon")),
    market_l1_coverage_extension_count:($rows | count_action("extend_market_l1_horizon_coverage")),
    sample_accumulation_count:($rows | count_action("accumulate_completed_native_replay_samples")),
    promotion_ready_for_review_count:($rows | count_action("promotion_gate_ready_for_review"))
  };

def retest_horizon_status_checkpoint($rows; $driver; $generated_at; $plan_file; $driver_summary_file; $latest_l1_as_of_ms):
  (plan_research_replay_completed($rows)) as $plan_research_replay_completed
  | (candidate_horizon_matrix($rows)) as $candidate_horizon_matrix
  | (next_wait_due_ms($rows)) as $next_wait_due_ms
  | {
      schema_version:"research_horizon_status_checkpoint_v1",
      generated_at:$generated_at,
      retest_horizon_plan_file:$plan_file,
      driver_summary_file:(if $driver_summary_file == "" then null else $driver_summary_file end),
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_summary_only:true,
        shadow_paper_live_enabled:false
      },
      stage_state:stage_state($rows; $driver; $plan_research_replay_completed),
      batch_state:batch_state($driver),
      horizon_summary:horizon_summary($rows),
      materialization_schedule:materialization_schedule($rows; $latest_l1_as_of_ms),
      by_symbol:by_symbol_summary($rows),
      by_horizon:($rows | horizon_counts),
      candidate_horizon_matrix_summary:candidate_horizon_matrix_summary($candidate_horizon_matrix),
      candidate_horizon_matrix:$candidate_horizon_matrix,
      next_decision:retest_next_decision($rows; $driver; $latest_l1_as_of_ms; $next_wait_due_ms)
    };
