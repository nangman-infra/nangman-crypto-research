include "summarize-retest-horizon-status-lib";

def retest_next_decision($rows; $driver; $latest_l1_as_of_ms; $next_wait_due_ms):
  {
    verdict:(
      if (($driver.stage_state.promotion_passed // false) == true) then "PROMOTE_PRESENT_REVIEW_BEFORE_SHADOW"
      elif ($rows | count_action("promotion_gate_ready_for_review")) > 0 then "PROMOTION_GATE_READY_FOR_REVIEW"
      elif ($rows | count_action("extend_market_l1_horizon_coverage")) > 0 then "EXTEND_MARKET_L1_HORIZON_COVERAGE"
      elif (
        (($rows | count_action("run_research_replay_for_horizon"))
        + ($rows | count_action("materialize_completed_native_replay_sample"))) > 0
      ) then "REPLAY_READY_FOR_SOME_HORIZONS"
      elif ($rows | count_action("wait_for_market_l1_horizon")) > 0 then "WAIT_FOR_MARKET_L1_HORIZON"
      elif ($rows | count_action("accumulate_completed_native_replay_samples")) > 0 then "ACCUMULATE_COMPLETED_NATIVE_REPLAY_SAMPLES"
      else "INSPECT_REMAINING_GATE_REASONS" end
    ),
    safe_next_actions:[
      if (($driver.stage_state.promotion_passed // false) == true)
        then "review_promoted_candidates_before_shadow"
        else empty end,
      if (($rows | count_action("promotion_gate_ready_for_review")) > 0)
        then "review_promotion_gate_ready_horizons"
        else empty end,
      if (($rows | count_action("extend_market_l1_horizon_coverage")) > 0)
        then "extend_market_l1_horizon_coverage"
        else empty end,
      if (
        (($rows | count_action("run_research_replay_for_horizon"))
        + ($rows | count_action("materialize_completed_native_replay_sample"))) > 0
      ) then "rerun_current_approved_research_batch_after_market_l1_advances"
        else empty end,
      if (($rows | count_action("wait_for_market_l1_horizon")) > 0)
        then "wait_for_market_l1_horizon_materialization"
        else empty end,
      if (($rows | count_action("accumulate_completed_native_replay_samples")) > 0)
        then "keep_accumulating_completed_native_replay_samples"
        else empty end
    ],
    scheduler_hint:{
      latest_l1_as_of_ms:$latest_l1_as_of_ms,
      latest_l1_as_of_iso:iso_ms($latest_l1_as_of_ms),
      run_research_after_l1_as_of_ms:$next_wait_due_ms,
      run_research_after_l1_as_of_iso:iso_ms($next_wait_due_ms),
      wait_deficit_ms:(
        if $latest_l1_as_of_ms == null or $next_wait_due_ms == null then null
        else ([($next_wait_due_ms - $latest_l1_as_of_ms), 0] | max)
        end
      ),
      run_now_replay_ready:(
        (($rows | count_action("run_research_replay_for_horizon"))
        + ($rows | count_action("materialize_completed_native_replay_sample"))) > 0
      ),
      promotion_ready_for_review:(($rows | count_action("promotion_gate_ready_for_review")) > 0)
    },
    blocked_actions:[
      if (($driver.stage_state.promotion_passed // false) != true)
        then "do_not_create_shadow_without_promotion"
        else empty end,
      if (($driver.stage_state.shadow_created // false) != true)
        then "do_not_create_paper_without_passed_shadow"
        else empty end,
      "do_not_enable_live_from_research_batch"
    ]
  };
