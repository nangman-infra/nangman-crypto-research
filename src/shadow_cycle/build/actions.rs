use crate::model::ShadowCycleSchedulerAction;

pub(super) fn select_scheduler_action(
    shadow_runs_empty: bool,
    latest_l1_watermark_known: bool,
    target_waiting_count: usize,
    partially_materialized_count: usize,
    deficient_count: usize,
    pending_count: usize,
    sample_ready_count: usize,
) -> (&'static str, ShadowCycleSchedulerAction) {
    if shadow_runs_empty {
        ("NO_SHADOW_CANDIDATES", ShadowCycleSchedulerAction::Noop)
    } else if !latest_l1_watermark_known {
        (
            "DISCOVER_LATEST_MARKET_L1_AS_OF",
            ShadowCycleSchedulerAction::DiscoverMarketL1Watermark,
        )
    } else if target_waiting_count > 0 {
        (
            "WAIT_FOR_TARGET_HOLDING_WINDOW",
            ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes,
        )
    } else if partially_materialized_count > 0 {
        (
            "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION",
            ShadowCycleSchedulerAction::WaitUntilPendingShadowTargetWindowMaterializes,
        )
    } else if deficient_count > 0 {
        (
            "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
            ShadowCycleSchedulerAction::HoldForOperatorReview,
        )
    } else if pending_count > 0 || sample_ready_count > 0 {
        (
            "REVIEW_SHADOW_COMPLETION_EVIDENCE",
            ShadowCycleSchedulerAction::ReviewShadowCompletionEvidence,
        )
    } else {
        (
            "NO_SHADOW_SAMPLE_GAP_DETECTED",
            ShadowCycleSchedulerAction::Noop,
        )
    }
}

pub(super) fn safe_next_actions(source_verdict: &str) -> Vec<String> {
    match source_verdict {
        "DISCOVER_LATEST_MARKET_L1_AS_OF" => vec!["discover_latest_market_l1_as_of"],
        "WAIT_FOR_TARGET_HOLDING_WINDOW" => vec![
            "wait_for_target_holding_window_materialization",
            "keep_shadow_status_pending_until_completion_evidence_exists",
        ],
        "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION" => vec![
            "wait_for_pending_shadow_target_window_materialization",
            "keep_shadow_status_pending_until_completion_evidence_exists",
        ],
        "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" => vec![
            "build_retest_horizon_status_before_focused_accumulation",
            "keep_shadow_status_pending_until_completion_evidence_exists",
        ],
        "REVIEW_SHADOW_COMPLETION_EVIDENCE" => vec!["review_shadow_completion_evidence"],
        _ => Vec::new(),
    }
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

pub(super) fn blocked_actions(source_verdict: &str) -> Vec<String> {
    let mut actions = vec![
        "do_not_mark_pending_shadow_passed_from_sample_counts_only",
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_shadow_sample_gap_manifest",
    ];
    if source_verdict == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" {
        actions.push("do_not_enable_live_from_shadow_accumulation_manifest");
    }
    actions.into_iter().map(ToOwned::to_owned).collect()
}
