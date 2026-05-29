use super::super::*;

pub(in crate::cli) async fn build_shadow_cycle_decision_mode(args: &Args) -> AppResult<RunSummary> {
    let manifest = load_input_manifest(args).await?;
    validate_input_manifest(manifest.as_ref())?;
    let shadow_validation_runs = load_shadow_validation_runs(args, manifest.as_ref()).await?;
    if shadow_validation_runs.is_empty() {
        return Err(AppError::validation(
            "shadow cycle decision build requires at least one shadow validation run",
        ));
    }

    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let decision = build_shadow_cycle_decision(
        &shadow_validation_runs,
        args.shadow_cycle_latest_l1_as_of_ms,
        output_partition_at_ms,
    );
    validate_shadow_cycle_decision(&decision)?;

    let output_files =
        write_shadow_cycle_decision_outputs(args, &decision, output_partition_at_ms).await?;
    emit_shadow_cycle_decision_alert_from_env(&decision).await;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 0,
        retest_cycle_scheduler_action: None,
        retest_cycle_run_not_before_ms: None,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 1,
        shadow_cycle_decisions_created: 1,
        shadow_cycle_scheduler_action: Some(decision.scheduler_action),
        shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
        shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: shadow_validation_runs.len(),
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        paper_watch_observer_iterations: 0,
        paper_watch_observer_snapshots_created: 0,
        paper_watch_observer_active_candidates: 0,
        paper_watch_observer_restored_live_marks: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}
