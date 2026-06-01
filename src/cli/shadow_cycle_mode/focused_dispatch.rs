use super::accumulation::try_build_shadow_accumulation_manifest_from_latest_state;
use super::*;

#[derive(Default)]
pub(super) struct FocusedShadowDispatchSummary {
    pub(super) focused_retest_manifests_created: usize,
    pub(super) focused_retest_horizon_count: usize,
    pub(super) focused_retest_candidate_bundle_refs: usize,
    pub(super) output_files: Vec<String>,
}

pub(super) async fn try_apply_focused_shadow_dispatch(
    args: &Args,
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    output_partition_at_ms: i64,
    decision: &mut crate::model::ShadowCycleDecision,
) -> AppResult<FocusedShadowDispatchSummary> {
    if decision.scheduler_action != ShadowCycleSchedulerAction::HoldForOperatorReview {
        return Ok(FocusedShadowDispatchSummary::default());
    }
    let Some(dispatch) = try_build_shadow_accumulation_manifest_from_latest_state(
        args,
        shadow_runs,
        latest_l1_as_of_ms,
        output_partition_at_ms,
    )
    .await?
    else {
        return Ok(FocusedShadowDispatchSummary::default());
    };

    decision.scheduler_action =
        ShadowCycleSchedulerAction::RunFocusedShadowSampleAccumulationResearch;
    let latest_l1_part = latest_l1_as_of_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let deficit_lifecycle_key_part = dispatch.deficit_lifecycle_keys.join("|");
    decision.decision_id = stable_id(
        "shadow_cycle_decision",
        &[
            "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
            latest_l1_part.as_str(),
            dispatch.manifest_uri.as_str(),
            deficit_lifecycle_key_part.as_str(),
        ],
    );
    decision.focused_research_manifest_file = Some(dispatch.manifest_uri.clone());
    decision.safe_next_actions = vec![
        "run_focused_shadow_sample_accumulation_research".to_owned(),
        "keep_shadow_status_pending_until_completion_evidence_exists".to_owned(),
    ];

    Ok(FocusedShadowDispatchSummary {
        focused_retest_manifests_created: usize::from(dispatch.created),
        focused_retest_horizon_count: dispatch.focused_horizon_count,
        focused_retest_candidate_bundle_refs: dispatch.focused_candidate_bundle_refs,
        output_files: vec![dispatch.manifest_uri],
    })
}
