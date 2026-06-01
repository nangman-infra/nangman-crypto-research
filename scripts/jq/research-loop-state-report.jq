{
  region:$region,
  buckets:{
    candidate:$candidate_bucket,
    market_l1:$market_l1_bucket,
    research_output:$output_bucket
  },
  stage_state:{
    runtime_alive:$runtime.runtime_alive,
    dispatcher_auto_research_enabled:($runtime.dispatcher_mode == "run_task"),
    major50_universe_observed:$universe.major_coverage_complete,
    major50_universe_approved:$universe.approved_major_coverage_complete,
    candidate_generated:($candidates.recent_candidate_record_count > 0),
    artifact_created:($prefixes.research_run_report.key != null and $prefixes.replay_run.key != null and $prefixes.replay_run_index.key != null),
    research_replay_completed:($research.present and $research.replay_run_count > 0),
    promotion_passed:($research.promotion_bias_count > 0 or $research.shadow_validation_count > 0 or $research.paper_trade_candidate_count > 0),
    shadow_created:($prefixes.shadow_validation_run.key != null),
    paper_created:($prefixes.paper_trade_run.key != null),
    live_enabled:false
  },
  major50_universe:$universe,
  recent_candidates:$candidates,
  latest_research_report:$report,
  best_current_approved_shard_batch:$current_approved_shard_batch,
  recent_research_report_coverage:$recent_report_coverage,
  research_evidence:$research,
  latest_prefixes:$prefixes,
  coverage_gaps:{
    approved_symbols_without_recent_candidate:(($universe.approved_symbols // []) - ($candidates.distinct_candidate_symbols // [])),
    recent_candidate_symbols_without_replay:(
      if (($recent_report_coverage.replayed_symbols // []) | length) > 0 then
        (($candidates.distinct_candidate_symbols // []) - ($recent_report_coverage.replayed_symbols // []))
      elif ($research.present and (($research.top_symbols // []) | length) > 0) then
        (($candidates.distinct_candidate_symbols // []) - ($research.top_symbols // []))
      else
        ($candidates.distinct_candidate_symbols // [])
      end
    ),
    replayed_symbols_without_promotion:(
      if ($research.promotion_bias_count == 0) then
        ($research.top_symbols // [])
      else
        []
      end
    )
  },
  next_decision: (
    (($universe.approved_symbols // []) - ($candidates.distinct_candidate_symbols // [])) as $candidate_gap
    | {
      schema_version:"research_loop_state_decision_v1",
      verdict:(
        if ($runtime.runtime_alive | not) then "RUNTIME_NOT_READY"
        elif ($runtime.dispatcher_mode != "run_task") then "AUTO_RESEARCH_DISABLED"
        elif ($universe.major_coverage_complete | not) then "WAIT_FOR_MAJOR50_OBSERVATION"
        elif ($universe.approved_major_coverage_complete | not) then "WAIT_FOR_MAJOR50_APPROVAL"
        elif (($candidate_gap | length) > 0) then "INCREASE_CANDIDATE_GENERATION_COVERAGE"
        elif (($research.present | not) or $research.replay_run_count == 0) then "RUN_RESEARCH_REPLAY"
        elif ($research.promotion_bias_count == 0 and $research.shadow_validation_count == 0) then "ACCUMULATE_RESEARCH_REPLAY_EVIDENCE"
        elif ($prefixes.shadow_validation_run.key == null) then "REVIEW_PROMOTION_FOR_SHADOW"
        elif ($prefixes.paper_trade_run.key == null) then "WAIT_FOR_PASSED_SHADOW_BEFORE_PAPER"
        else "REVIEW_PAPER_PROGRESS"
        end
      ),
      safe_next_actions:([
        if ($runtime.dispatcher_mode != "run_task") then "keep_dispatcher_dry_run_until_output_write_and_duplicate_controls_are_approved" else empty end,
        if ($universe.major_coverage_complete | not) then "wait_for_major50_observation" else empty end,
        if ($universe.approved_major_coverage_complete | not) then "wait_for_major50_approval" else empty end,
        if (($candidate_gap | length) > 0) then "increase_candidate_generation_for_approved_major50_symbols" else empty end,
        if ($research.present and $research.replay_run_count > 0 and $research.promotion_bias_count == 0) then "keep_accumulating_completed_native_replay_samples" else empty end,
        if (($research.present | not) or $research.replay_run_count == 0) then "run_research_replay_for_recent_candidates" else empty end,
        if ($research.promotion_bias_count > 0 and $prefixes.shadow_validation_run.key == null) then "review_promotion_to_shadow_evidence" else empty end
      ]),
      blocked_actions:[
        "do_not_create_shadow_without_promotion",
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_loop_state"
      ],
      safety:{
        read_only_check:true,
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        paper_live_enabled:false,
        live_enabled:false,
        order_execution_enabled:false
      },
      evidence:{
        dispatcher_mode:$runtime.dispatcher_mode,
        major50_observed:$universe.major_coverage_complete,
        major50_approved:$universe.approved_major_coverage_complete,
        approved_symbols_without_recent_candidate_count:($candidate_gap | length),
        recent_candidate_symbol_count:$candidates.distinct_candidate_symbol_count,
        research_evidence_source:$research.evidence_source,
        research_replay_count:$research.replay_run_count,
        promotion_bias_count:$research.promotion_bias_count,
        shadow_output_present:($prefixes.shadow_validation_run.key != null),
        paper_output_present:($prefixes.paper_trade_run.key != null)
      }
    }
  ),
  bottlenecks:([
    if ($runtime.dispatcher_mode != "run_task") then "dispatcher_not_run_task" else empty end,
    if ($universe.major_coverage_complete | not) then "major50_observed_universe_incomplete" else empty end,
    if ($universe.approved_major_coverage_complete | not) then "major50_approved_universe_incomplete" else empty end,
    if ($candidates.recent_candidate_record_count == 0) then "no_recent_candidate_bundles" else empty end,
    if (($research.present | not) or $research.replay_run_count == 0) then "research_replay_not_completed" else empty end,
    if ($research.promotion_bias_count == 0 and $research.shadow_validation_count == 0) then "no_promoted_shadow_candidate" else empty end,
    if ($prefixes.shadow_validation_run.key == null) then "shadow_output_absent" else empty end,
    if ($prefixes.paper_trade_run.key == null) then "paper_output_absent" else empty end
  ])
}
