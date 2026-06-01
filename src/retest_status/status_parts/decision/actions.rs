pub(in crate::retest_status) fn next_decision_verdict(
    promotion_passed: bool,
    promotion_ready_for_review_count: usize,
    market_l1_coverage_extension_count: usize,
    ready_for_replay_count: usize,
    waiting_for_market_l1_count: usize,
    sample_accumulation_count: usize,
) -> &'static str {
    if promotion_passed {
        "PROMOTE_PRESENT_REVIEW_BEFORE_SHADOW"
    } else if promotion_ready_for_review_count > 0 {
        "PROMOTION_GATE_READY_FOR_REVIEW"
    } else if market_l1_coverage_extension_count > 0 {
        "EXTEND_MARKET_L1_HORIZON_COVERAGE"
    } else if ready_for_replay_count > 0 {
        "REPLAY_READY_FOR_SOME_HORIZONS"
    } else if waiting_for_market_l1_count > 0 {
        "WAIT_FOR_MARKET_L1_HORIZON"
    } else if sample_accumulation_count > 0 {
        "ACCUMULATE_COMPLETED_NATIVE_REPLAY_SAMPLES"
    } else {
        "INSPECT_REMAINING_GATE_REASONS"
    }
}

pub(in crate::retest_status) fn safe_next_actions(
    promotion_passed: bool,
    promotion_ready_for_review_count: usize,
    market_l1_coverage_extension_count: usize,
    ready_for_replay_count: usize,
    waiting_for_market_l1_count: usize,
    sample_accumulation_count: usize,
) -> Vec<String> {
    let mut actions = Vec::new();
    if promotion_passed {
        actions.push("review_promoted_candidates_before_shadow".to_owned());
    }
    if promotion_ready_for_review_count > 0 {
        actions.push("review_promotion_gate_ready_horizons".to_owned());
    }
    if market_l1_coverage_extension_count > 0 {
        actions.push("extend_market_l1_horizon_coverage".to_owned());
    }
    if ready_for_replay_count > 0 {
        actions.push("rerun_current_approved_research_batch_after_market_l1_advances".to_owned());
    }
    if waiting_for_market_l1_count > 0 {
        actions.push("wait_for_market_l1_horizon_materialization".to_owned());
    }
    if sample_accumulation_count > 0 {
        actions.push("keep_accumulating_completed_native_replay_samples".to_owned());
    }
    actions.sort();
    actions.dedup();
    actions
}

pub(in crate::retest_status) fn blocked_actions(
    promotion_passed: bool,
    shadow_created: bool,
) -> Vec<String> {
    let mut actions = Vec::new();
    if !promotion_passed {
        actions.push("do_not_create_shadow_without_promotion".to_owned());
    }
    if !shadow_created {
        actions.push("do_not_create_paper_without_passed_shadow".to_owned());
    }
    actions.push("do_not_enable_live_from_research_batch".to_owned());
    actions
}
