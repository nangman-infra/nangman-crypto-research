use super::*;

mod accumulation;

#[cfg(test)]
pub(super) use accumulation::build_shadow_accumulation_manifest_dispatch;
use accumulation::try_build_shadow_accumulation_manifest_from_latest_state;

pub(super) async fn write_shadow_cycle_decision_outputs(
    args: &Args,
    decision: &crate::model::ShadowCycleDecision,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(output_file) = args.shadow_cycle_decision_output_file.as_deref() {
        return write_shadow_cycle_decision(output_file, decision)
            .map(|path| vec![path.display().to_string()]);
    }
    if let Some(output_dir) = args.output_dir.as_deref() {
        return write_shadow_cycle_decision_to_dir(output_dir, decision, output_partition_at_ms)
            .map(|path| vec![path.display().to_string()]);
    }
    if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        return write_shadow_cycle_decision_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            decision,
            output_partition_at_ms,
        )
        .await
        .map(|uri| vec![uri]);
    }
    Err(AppError::config(
        "shadow cycle decision output target is required",
    ))
}

pub(super) async fn run_shadow_cycle_from_latest_state_mode(args: &Args) -> AppResult<RunSummary> {
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let output_bucket = args.output_s3_bucket.as_deref().ok_or_else(|| {
        AppError::config("--run-shadow-cycle-from-latest-state requires --output-s3-bucket")
    })?;
    let shadow_keys = discover_shadow_validation_run_keys_from_s3(
        output_bucket,
        DEFAULT_SHADOW_VALIDATION_RUN_PREFIX,
        DEFAULT_SHADOW_VALIDATION_RUN_READ_LIMIT,
        DEFAULT_SHADOW_VALIDATION_RUN_SCAN_LIMIT,
    )
    .await?;
    let shadow_runs = if shadow_keys.is_empty() {
        Vec::new()
    } else {
        read_shadow_validation_runs_from_s3(output_bucket, &shadow_keys).await?
    };
    let latest_l1_as_of_ms = shadow_cycle_latest_l1_as_of_ms(args).await?;
    let mut decision =
        build_shadow_cycle_decision(&shadow_runs, latest_l1_as_of_ms, output_partition_at_ms);
    let mut focused_retest_manifests_created = 0usize;
    let mut focused_retest_horizon_count = 0usize;
    let mut focused_retest_candidate_bundle_refs = 0usize;
    let mut output_files = Vec::new();

    if decision.scheduler_action == ShadowCycleSchedulerAction::HoldForOperatorReview
        && let Some(dispatch) = try_build_shadow_accumulation_manifest_from_latest_state(
            args,
            &shadow_runs,
            latest_l1_as_of_ms,
            output_partition_at_ms,
        )
        .await?
    {
        if dispatch.created {
            focused_retest_manifests_created = 1;
        }
        focused_retest_horizon_count = dispatch.focused_horizon_count;
        focused_retest_candidate_bundle_refs = dispatch.focused_candidate_bundle_refs;
        output_files.push(dispatch.manifest_uri.clone());
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
        decision.focused_research_manifest_file = Some(dispatch.manifest_uri);
        decision.safe_next_actions = vec![
            "run_focused_shadow_sample_accumulation_research".to_owned(),
            "keep_shadow_status_pending_until_completion_evidence_exists".to_owned(),
        ];
    }
    validate_shadow_cycle_decision(&decision)?;
    let output_files = append_output_files(
        output_files,
        write_shadow_cycle_decision_outputs(args, &decision, output_partition_at_ms).await?,
    );
    emit_shadow_cycle_decision_alert_from_env(&decision).await;

    Ok(RunSummary {
        shadow_cycle_decisions_validated: 1,
        shadow_cycle_decisions_created: 1,
        shadow_cycle_scheduler_action: Some(decision.scheduler_action),
        shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
        shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
        focused_retest_manifests_created,
        focused_retest_horizon_count,
        focused_retest_candidate_bundle_refs,
        shadow_validation_runs_loaded: shadow_runs.len(),
        output_files,
        ..RunSummary::default()
    })
}

fn append_output_files(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    left.extend(right);
    left
}
